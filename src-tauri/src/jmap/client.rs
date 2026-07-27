use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use crate::jmap::session::JmapSessionManager;

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mailbox {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub sort_order: i64,
    pub total_emails: u64,
    pub unread_emails: u64,
    pub total_threads: u64,
    pub unread_threads: u64,
    #[serde(default)]
    pub is_subscribed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    pub id: String,
    pub blob_id: String,
    pub thread_id: String,
    pub mailbox_ids: HashMap<String, bool>,
    pub keywords: HashMap<String, bool>,
    pub subject: String,
    pub received_at: String,
    pub sent_at: String,
    pub size: u64,
    pub preview: String,
    pub has_attachment: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Vec<EmailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<Vec<BodyPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<Vec<BodyPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<BodyPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_values: Option<HashMap<String, BodyValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    pub part_id: String,
    pub blob_id: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyValue {
    pub value: String,
    pub encoding: String,
    pub is_trusted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub query_state: String,
    pub ids: Vec<String>,
    pub can_calculate_changes: bool,
    pub position: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesResult {
    pub old_state: String,
    pub new_state: String,
    pub has_more_changes: bool,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub destroyed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetResult {
    pub account_id: String,
    pub old_state: Option<String>,
    pub new_state: String,
    pub created: serde_json::Value,
    pub updated: serde_json::Value,
    pub destroyed: serde_json::Value,
    pub not_created: serde_json::Value,
    pub not_updated: serde_json::Value,
    pub not_destroyed: serde_json::Value,
}

// ── Read API ──

/// Fetch all mailboxes.
pub async fn get_mailboxes(session: &JmapSessionManager) -> AppResult<Vec<Mailbox>> {
    let account_id = session.primary_mail_account_id()?;

    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Mailbox/get".into(),
                serde_json::json!({ "accountId": account_id, "ids": null }),
                "m1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Mailbox/get" {
            let list: Vec<Mailbox> = serde_json::from_value(args["list"].clone())
                .map_err(|e| AppError::Other(format!("Failed to parse mailboxes: {}", e)))?;
            if let Some(state) = args["state"].as_str() {
                session.set_mailbox_state(state.to_string());
            }
            return Ok(list);
        }
    }

    Err(AppError::Other("No Mailbox/get response".into()))
}

/// Query emails by filter.
pub async fn query_emails(
    session: &JmapSessionManager,
    account_id: &str,
    filter: serde_json::Value,
    sort: serde_json::Value,
    limit: u64,
    position: u64,
) -> AppResult<QueryResult> {
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Email/query".into(),
                serde_json::json!({
                    "accountId": account_id,
                    "filter": filter,
                    "sort": sort,
                    "limit": limit,
                    "position": position
                }),
                "q1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/query" {
            let result: QueryResult = serde_json::from_value(args.clone())
                .map_err(|e| AppError::Other(format!("Failed to parse query: {}", e)))?;
            return Ok(result);
        }
    }

    Err(AppError::Other("No Email/query response".into()))
}

/// Fetch emails by ID with all properties.
pub async fn get_emails(
    session: &JmapSessionManager,
    account_id: &str,
    ids: &[String],
) -> AppResult<Vec<Email>> {
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Email/get".into(),
                serde_json::json!({
                    "accountId": account_id,
                    "ids": ids,
                    "properties": [
                        "id", "blobId", "threadId", "mailboxIds", "keywords",
                        "from", "to", "cc", "bcc", "replyTo",
                        "subject", "sentAt", "receivedAt", "size", "preview",
                        "hasAttachment", "htmlBody", "textBody", "attachments", "bodyValues"
                    ]
                }),
                "e1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/get" {
            let list: Vec<Email> = serde_json::from_value(args["list"].clone())
                .map_err(|e| AppError::Other(format!("Failed to parse emails: {}", e)))?;
            if let Some(state) = args["state"].as_str() {
                session.set_email_state(state.to_string());
            }
            return Ok(list);
        }
    }

    Err(AppError::Other("No Email/get response".into()))
}

// ── Changes API (RFC 8620 §5) ──

/// Get email changes since a state token.
pub async fn get_email_changes(
    session: &JmapSessionManager,
    account_id: &str,
    since_state: &str,
    max_changes: Option<u64>,
) -> AppResult<ChangesResult> {
    let mut args = serde_json::json!({
        "accountId": account_id,
        "sinceState": since_state
    });
    if let Some(max) = max_changes {
        args["maxChanges"] = serde_json::json!(max);
    }

    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![("Email/changes".into(), args, "ec1".into())],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/changes" {
            let result: ChangesResult = serde_json::from_value(args.clone())
                .map_err(|e| AppError::Other(format!("Failed to parse Email/changes: {}", e)))?;
            session.set_email_state(result.new_state.clone());
            return Ok(result);
        }
    }

    Err(AppError::Other("No Email/changes response".into()))
}

/// Get mailbox changes since a state token.
pub async fn get_mailbox_changes(
    session: &JmapSessionManager,
    account_id: &str,
    since_state: &str,
) -> AppResult<ChangesResult> {
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Mailbox/changes".into(),
                serde_json::json!({
                    "accountId": account_id,
                    "sinceState": since_state
                }),
                "mc1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Mailbox/changes" {
            let result: ChangesResult = serde_json::from_value(args.clone())
                .map_err(|e| AppError::Other(format!("Failed to parse Mailbox/changes: {}", e)))?;
            session.set_mailbox_state(result.new_state.clone());
            return Ok(result);
        }
    }

    Err(AppError::Other("No Mailbox/changes response".into()))
}

// ── Set API (RFC 8620 §6, RFC 8621 §4.6) ──

/// Upload a blob for attachment (RFC 8620 §6.1).
pub async fn upload_blob(
    session: &JmapSessionManager,
    content_type: &str,
    data: Vec<u8>,
) -> AppResult<serde_json::Value> {
    let client = session.get_client()?;
    let sess = session.get_session()?;

    let resp = client
        .post(&session.upload_url())
        .header("Content-Type", content_type)
        .body(data)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::Other(format!("Upload failed: {}", e)))?;

    let result: serde_json::Value = resp.json().await?;
    Ok(result)
}

/// Create an email (draft).
pub async fn create_email(
    session: &JmapSessionManager,
    account_id: &str,
    email_create: serde_json::Value,
) -> AppResult<SetResult> {
    let client_id = "c1";
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:submission", "urn:ietf:params:jmap:core"],
            vec![(
                "Email/set".into(),
                serde_json::json!({
                    "accountId": account_id,
                    "create": { client_id: email_create }
                }),
                "es1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/set" {
            let mut result: SetResult = serde_json::from_value(args.clone())
                .map_err(|e| AppError::Other(format!("Failed to parse Email/set: {}", e)))?;
            result.account_id = account_id.to_string();
            return Ok(result);
        }
    }

    Err(AppError::Other("No Email/set response".into()))
}

/// Update email properties (keywords, mailboxIds).
pub async fn update_email(
    session: &JmapSessionManager,
    account_id: &str,
    id: &str,
    update: serde_json::Value,
    if_in_state: Option<&str>,
) -> AppResult<SetResult> {
    let mut args = serde_json::json!({
        "accountId": account_id,
        "update": { id: update }
    });
    if let Some(state) = if_in_state {
        args["ifInState"] = serde_json::json!(state);
    }

    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![("Email/set".into(), args, "eu1".into())],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/set" {
            let mut result: SetResult = serde_json::from_value(args.clone())
                .map_err(|e| AppError::Other(format!("Failed to parse Email/set: {}", e)))?;
            result.account_id = account_id.to_string();
            return Ok(result);
        }
    }

    Err(AppError::Other("No Email/set response".into()))
}

/// Destroy emails (move to trash or hard delete).
pub async fn destroy_emails(
    session: &JmapSessionManager,
    account_id: &str,
    ids: &[String],
    if_in_state: Option<&str>,
) -> AppResult<SetResult> {
    let mut args = serde_json::json!({
        "accountId": account_id,
        "destroy": ids
    });
    if let Some(state) = if_in_state {
        args["ifInState"] = serde_json::json!(state);
    }

    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![("Email/set".into(), args, "ed1".into())],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/set" {
            let mut result: SetResult = serde_json::from_value(args.clone())
                .map_err(|e| AppError::Other(format!("Failed to parse Email/set: {}", e)))?;
            result.account_id = account_id.to_string();
            return Ok(result);
        }
    }

    Err(AppError::Other("No Email/set response".into()))
}

/// Submit an email for delivery (RFC 8621 §7).
pub async fn submit_email(
    session: &JmapSessionManager,
    account_id: &str,
    email_id: Option<&str>,
    email_create: Option<serde_json::Value>,
) -> AppResult<serde_json::Value> {
    let mut submission = serde_json::json!({});

    if let Some(id) = email_id {
        submission["emailId"] = serde_json::json!(id);
    }

    let mut method_calls = vec![(
        "EmailSubmission/set".into(),
        serde_json::json!({
            "accountId": account_id,
            "create": { "s1": submission }
        }),
        "sub1".into(),
    )];

    // If creating email inline
    if let Some(create) = email_create {
        // Create email + submit in single request
        method_calls.insert(0, (
            "Email/set".into(),
            serde_json::json!({
                "accountId": account_id,
                "create": { "c1": create }
            }),
            "es1".into(),
        ));
        // Update submission to reference created email
        method_calls[1].1.as_object_mut().unwrap()["create"]["s1"] = serde_json::json!({
            "emailId": "#c1"
        });
    }

    let responses = session
        .request(
            &[
                "urn:ietf:params:jmap:mail",
                "urn:ietf:params:jmap:submission",
                "urn:ietf:params:jmap:core",
            ],
            method_calls,
        )
        .await?;

    // Collect all results
    let mut results = HashMap::new();
    for (name, args, _) in &responses {
        results.insert(name.clone(), args.clone());
    }

    Ok(serde_json::to_value(results)?)
}

/// Search emails with text filter.
pub async fn search_emails(
    session: &JmapSessionManager,
    account_id: &str,
    text: &str,
    limit: u64,
) -> AppResult<QueryResult> {
    let filter = serde_json::json!({
        "text": text
    });
    let sort = serde_json::json!([{
        "property": "receivedAt",
        "isAscending": false
    }]);

    query_emails(session, account_id, filter, sort, limit, 0).await
}
