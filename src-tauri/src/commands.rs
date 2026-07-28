use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, info, warn};

use crate::jmap::client;
use crate::jmap::{ConnectionSettings, JmapSessionManager};
use tauri::{Emitter, State};

// ── Configuration ──

/// Check for preconfigured JMAP settings from environment variables.
/// Returns ConnectionSettings if JMAP_SERVER_URL, JMAP_USERNAME, and JMAP_PASSWORD are all set.
#[tauri::command]
pub fn get_preconfigured_settings() -> Option<ConnectionSettings> {
    let server_url = std::env::var("JMAP_SERVER_URL").ok()?;
    let username = std::env::var("JMAP_USERNAME").ok()?;
    let password = std::env::var("JMAP_PASSWORD").ok()?;

    if server_url.is_empty() || username.is_empty() || password.is_empty() {
        return None;
    }

    let skip_tls_verify =
        std::env::var("JMAP_SKIP_TLS_VERIFY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

    info!(
        %server_url,
        %username,
        has_password = !password.is_empty(),
        skip_tls_verify,
        "preconfigured settings loaded from env vars"
    );

    Some(ConnectionSettings {
        server_url: server_url.trim_end_matches('/').to_string(),
        username,
        password,
        skip_tls_verify,
    })
}

// ── Connection ──

#[tauri::command]
pub async fn connect_jmap(
    session: State<'_, Arc<JmapSessionManager>>,
    app: tauri::AppHandle,
    settings: ConnectionSettings,
) -> Result<serde_json::Value, String> {
    debug!(server_url = %settings.server_url, username = %settings.username, "connect_jmap called");
    let jmap_session = session.connect(settings).await.map_err(|e| {
        warn!(%e, "connect_jmap failed");
        e.to_string()
    })?;
    session.start_sync(app.clone());
    let _ = app.emit("jmap://connected", true);
    info!("JMAP connected and sync started");
    serde_json::to_value(jmap_session).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disconnect_jmap(
    session: State<'_, Arc<JmapSessionManager>>,
    app: tauri::AppHandle,
) {
    info!("disconnect_jmap called");
    session.disconnect();
    let _ = app.emit("jmap://connected", false);
}

#[tauri::command]
pub fn get_session(
    session: State<'_, Arc<JmapSessionManager>>,
) -> Result<serde_json::Value, String> {
    session
        .get_session()
        .map_err(|e| e.to_string())
        .and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string()))
}

// ── Mailbox ──

#[tauri::command]
pub async fn get_mailboxes(
    session: State<'_, Arc<JmapSessionManager>>,
) -> Result<Vec<serde_json::Value>, String> {
    let list = client::get_mailboxes(&session)
        .await
        .map_err(|e| {
            warn!(%e, "get_mailboxes failed");
            e.to_string()
        })?;
    debug!(count = list.len(), "get_mailboxes succeeded");
    Ok(list
        .into_iter()
        .map(|m| serde_json::to_value(m).unwrap())
        .collect())
}

#[tauri::command]
pub async fn create_mailbox(
    session: State<'_, Arc<JmapSessionManager>>,
    name: String,
    parent_id: Option<String>,
    role: Option<String>,
) -> Result<serde_json::Value, String> {
    debug!(%name, ?parent_id, ?role, "create_mailbox");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let mut create = serde_json::json!({ "name": name });
    if let Some(pid) = parent_id {
        create["parentId"] = serde_json::json!(pid);
    }
    if let Some(r) = role {
        create["role"] = serde_json::json!(r);
    }
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Mailbox/set".into(),
                serde_json::json!({ "accountId": account_id, "create": { "m1": create } }),
                "ms1".into(),
            )],
        )
        .await
        .map_err(|e| {
            warn!(%e, "create_mailbox failed");
            e.to_string()
        })?;
    for (name, args, _) in &responses {
        if name == "Mailbox/set" {
            return Ok(args.clone());
        }
    }
    Err("No Mailbox/set response".into())
}

// ── Email Read ──

#[tauri::command]
pub async fn query_emails(
    session: State<'_, Arc<JmapSessionManager>>,
    filter: serde_json::Value,
    sort: serde_json::Value,
    limit: Option<u64>,
    position: Option<u64>,
) -> Result<serde_json::Value, String> {
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::query_emails(
        &session,
        &account_id,
        filter,
        sort,
        limit.unwrap_or(50),
        position.unwrap_or(0),
    )
    .await
    .map_err(|e| {
        warn!(%e, "query_emails failed");
        e.to_string()
    })?;
    debug!(ids = result.ids.len(), total = ?result.total, "query_emails succeeded");
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_emails(
    session: State<'_, Arc<JmapSessionManager>>,
    ids: Vec<String>,
    _properties: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    if ids.is_empty() {
        return Ok(serde_json::json!({ "list": [], "notFound": [] }));
    }
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let emails = client::get_emails(&session, &account_id, &ids)
        .await
        .map_err(|e| {
            warn!(%e, "get_emails failed");
            e.to_string()
        })?;
    debug!(returned = emails.len(), requested = ids.len(), "get_emails succeeded");
    Ok(serde_json::json!({
        "accountId": account_id,
        "state": session.email_state(),
        "list": emails,
        "notFound": []
    }))
}

#[tauri::command]
pub async fn search_emails(
    session: State<'_, Arc<JmapSessionManager>>,
    text: String,
    limit: Option<u64>,
) -> Result<serde_json::Value, String> {
    debug!(query = %text, "search_emails");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::search_emails(&session, &account_id, &text, limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())?;
    debug!(results = result.ids.len(), "search_emails succeeded");
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_threads(
    session: State<'_, Arc<JmapSessionManager>>,
    ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    debug!(count = ids.len(), "get_threads");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Thread/get".into(),
                serde_json::json!({ "accountId": account_id, "ids": ids }),
                "t1".into(),
            )],
        )
        .await
        .map_err(|e| e.to_string())?;
    for (name, args, _) in &responses {
        if name == "Thread/get" {
            return Ok(args.clone());
        }
    }
    Err("No Thread/get response".into())
}

// ── Email Mutations ──

#[tauri::command]
pub async fn set_email_keywords(
    session: State<'_, Arc<JmapSessionManager>>,
    id: String,
    keywords: HashMap<String, bool>,
) -> Result<serde_json::Value, String> {
    debug!(email_id = %id, keywords = ?keywords, "set_email_keywords");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::update_email(
        &session,
        &account_id,
        &id,
        serde_json::json!({ "keywords": keywords }),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// Mark an email as seen (read).
#[tauri::command]
pub async fn mark_seen(
    session: State<'_, Arc<JmapSessionManager>>,
    id: String,
    seen: bool,
) -> Result<serde_json::Value, String> {
    debug!(email_id = %id, seen, "mark_seen");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::update_email(
        &session,
        &account_id,
        &id,
        serde_json::json!({ "keywords/$seen": seen }),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// Toggle flagged on an email.
#[tauri::command]
pub async fn toggle_flagged(
    session: State<'_, Arc<JmapSessionManager>>,
    id: String,
    value: bool,
) -> Result<serde_json::Value, String> {
    debug!(email_id = %id, value, "toggle_flagged");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::update_email(
        &session,
        &account_id,
        &id,
        serde_json::json!({ "keywords/$flagged": value }),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// Move an email to another mailbox. Removes from all other mailboxes.
#[tauri::command]
pub async fn move_email(
    session: State<'_, Arc<JmapSessionManager>>,
    id: String,
    to_mailbox_id: String,
) -> Result<serde_json::Value, String> {
    debug!(email_id = %id, to_mailbox_id = %to_mailbox_id, "move_email");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::move_email_to(
        &session,
        &account_id,
        &id,
        &to_mailbox_id,
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// Delete an email (move to trash if available, else destroy).
#[tauri::command]
pub async fn delete_email(
    session: State<'_, Arc<JmapSessionManager>>,
    id: String,
) -> Result<serde_json::Value, String> {
    debug!(email_id = %id, "delete_email");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;

    // Try to find a trash mailbox
    let mailboxes = client::get_mailboxes(&session)
        .await
        .map_err(|e| e.to_string())?;
    let trash = mailboxes
        .iter()
        .find(|m| m.role.as_deref() == Some("trash"))
        .map(|m| m.id.clone());

    if let Some(trash_id) = trash {
        debug!(trash_id = %trash_id, "moving to trash");
        let result = client::move_email_to(&session, &account_id, &id, &trash_id)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    } else {
        warn!("no trash mailbox — hard destroying email");
        let result = client::destroy_emails(&session, &account_id, &[id], None)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn send_email(
    session: State<'_, Arc<JmapSessionManager>>,
    from: String,
    to: Vec<String>,
    subject: String,
    body_text: String,
    body_html: Option<String>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    _reply_to_id: Option<String>,
) -> Result<serde_json::Value, String> {
    info!(
        from = %from,
        to_count = to.len(),
        cc_count = cc.as_ref().map(|c| c.len()).unwrap_or(0),
        subject_len = subject.len(),
        "send_email"
    );
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;

    // Parse from address — split "Name <email>" or use raw email
    let (from_name, from_email) = parse_address(&from);

    let mut email_create = serde_json::json!({
        "from": [{ "name": from_name, "email": from_email }],
        "to": to.iter().map(|a| parse_address_to_json(a)).collect::<Vec<_>>(),
        "subject": subject,
    });
    if let Some(list) = cc {
        email_create["cc"] = serde_json::json!(
            list.iter().map(|a| parse_address_to_json(a)).collect::<Vec<_>>()
        );
    }
    if let Some(list) = bcc {
        email_create["bcc"] = serde_json::json!(
            list.iter().map(|a| parse_address_to_json(a)).collect::<Vec<_>>()
        );
    }

    if let Some(html) = body_html {
        email_create["htmlBody"] =
            serde_json::json!([{ "partId": "1", "type": "text/html" }]);
        email_create["bodyValues"] =
            serde_json::json!({ "1": { "value": html, "charset": "utf-8" } });
    } else {
        email_create["textBody"] =
            serde_json::json!([{ "partId": "1", "type": "text/plain" }]);
        email_create["bodyValues"] =
            serde_json::json!({ "1": { "value": body_text, "charset": "utf-8" } });
    }

    client::submit_email(&session, &account_id, None, Some(email_create))
        .await
        .map_err(|e| {
            warn!(%e, "send_email failed");
            e.to_string()
        })
}

// ── Changes ──

#[tauri::command]
pub async fn get_email_changes(
    session: State<'_, Arc<JmapSessionManager>>,
    since_state: String,
    max_changes: Option<u64>,
) -> Result<serde_json::Value, String> {
    debug!(since = %since_state, ?max_changes, "get_email_changes");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::get_email_changes(
        &session,
        &account_id,
        &since_state,
        max_changes,
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mailbox_changes(
    session: State<'_, Arc<JmapSessionManager>>,
    since_state: String,
) -> Result<serde_json::Value, String> {
    debug!(since = %since_state, "get_mailbox_changes");
    let account_id = session
        .primary_mail_account_id()
        .map_err(|e| e.to_string())?;
    let result = client::get_mailbox_changes(&session, &account_id, &since_state)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

// ── Helpers ──

/// Parse an email address string into (name, email).
/// Handles "Name <email>" and bare "email" formats.
fn parse_address(addr: &str) -> (String, String) {
    let addr = addr.trim();
    if let Some((name, email)) = addr.split_once('<') {
        let name = name.trim().trim_matches('\"').to_string();
        let email = email.trim().trim_end_matches('>').trim_matches('\"').to_string();
        (name, email)
    } else {
        (String::new(), addr.to_string())
    }
}

/// Parse an address into the JMAP EmailAddress JSON format.
fn parse_address_to_json(addr: &str) -> serde_json::Value {
    let (name, email) = parse_address(addr);
    serde_json::json!({ "name": name, "email": email })
}
