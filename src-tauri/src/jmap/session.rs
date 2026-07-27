use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use tauri::Emitter;

/// Connection settings — password intentionally excluded from Debug.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSettings {
    pub server_url: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
}

impl std::fmt::Debug for ConnectionSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionSettings")
            .field("server_url", &self.server_url)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

impl Drop for ConnectionSettings {
    fn drop(&mut self) {
        // Zero the password on drop
        self.password.zeroize();
        self.server_url.zeroize();
    }
}

/// Zeroize a string's contents.
trait Zeroize {
    fn zeroize(&mut self);
}
impl Zeroize for String {
    fn zeroize(&mut self) {
        unsafe {
            for b in self.as_bytes_mut() {
                *b = 0;
            }
        }
        self.clear();
    }
}

/// JMAP Session resource (RFC 8620 §2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JmapSession {
    #[serde(rename = "capabilities")]
    pub capabilities: serde_json::Value,
    #[serde(rename = "accounts")]
    pub accounts: HashMap<String, JmapAccount>,
    #[serde(rename = "primaryAccounts")]
    pub primary_accounts: HashMap<String, String>,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
    #[serde(rename = "eventSourceUrl")]
    pub event_source_url: Option<String>,
    #[serde(rename = "state")]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JmapAccount {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "isPersonal")]
    pub is_personal: bool,
    #[serde(rename = "isReadOnly")]
    pub is_read_only: bool,
    #[serde(rename = "accountCapabilities")]
    pub account_capabilities: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JmapRequest {
    #[serde(rename = "using")]
    using: Vec<String>,
    #[serde(rename = "methodCalls")]
    method_calls: Vec<(String, serde_json::Value, String)>,
}

#[derive(Debug, Deserialize)]
struct JmapResponse {
    #[serde(rename = "methodResponses")]
    method_responses: Vec<(String, serde_json::Value, String)>,
}

/// Manages the JMAP session: HTTP client, session discovery, request dispatch, and sync.
pub struct JmapSessionManager {
    session: Mutex<Option<JmapSession>>,
    client: Mutex<Option<Client>>,
    credentials: Mutex<Option<ConnectionSettings>>,
    mailbox_state: Mutex<Option<String>>,
    email_state: Mutex<Option<String>>,
    sync_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Default for JmapSessionManager {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
            client: Mutex::new(None),
            credentials: Mutex::new(None),
            mailbox_state: Mutex::new(None),
            email_state: Mutex::new(None),
            sync_handle: Mutex::new(None),
        }
    }
}

impl JmapSessionManager {
    pub async fn connect(&self, settings: ConnectionSettings) -> AppResult<JmapSession> {
        let server_url = settings.server_url.trim_end_matches('/').to_string();
        url::Url::parse(&server_url)
            .map_err(|e| AppError::InvalidUrl(e.to_string()))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                let auth = format!("{}:{}", settings.username, settings.password);
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Basic {}", base64_encode(&auth)))
                        .unwrap(),
                );
                headers
            })
            .build()?;

        let well_known_url = format!("{}/.well-known/jmap", server_url);
        let response = client.get(&well_known_url).send().await;

        let jmap_session: JmapSession = match response {
            Ok(resp) if resp.status().is_success() => resp.json().await?,
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::AuthFailed(format!("HTTP {} — {}", status, body)));
            }
            Err(e) => return Err(AppError::Connection(e)),
        };

        *self.session.lock().unwrap() = Some(jmap_session.clone());
        *self.client.lock().unwrap() = Some(client);
        *self.credentials.lock().unwrap() = Some(settings);

        Ok(jmap_session)
    }

    pub fn disconnect(&self) {
        self.stop_sync();
        *self.session.lock().unwrap() = None;
        *self.client.lock().unwrap() = None;
        *self.credentials.lock().unwrap() = None;
        *self.mailbox_state.lock().unwrap() = None;
        *self.email_state.lock().unwrap() = None;
    }

    pub fn get_session(&self) -> AppResult<JmapSession> {
        self.session.lock().unwrap().clone().ok_or(AppError::NotConnected)
    }

    pub fn get_client(&self) -> AppResult<Client> {
        self.client.lock().unwrap().clone().ok_or(AppError::NotConnected)
    }

    /// Send a JMAP request. Validates the response and checks for JMAP-level errors.
    pub async fn request(
        &self,
        using: &[&str],
        method_calls: Vec<(String, serde_json::Value, String)>,
    ) -> AppResult<Vec<(String, serde_json::Value, String)>> {
        let client = self.get_client()?;
        let session = self.get_session()?;

        let body = JmapRequest {
            using: using.iter().map(|s| s.to_string()).collect(),
            method_calls,
        };

        let resp = client
            .post(&session.api_url)
            .timeout(Duration::from_secs(60))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                detail: resp.text().await.unwrap_or_default(),
            });
        }

        let jmap_resp: JmapResponse = resp.json().await?;

        // Check for JMAP-level errors in method responses
        for (name, args, _call_id) in &jmap_resp.method_responses {
            if name.ends_with("Error") {
                let error_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
                return Err(AppError::Method {
                    method: name.clone(),
                    description: format!("{}: {}", error_type, description),
                });
            }
        }

        Ok(jmap_resp.method_responses)
    }

    pub fn primary_mail_account_id(&self) -> AppResult<String> {
        let session = self.get_session()?;
        session
            .primary_accounts
            .get("urn:ietf:params:jmap:mail")
            .cloned()
            .ok_or(AppError::Session("No primary mail account found".into()))
    }

    pub fn set_mailbox_state(&self, state: String) {
        *self.mailbox_state.lock().unwrap() = Some(state);
    }
    pub fn mailbox_state(&self) -> Option<String> {
        self.mailbox_state.lock().unwrap().clone()
    }
    pub fn set_email_state(&self, state: String) {
        *self.email_state.lock().unwrap() = Some(state);
    }
    pub fn email_state(&self) -> Option<String> {
        self.email_state.lock().unwrap().clone()
    }

    pub fn event_source_url(&self) -> Option<String> {
        self.session.lock().unwrap().as_ref().and_then(|s| s.event_source_url.clone())
    }

    /// Start background sync (polling + EventSource).
    pub fn start_sync(self: &Arc<Self>, app: tauri::AppHandle) {
        self.stop_sync();

        let arc_poll = Arc::clone(self);
        let arc_es = Arc::clone(self);
        let app_poll = app.clone();
        let app_es = app;

        let handle = tokio::spawn(async move {
            let _ = app_poll.emit("jmap://sync-status", "starting");
            tokio::time::sleep(Duration::from_secs(2)).await;
            poll_changes(&arc_poll, &app_poll).await;

            let mut ticker = tokio::time::interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                poll_changes(&arc_poll, &app_poll).await;
            }
        });

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            eventsource_loop(&arc_es, &app_es).await;
        });

        *self.sync_handle.lock().unwrap() = Some(handle);
    }

    pub fn stop_sync(&self) {
        if let Some(handle) = self.sync_handle.lock().unwrap().take() {
            handle.abort();
        }
    }
}

/// Poll for email and mailbox changes, emit Tauri events.
async fn poll_changes(session: &JmapSessionManager, app: &tauri::AppHandle) {
    let _ = app.emit("jmap://sync-status", "syncing");
    if let Ok(account_id) = session.primary_mail_account_id() {
        if let Some(old) = session.email_state() {
            match crate::jmap::client::get_email_changes(session, &account_id, &old, None).await {
                Ok(changes) => {
                    if !changes.created.is_empty()
                        || !changes.updated.is_empty()
                        || !changes.destroyed.is_empty()
                    {
                        let _ = app.emit(
                            "jmap://emails-changed",
                            serde_json::json!({
                                "created": changes.created,
                                "updated": changes.updated,
                                "destroyed": changes.destroyed,
                            }),
                        );
                    }
                    if changes.has_more_changes {
                        // Drain remaining changes iteratively (non-recursive)
                        let mut since = changes.new_state;
                        for _ in 0..10 {
                            match crate::jmap::client::get_email_changes(session, &account_id, &since, None).await {
                                Ok(c) => {
                                    if c.created.is_empty() && c.updated.is_empty() && c.destroyed.is_empty() {
                                        break;
                                    }
                                    let _ = app.emit(
                                        "jmap://emails-changed",
                                        serde_json::json!({
                                            "created": c.created,
                                            "updated": c.updated,
                                            "destroyed": c.destroyed,
                                        }),
                                    );
                                    if c.has_more_changes { since = c.new_state; } else { break; }
                                }
                                Err(e) => { eprintln!("Email/changes drain failed: {}", e); break; }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Email/changes failed: {}", e),
            }
        }
        if let Some(old) = session.mailbox_state() {
            if let Ok(changes) =
                crate::jmap::client::get_mailbox_changes(session, &account_id, &old).await
            {
                if !changes.created.is_empty()
                    || !changes.updated.is_empty()
                    || !changes.destroyed.is_empty()
                {
                    let _ = app.emit(
                        "jmap://mailboxes-changed",
                        serde_json::json!({
                            "created": changes.created,
                            "updated": changes.updated,
                            "destroyed": changes.destroyed,
                        }),
                    );
                }
            }
        }
    }
    let _ = app.emit("jmap://sync-status", "synced");
}

/// EventSource push listener — properly parses SSE fields.
async fn eventsource_loop(session: &JmapSessionManager, app: &tauri::AppHandle) {
    let es_url = match session.event_source_url() {
        Some(u) => u,
        None => return,
    };
    let client = match session.get_client() {
        Ok(c) => c,
        Err(_) => return,
    };

    let resp = match client
        .get(&es_url)
        .header("Accept", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .timeout(Duration::from_secs(0)) // streaming — no timeout
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("EventSource connect failed: {}", e);
            return;
        }
    };
    if !resp.status().is_success() {
        return;
    }

    let _ = app.emit("jmap://sync-status", "push-connected");

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    let mut current_event = String::new();

    loop {
        match stream.next().await {
            Some(Ok(bytes)) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                // Process complete lines
                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim_end_matches('\r').to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.is_empty() {
                        // End of event — dispatch
                        if !current_event.is_empty() {
                            current_event.clear();
                        }
                    } else if let Some(data) = line.strip_prefix("data:") {
                        // We got data — trigger a poll
                        if !data.trim().is_empty() {
                            poll_changes(session, app).await;
                        }
                    } else if let Some(event) = line.strip_prefix("event:") {
                        current_event = event.trim().to_string();
                    }
                    // Ignore id:, retry:, comments
                }
            }
            _ => {
                let _ = app.emit("jmap://sync-status", "push-disconnected");
                break;
            }
        }
    }
}

fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let mut n = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            n |= (b as u32) << (16 - 8 * i);
        }
        let padding = 3 - chunk.len();
        for i in 0..(4 - padding) {
            result.push(CHARS[((n >> (18 - 6 * i)) & 0x3F) as usize] as char);
        }
        for _ in 0..padding {
            result.push('=');
        }
    }
    result
}
