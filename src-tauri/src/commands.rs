use crate::error::{AppError, AppResult};
use crate::jmap::client;
use crate::jmap::{ConnectionSettings, JmapSessionManager};
use tauri::State;

/// Connect to a JMAP server and discover the session.
#[tauri::command]
pub async fn connect_jmap(
    session: State<'_, JmapSessionManager>,
    settings: ConnectionSettings,
) -> AppResult<serde_json::Value> {
    let jmap_session = session.connect(settings).await?;
    Ok(serde_json::to_value(jmap_session)?)
}

/// Disconnect and clear all state.
#[tauri::command]
pub fn disconnect_jmap(session: State<'_, JmapSessionManager>) {
    session.disconnect();
}

/// Get the current JMAP session info.
#[tauri::command]
pub fn get_session(session: State<'_, JmapSessionManager>) -> AppResult<serde_json::Value> {
    let jmap_session = session.get_session()?;
    Ok(serde_json::to_value(jmap_session)?)
}

/// Get all mailboxes.
#[tauri::command]
pub async fn get_mailboxes(session: State<'_, JmapSessionManager>) -> AppResult<Vec<serde_json::Value>> {
    let mailboxes = client::get_mailboxes(&session).await?;
    Ok(mailboxes
        .into_iter()
        .map(|m| serde_json::to_value(m).unwrap())
        .collect())
}

/// Query emails by filter.
#[tauri::command]
pub async fn query_emails(
    session: State<'_, JmapSessionManager>,
    filter: serde_json::Value,
    sort: serde_json::Value,
    limit: Option<u64>,
    position: Option<u64>,
    anchor: Option<String>,
) -> AppResult<serde_json::Value> {
    let account_id = session.primary_mail_account_id()?;
    let limit = limit.unwrap_or(50);
    let position = position.unwrap_or(0);

    let mut filter_val = filter;
    // If anchor is provided, use it as the mailbox filter
    if let Some(anchor_ref) = &anchor {
        filter_val = serde_json::json!({ "inMailbox": anchor_ref });
    }

    let result = client::query_emails(&session, &account_id, filter_val, sort, limit, position).await?;
    Ok(serde_json::to_value(result)?)
}

/// Fetch full email objects by ID.
#[tauri::command]
pub async fn get_emails(
    session: State<'_, JmapSessionManager>,
    ids: Vec<String>,
    properties: Option<Vec<String>>,
) -> AppResult<serde_json::Value> {
    let account_id = session.primary_mail_account_id()?;
    let emails = client::get_emails(&session, &account_id, &ids).await?;
    Ok(serde_json::json!({
        "accountId": account_id,
        "state": session.email_state(),
        "list": emails,
        "notFound": []
    }))
}

/// Get thread objects.
#[tauri::command]
pub async fn get_threads(
    _session: State<'_, JmapSessionManager>,
    ids: Vec<String>,
) -> AppResult<serde_json::Value> {
    // TODO: Implement thread fetching
    Ok(serde_json::json!({
        "list": [],
        "notFound": ids
    }))
}

/// Get email changes since last state.
#[tauri::command]
pub async fn get_email_changes(
    _session: State<'_, JmapSessionManager>,
    since_state: String,
) -> AppResult<serde_json::Value> {
    // TODO: Implement Email/changes
    Ok(serde_json::json!({
        "oldState": since_state,
        "newState": since_state,
        "hasMoreChanges": false,
        "created": [],
        "updated": [],
        "destroyed": []
    }))
}

/// Get mailbox changes since last state.
#[tauri::command]
pub async fn get_mailbox_changes(
    _session: State<'_, JmapSessionManager>,
    since_state: String,
) -> AppResult<serde_json::Value> {
    // TODO: Implement Mailbox/changes
    Ok(serde_json::json!({
        "oldState": since_state,
        "newState": since_state,
        "hasMoreChanges": false,
        "created": [],
        "updated": [],
        "destroyed": []
    }))
}

/// Set keywords on an email (flag, mark read, etc.).
#[tauri::command]
pub async fn set_email_keywords(
    _session: State<'_, JmapSessionManager>,
    _id: String,
    keywords: std::collections::HashMap<String, bool>,
) -> AppResult<serde_json::Value> {
    // TODO: Implement Email/set update
    Ok(serde_json::json!({
        "newState": "",
        "updated": { "id": null },
        "notUpdated": {}
    }))
}

/// Move an email to another mailbox.
#[tauri::command]
pub async fn move_email(
    _session: State<'_, JmapSessionManager>,
    _id: String,
    _to_mailbox_id: String,
) -> AppResult<serde_json::Value> {
    // TODO: Implement move via Email/set
    Ok(serde_json::json!({
        "newState": "",
        "updated": { "id": null },
        "notUpdated": {}
    }))
}

/// Move an email to trash.
#[tauri::command]
pub async fn delete_email(
    _session: State<'_, JmapSessionManager>,
    _id: String,
) -> AppResult<serde_json::Value> {
    // TODO: Implement delete via Email/set destroy
    Ok(serde_json::json!({
        "newState": "",
        "destroyed": {},
        "notDestroyed": {}
    }))
}
