use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zeroize::Zeroize;

use crate::error::{AppError, AppResult};
use tauri::Emitter;
use tracing::{debug, error, info, instrument, trace, warn};

/// Connection settings — password intentionally excluded from Debug.
#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSettings {
    pub server_url: String,
    pub username: String,
    pub password: String,
    /// Skip TLS certificate verification (for self-signed certs during development).
    #[serde(default)]
    pub skip_tls_verify: bool,
}

impl std::fmt::Debug for ConnectionSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionSettings")
            .field("server_url", &self.server_url)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("skip_tls_verify", &self.skip_tls_verify)
            .finish()
    }
}

/// Rewrite session URLs to use the user-provided server URL base.
///
/// JMAP servers often return internal hostnames in apiUrl, downloadUrl, uploadUrl,
/// and eventSourceUrl. We replace the scheme+authority portion with whatever the user
/// connected to, preserving the path segment.
fn rewrite_session_urls(session: &mut JmapSession, server_url: &str) {
    let base = url::Url::parse(server_url).unwrap_or_else(|_| url::Url::parse("https://localhost").unwrap());
    let base_authority = format!("{}://{}", base.scheme(), base.host_str().unwrap_or("localhost"));
    // Preserve port if non-default
    let base_authority = match base.port() {
        Some(p) => format!("{}:{}", base_authority, p),
        None => base_authority,
    };

    let mut rewritten = 0u32;

    if let Ok(parsed) = url::Url::parse(&session.api_url) {
        let path = parsed.path();
        session.api_url = format!("{}{}", base_authority, path);
        rewritten += 1;
    }
    if let Ok(parsed) = url::Url::parse(&session.download_url) {
        let path = parsed.path();
        session.download_url = format!("{}{}", base_authority, path);
        rewritten += 1;
    }
    if let Ok(parsed) = url::Url::parse(&session.upload_url) {
        let path = parsed.path();
        session.upload_url = format!("{}{}", base_authority, path);
        rewritten += 1;
    }
    if let Some(ref es_url) = session.event_source_url {
        if let Ok(parsed) = url::Url::parse(es_url) {
            let path = parsed.path();
            let query = parsed.query().map(|q| format!("?{}", q)).unwrap_or_default();
            session.event_source_url = Some(format!("{}{}{}", base_authority, path, query));
            rewritten += 1;
        }
    }

    if rewritten > 0 {
        debug!(rewritten, "rewrote session URLs to match server_url");
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
    /// Original apiUrl from the JMAP session — used as Host header.
    original_api_url: Mutex<Option<String>>,
    mailbox_state: Mutex<Option<String>>,
    email_state: Mutex<Option<String>>,
    sync_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    es_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Default for JmapSessionManager {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
            client: Mutex::new(None),
            credentials: Mutex::new(None),
            original_api_url: Mutex::new(None),
            mailbox_state: Mutex::new(None),
            email_state: Mutex::new(None),
            sync_handle: Mutex::new(None),
            es_handle: Mutex::new(None),
        }
    }
}

impl JmapSessionManager {
    #[instrument(level = "info", skip(self, settings), fields(
        server_url = %settings.server_url,
        username = %settings.username,
        skip_tls = settings.skip_tls_verify
    ))]
    pub async fn connect(&self, settings: ConnectionSettings) -> AppResult<JmapSession> {
        let server_url = settings.server_url.trim_end_matches('/').to_string();
        url::Url::parse(&server_url)
            .map_err(|e| AppError::InvalidUrl(e.to_string()))?;

        info!(%server_url, "building HTTP client");

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(settings.skip_tls_verify)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                let auth = format!("{}:{}", settings.username, settings.password);
                let encoded = base64::engine::general_purpose::STANDARD.encode(auth.as_bytes());
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Basic {}", encoded))
                        .unwrap(),
                );
                headers
            })
            .build()?;

        let well_known_url = format!("{}/.well-known/jmap", server_url);
        debug!(url = %well_known_url, "fetching JMAP session");

        let response = client.get(&well_known_url).send().await;

        let mut jmap_session: JmapSession = match response {
            Ok(resp) if resp.status().is_success() => {
                let status = resp.status().as_u16();
                debug!(status, "session discovery succeeded");
                resp.json().await?
            }
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                warn!(status, body_len = body.len(), "session discovery failed");
                return Err(AppError::AuthFailed(format!("HTTP {} — {}", status, body)));
            }
            Err(e) => {
                error!(%e, "connection error during session discovery");
                return Err(AppError::Connection(e));
            }
        };

        let account_ids: Vec<&String> = jmap_session.accounts.keys().collect();
        let primary_mail = jmap_session.primary_accounts.get("urn:ietf:params:jmap:mail");
        info!(
            username = %jmap_session.username,
            accounts = ?account_ids,
            primary_mail = ?primary_mail,
            api_url = %jmap_session.api_url,
            "JMAP session established"
        );

        // Rewrite session URLs to use the user-provided server URL
        // instead of internal hostnames (e.g. mail.horde.local → user's IP/hostname).
        let original_api_url = jmap_session.api_url.clone();
        rewrite_session_urls(&mut jmap_session, &server_url);

        if original_api_url != jmap_session.api_url {
            info!(
                original = %original_api_url,
                rewritten = %jmap_session.api_url,
                "apiUrl rewritten"
            );
        }

        *self.session.lock().unwrap() = Some(jmap_session.clone());
        *self.client.lock().unwrap() = Some(client);
        *self.credentials.lock().unwrap() = Some(settings);
        *self.original_api_url.lock().unwrap() = Some(original_api_url);

        Ok(jmap_session)
    }

    pub fn disconnect(&self) {
        info!("disconnecting JMAP session");
        self.stop_sync();
        *self.session.lock().unwrap() = None;
        *self.client.lock().unwrap() = None;
        // Zeroize credentials on disconnect
        if let Some(mut creds) = self.credentials.lock().unwrap().take() {
            creds.zeroize();
        }
        *self.original_api_url.lock().unwrap() = None;
        *self.mailbox_state.lock().unwrap() = None;
        *self.email_state.lock().unwrap() = None;
        debug!("session state cleared");
    }

    pub fn get_session(&self) -> AppResult<JmapSession> {
        self.session.lock().unwrap().clone().ok_or(AppError::NotConnected)
    }

    pub fn get_client(&self) -> AppResult<Client> {
        self.client.lock().unwrap().clone().ok_or(AppError::NotConnected)
    }

    /// Send a JMAP request. Validates the response and checks for JMAP-level errors.
    #[instrument(level = "debug", skip(self), fields(
        num_methods = method_calls.len()
    ))]
    pub async fn request(
        &self,
        using: &[&str],
        method_calls: Vec<(String, serde_json::Value, String)>,
    ) -> AppResult<Vec<(String, serde_json::Value, String)>> {
        let client = self.get_client()?;
        let session = self.get_session()?;

        // Log method names for the span
        let method_names: Vec<String> = method_calls.iter().map(|(n, _, _)| n.clone()).collect();
        debug!(methods = ?method_names, "sending JMAP request");

        let body = JmapRequest {
            using: using.iter().map(|s| s.to_string()).collect(),
            method_calls,
        };

        debug!(api_url = %session.api_url, methods = ?method_names, "sending JMAP request");

        let mut req = client
            .post(&session.api_url)
            .timeout(Duration::from_secs(60))
            .json(&body);

        // If the apiUrl was rewritten, set the Host header to the original value
        // so JMAP servers using virtual hosting (e.g. Stalwart) route correctly.
        if let Some(ref original) = self.original_api_url.lock().unwrap().as_ref() {
            if **original != session.api_url {
                if let Ok(parsed) = url::Url::parse(original) {
                    let host = parsed.host_str().unwrap_or("");
                    let host_val = match parsed.port() {
                        Some(p) => format!("{}:{}", host, p),
                        None => host.to_string(),
                    };
                    debug!(host_override = %host_val, "setting Host header");
                    req = req.header(reqwest::header::HOST, host_val);
                }
            }
        }

        let start = std::time::Instant::now();
        let resp = req.send().await?;
        let elapsed = start.elapsed();

        let status = resp.status().as_u16();
        debug!(status, elapsed_ms = elapsed.as_millis() as u64, "received JMAP response");

        if !resp.status().is_success() {
            let detail = resp.text().await.unwrap_or_default();
            warn!(status, detail_len = detail.len(), "JMAP request returned HTTP error");
            return Err(AppError::Api { status, detail });
        }

        let jmap_resp: JmapResponse = resp.json().await?;

        // Check for JMAP-level errors in method responses
        for (name, args, _call_id) in &jmap_resp.method_responses {
            if name.ends_with("Error") {
                let error_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
                warn!(%name, %error_type, %description, "JMAP method returned error");
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
        debug!(state = %state, "mailbox state updated");
        *self.mailbox_state.lock().unwrap() = Some(state);
    }
    pub fn mailbox_state(&self) -> Option<String> {
        self.mailbox_state.lock().unwrap().clone()
    }
    pub fn set_email_state(&self, state: String) {
        debug!(state = %state, "email state updated");
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
        info!("starting background sync (30s polling + EventSource)");
        self.stop_sync();

        let arc_poll = Arc::clone(self);
        let arc_es = Arc::clone(self);
        let app_poll = app.clone();
        let app_es = app.clone();

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

        let es_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            eventsource_loop_with_reconnect(&arc_es, &app_es).await;
        });

        *self.sync_handle.lock().unwrap() = Some(handle);
        *self.es_handle.lock().unwrap() = Some(es_handle);
    }

    pub fn stop_sync(&self) {
        if let Some(handle) = self.sync_handle.lock().unwrap().take() {
            handle.abort();
            debug!("poll sync task aborted");
        }
        if let Some(handle) = self.es_handle.lock().unwrap().take() {
            handle.abort();
            debug!("EventSource task aborted");
        }
    }
}

/// Poll for email and mailbox changes, emit Tauri events.
#[instrument(level = "debug", skip(session, app))]
async fn poll_changes(session: &JmapSessionManager, app: &tauri::AppHandle) {
    let _ = app.emit("jmap://sync-status", "syncing");

    let account_id = match session.primary_mail_account_id() {
        Ok(id) => id,
        Err(_) => return,
    };

    // ── Email changes ──
    if let Some(old) = session.email_state() {
        debug!(account_id = %account_id, since = %old, "polling email changes");
        match crate::jmap::client::get_email_changes(session, &account_id, &old, None).await {
            Ok(changes) => {
                let total = changes.created.len() + changes.updated.len() + changes.destroyed.len();
                if total > 0 {
                    info!(
                        created = changes.created.len(),
                        updated = changes.updated.len(),
                        destroyed = changes.destroyed.len(),
                        "email changes detected"
                    );
                    let _ = app.emit(
                        "jmap://emails-changed",
                        serde_json::json!({
                            "created": changes.created,
                            "updated": changes.updated,
                            "destroyed": changes.destroyed,
                        }),
                    );
                } else {
                    debug!("no email changes");
                }
                if changes.has_more_changes {
                    debug!("hasMoreChanges — draining remaining");
                    let mut since = changes.new_state;
                    for i in 0..10 {
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
                            Err(e) => {
                                warn!(drain_iteration = i, %e, "Email/changes drain failed");
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => warn!(%e, "Email/changes poll failed"),
        }
    } else {
        debug!("no email state yet — skipping email changes poll");
    }

    // ── Mailbox changes ──
    if let Some(old) = session.mailbox_state() {
        debug!(since = %old, "polling mailbox changes");
        if let Ok(changes) =
            crate::jmap::client::get_mailbox_changes(session, &account_id, &old).await
        {
            let total = changes.created.len() + changes.updated.len() + changes.destroyed.len();
            if total > 0 {
                info!(
                    created = changes.created.len(),
                    updated = changes.updated.len(),
                    destroyed = changes.destroyed.len(),
                    "mailbox changes detected"
                );
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

    let _ = app.emit("jmap://sync-status", "synced");
}

/// EventSource push listener — polls once per SSE event (blank line delimiter).
/// Reconnects with exponential backoff when the stream drops.
#[instrument(level = "info", skip(session, app))]
async fn eventsource_loop_with_reconnect(session: &JmapSessionManager, app: &tauri::AppHandle) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(60);
    let mut attempt = 0u32;

    loop {
        attempt += 1;

        // Check we still have a session (may have been disconnected)
        let es_url = match session.event_source_url() {
            Some(u) => u,
            None => {
                info!("no EventSource URL — stopping listener");
                return;
            }
        };
        let client = match session.get_client() {
            Ok(c) => c,
            Err(_) => {
                info!("not connected — stopping listener");
                return;
            }
        };

        debug!(
            attempt,
            url = %es_url,
            backoff_ms = backoff.as_millis() as u64,
            "connecting to EventSource"
        );

        // IMPORTANT: Do NOT set a timeout for streaming responses.
        // Duration::from_secs(0) means "timeout immediately" in reqwest.
        // We must omit .timeout() entirely for SSE.
        let mut es_req = client
            .get(&es_url)
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache");

        // Set Host header if URL was rewritten
        if let Some(ref original) = session.original_api_url.lock().unwrap().as_ref() {
            if let Ok(parsed) = url::Url::parse(original) {
                let host = parsed.host_str().unwrap_or("");
                let host_val = match parsed.port() {
                    Some(p) => format!("{}:{}", host, p),
                    None => host.to_string(),
                };
                es_req = es_req.header(reqwest::header::HOST, host_val);
            }
        }

        let resp = match es_req.send().await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let status = r.status().as_u16();
                warn!(status, backoff_ms = backoff.as_millis() as u64, "EventSource returned non-success status");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
                continue;
            }
            Err(e) => {
                warn!(%e, backoff_ms = backoff.as_millis() as u64, "EventSource connection failed");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
                continue;
            }
        };

        info!("EventSource connected — push notifications active");
        let _ = app.emit("jmap://sync-status", "push-connected");
        backoff = Duration::from_secs(1); // reset on successful connect
        attempt = 0;

        use futures_util::StreamExt;
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut has_pending_data = false;
        let mut event_count = 0u64;

        loop {
            match stream.next().await {
                Some(Ok(bytes)) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));

                    // Process complete lines
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].trim_end_matches('\r').to_string();
                        buffer = buffer[pos + 1..].to_string();

                        if line.is_empty() {
                            // End of event — poll for changes if we saw data
                            if has_pending_data {
                                event_count += 1;
                                debug!(event = event_count, "SSE event boundary — polling for changes");
                                poll_changes(session, app).await;
                                has_pending_data = false;
                            }
                        } else if let Some(data) = line.strip_prefix("data:") {
                            if !data.trim().is_empty() {
                                has_pending_data = true;
                                trace!(data_len = data.trim().len(), "SSE data received");
                            }
                        }
                        // Ignore id:, retry:, event:, comments
                    }
                }
                _ => {
                    // Stream ended — break inner loop to reconnect
                    warn!(
                        events_seen = event_count,
                        backoff_ms = backoff.as_millis() as u64,
                        "EventSource stream ended — reconnecting"
                    );
                    let _ = app.emit("jmap://sync-status", "push-disconnected");
                    break;
                }
            }
        }
    }
}
