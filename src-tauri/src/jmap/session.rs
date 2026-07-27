use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};

/// Connection settings provided by the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSettings {
    pub server_url: String,
    pub username: String,
    pub password: String,
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

/// JMAP request envelope (RFC 8620 §2.6).
#[derive(Debug, Serialize)]
struct JmapRequest {
    #[serde(rename = "using")]
    using: Vec<String>,
    #[serde(rename = "methodCalls")]
    method_calls: Vec<(String, serde_json::Value, String)>,
}

/// JMAP response envelope.
#[derive(Debug, Deserialize)]
struct JmapResponse {
    #[serde(rename = "sessionState")]
    session_state: String,
    #[serde(rename = "methodResponses")]
    method_responses: Vec<(String, serde_json::Value, String)>,
    #[serde(rename = "createdIds")]
    created_ids: Option<HashMap<String, String>>,
}

/// Manages the JMAP session: HTTP client, session discovery, and request dispatch.
pub struct JmapSessionManager {
    /// The active session, if connected.
    session: Mutex<Option<JmapSession>>,
    /// HTTP client for JMAP requests.
    client: Mutex<Option<Client>>,
    /// Auth credentials (stored for reconnection / EventSource).
    credentials: Mutex<Option<ConnectionSettings>>,
    /// Cached mailbox state.
    mailbox_state: Mutex<Option<String>>,
    /// Cached email state.
    email_state: Mutex<Option<String>>,
}

impl Default for JmapSessionManager {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
            client: Mutex::new(None),
            credentials: Mutex::new(None),
            mailbox_state: Mutex::new(None),
            email_state: Mutex::new(None),
        }
    }
}

impl JmapSessionManager {
    /// Discover the JMAP session by querying `.well-known/jmap` or the root URL.
    pub async fn connect(&self, settings: ConnectionSettings) -> AppResult<JmapSession> {
        let server_url = settings.server_url.trim_end_matches('/');
        let base_url = url::Url::parse(server_url)
            .map_err(|e| AppError::InvalidUrl(e.to_string()))?;

        // Build a client with Basic auth
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                let auth = format!("{}:{}", settings.username, settings.password);
                let encoded = base64_encode(&auth);
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Basic {}", encoded))
                        .unwrap(),
                );
                headers
            })
            .build()?;

        // Try .well-known/jmap first
        let well_known_url = format!("{}/.well-known/jmap", server_url);
        let response = client.get(&well_known_url).send().await;

        let session: JmapSession = match response {
            Ok(resp) if resp.status().is_success() => resp.json().await?,
            _ => {
                // Fallback: try root URL with Accept: application/json
                let resp = client
                    .get(server_url)
                    .header("Accept", "application/json")
                    .send()
                    .await?
                    .error_for_status()
                    .map_err(|e| AppError::AuthFailed)?;
                resp.json().await?
            }
        };

        *self.session.lock().unwrap() = Some(session.clone());
        *self.client.lock().unwrap() = Some(client);
        *self.credentials.lock().unwrap() = Some(settings);

        Ok(session)
    }

    /// Disconnect and clear state.
    pub fn disconnect(&self) {
        *self.session.lock().unwrap() = None;
        *self.client.lock().unwrap() = None;
        *self.credentials.lock().unwrap() = None;
        *self.mailbox_state.lock().unwrap() = None;
        *self.email_state.lock().unwrap() = None;
    }

    /// Get a reference to the current session.
    pub fn get_session(&self) -> AppResult<JmapSession> {
        self.session
            .lock()
            .unwrap()
            .clone()
            .ok_or(AppError::NotConnected)
    }

    /// Get the HTTP client.
    fn get_client(&self) -> AppResult<Client> {
        self.client
            .lock()
            .unwrap()
            .clone()
            .ok_or(AppError::NotConnected)
    }

    /// Send a JMAP request and return the raw method responses.
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
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(AppError::Api { status, detail });
        }

        let jmap_resp: JmapResponse = resp.json().await?;
        Ok(jmap_resp.method_responses)
    }

    /// Get the primary account ID for mail.
    pub fn primary_mail_account_id(&self) -> AppResult<String> {
        let session = self.get_session()?;
        session
            .primary_accounts
            .get("urn:ietf:params:jmap:mail")
            .cloned()
            .ok_or_else(|| AppError::Session("No primary mail account found".into()))
    }

    /// Set mailbox state (for change tracking).
    pub fn set_mailbox_state(&self, state: String) {
        *self.mailbox_state.lock().unwrap() = Some(state);
    }

    /// Get cached mailbox state.
    pub fn mailbox_state(&self) -> Option<String> {
        self.mailbox_state.lock().unwrap().clone()
    }

    /// Set email state (for change tracking).
    pub fn set_email_state(&self, state: String) {
        *self.email_state.lock().unwrap() = Some(state);
    }

    /// Get cached email state.
    pub fn email_state(&self) -> Option<String> {
        self.email_state.lock().unwrap().clone()
    }
}

// Simple base64 encode (avoid depending on base64 crate for just this)
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
