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

// ── API Calls ──

/// Fetch all mailboxes for the primary account.
pub async fn get_mailboxes(session: &JmapSessionManager) -> AppResult<Vec<Mailbox>> {
    let account_id = session.primary_mail_account_id()?;

    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Mailbox/get".into(),
                serde_json::json!({
                    "accountId": account_id,
                    "ids": null
                }),
                "m1".into(),
            )],
        )
        .await?;

    // Parse the Mailbox/get response
    for (name, args, _) in &responses {
        if name == "Mailbox/get" {
            let list: Vec<Mailbox> = serde_json::from_value(args["list"].clone())
                .map_err(|e| AppError::Other(format!("Failed to parse mailboxes: {}", e)))?;

            // Cache state for change tracking
            if let Some(state) = args["state"].as_str() {
                session.set_mailbox_state(state.to_string());
            }

            return Ok(list);
        }
    }

    Err(AppError::Other("No Mailbox/get response".into()))
}

/// Query emails matching a filter.
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

/// Fetch emails by ID with full properties.
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
