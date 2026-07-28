use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tracing::{instrument};

use crate::error::AppResult;
use crate::jmap::session::JmapSessionManager;

// ── Types (RFC 8620/8621) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(rename = "role", skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: u32,
    #[serde(rename = "totalEmails")]
    pub total_emails: u64,
    #[serde(rename = "unreadEmails")]
    pub unread_emails: u64,
    #[serde(rename = "totalThreads")]
    pub total_threads: u64,
    #[serde(rename = "unreadThreads")]
    pub unread_threads: u64,
    #[serde(rename = "myRights")]
    pub my_rights: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    #[serde(rename = "partId")]
    pub part_id: Option<String>,
    #[serde(rename = "blobId")]
    pub blob_id: Option<String>,
    #[serde(rename = "type")]
    pub body_type: Option<String>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "charset", skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,
    #[serde(rename = "disposition", skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[serde(rename = "language", skip_serializing_if = "Option::is_none")]
    pub language: Option<Vec<String>>,
    #[serde(rename = "location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "size")]
    pub size: Option<u64>,
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
    #[serde(rename = "subParts", skip_serializing_if = "Option::is_none")]
    pub sub_parts: Option<Vec<BodyPart>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyValue {
    pub value: String,
    #[serde(rename = "charset")]
    pub charset: Option<String>,
    #[serde(rename = "encoding")]
    pub encoding: Option<String>,
    #[serde(rename = "language")]
    pub language: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "blobId")]
    pub blob_id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "mailboxIds")]
    pub mailbox_ids: HashMap<String, bool>,
    #[serde(rename = "keywords")]
    pub keywords: HashMap<String, bool>,
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<EmailAddress>>,
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<EmailAddress>>,
    #[serde(rename = "cc", skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<EmailAddress>>,
    #[serde(rename = "bcc", skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,
    #[serde(rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Vec<EmailAddress>>,
    #[serde(rename = "subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(rename = "sentAt")]
    pub sent_at: Option<String>,
    #[serde(rename = "receivedAt")]
    pub received_at: Option<String>,
    #[serde(rename = "size")]
    pub size: Option<u64>,
    #[serde(rename = "preview", skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(rename = "hasAttachment")]
    pub has_attachment: bool,
    #[serde(rename = "textBody", skip_serializing_if = "Option::is_none")]
    pub text_body: Option<Vec<BodyPart>>,
    #[serde(rename = "htmlBody", skip_serializing_if = "Option::is_none")]
    pub html_body: Option<Vec<BodyPart>>,
    #[serde(rename = "bodyValues", skip_serializing_if = "Option::is_none")]
    pub body_values: Option<HashMap<String, BodyValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Thread {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "emailIds")]
    pub email_ids: Vec<String>,
}

// ── API Functions ──

pub async fn get_mailboxes(
    session: &JmapSessionManager,
) -> AppResult<Vec<Mailbox>> {
    let account_id = session.primary_mail_account_id()?;
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![(
                "Mailbox/get".into(),
                serde_json::json!({
                    "accountId": account_id,
                    "properties": ["id", "name", "parentId", "role", "sortOrder",
                        "totalEmails", "unreadEmails", "totalThreads", "unreadThreads", "myRights"]
                }),
                "mb1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Mailbox/get" {
            if let Some(state) = args.get("newState").and_then(|v| v.as_str()) {
                session.set_mailbox_state(state.to_string());
            }
            if let Some(list) = args.get("list").and_then(|v| v.as_array()) {
                let mailboxes: Vec<Mailbox> =
                    serde_json::from_value(serde_json::Value::Array(list.clone()))?;
                return Ok(mailboxes);
            }
        }
    }
    Ok(vec![])
}

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
                    "position": position,
                    "calculateTotal": true,
                }),
                "eq1".into(),
            )],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/query" {
            let query_state = args
                .get("queryState")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let ids: Vec<String> = args
                .get("ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            // Update email state so we can track changes
            // queryState != email state, tracked separately

            return Ok(QueryResult {
                account_id: account_id.to_string(),
                query_state,
                ids,
                total: args.get("total").and_then(|v| v.as_u64()),
                can_calculate_changes: args
                    .get("canCalculateChanges")
                    .and_then(|v| v.as_bool()),
                position: Some(position),
            });
        }
    }
    Ok(QueryResult {
        account_id: account_id.to_string(),
        query_state: String::new(),
        ids: vec![],
        total: None,
        can_calculate_changes: None,
        position: None,
    })
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub account_id: String,
    pub query_state: String,
    pub ids: Vec<String>,
    pub total: Option<u64>,
    pub can_calculate_changes: Option<bool>,
    pub position: Option<u64>,
}

pub async fn get_emails(
    session: &JmapSessionManager,
    account_id: &str,
    ids: &[String],
) -> AppResult<Vec<Email>> {
    // JMAP servers typically cap at 50 per request; batch if needed
    let chunk_size = 50;
    let mut all_emails = Vec::new();
    for chunk in ids.chunks(chunk_size) {
        let responses = session
            .request(
                &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
                vec![(
                    "Email/get".into(),
                    serde_json::json!({
                        "accountId": account_id,
                        "ids": chunk,
                        "properties": [
                            "id", "blobId", "threadId", "mailboxIds", "keywords",
                            "from", "to", "cc", "bcc", "replyTo", "subject",
                            "sentAt", "receivedAt", "size", "preview", "hasAttachment",
                            "textBody", "htmlBody", "bodyValues"
                        ]
                    }),
                    format!("eg{}", all_emails.len()),
                )],
            )
            .await?;

        for (name, args, _) in &responses {
            if name == "Email/get" {
                if let Some(state) = args.get("newState").and_then(|v| v.as_str()) {
                    session.set_email_state(state.to_string());
                }
                if let Some(list) = args.get("list").and_then(|v| v.as_array()) {
                    let emails: Vec<Email> =
                        serde_json::from_value(serde_json::Value::Array(list.clone()))?;
                    all_emails.extend(emails);
                }
            }
        }
    }
    Ok(all_emails)
}

pub async fn update_email(
    session: &JmapSessionManager,
    account_id: &str,
    id: &str,
    update: serde_json::Value,
    if_in_state: Option<&str>,
) -> AppResult<serde_json::Value> {
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
            vec![("Email/set".into(), args, "es1".into())],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/set" {
            return Ok(args.clone());
        }
    }
    Err(crate::error::AppError::Other("No Email/set response".into()))
}

/// Move an email to a target mailbox. Removes it from all other mailboxes.
pub async fn move_email_to(
    session: &JmapSessionManager,
    account_id: &str,
    id: &str,
    to_mailbox_id: &str,
) -> AppResult<serde_json::Value> {
    // Get current email to know which mailboxes it's in
    let emails = get_emails(session, account_id, &[id.to_string()]).await?;
    let email = emails.into_iter().next();

    let mut update = serde_json::json!({ to_mailbox_id: true });
    if let Some(ref e) = email {
        for (mb_id, _) in &e.mailbox_ids {
            if mb_id != to_mailbox_id {
                update[mb_id] = serde_json::json!(false);
            }
        }
    }

    update_email(session, account_id, id, update, None).await
}

/// Permanently destroy emails.
pub async fn destroy_emails(
    session: &JmapSessionManager,
    account_id: &str,
    ids: &[String],
    if_in_state: Option<&str>,
) -> AppResult<serde_json::Value> {
    let mut args = serde_json::json!({
        "accountId": account_id,
        "destroy": serde_json::json!(ids),
    });
    if let Some(state) = if_in_state {
        args["ifInState"] = serde_json::json!(state);
    }
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![("Email/set".into(), args, "es1".into())],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Email/set" {
            return Ok(args.clone());
        }
    }
    Err(crate::error::AppError::Other("No Email/set response".into()))
}

/// Send an email — creates Email + EmailSubmission in one request.
///
/// Per RFC 8620 §4.1: a create id MUST be resolved in a previous method call.
/// So Email/set (creating #c1) must come FIRST, EmailSubmission/set (referencing #c1) AFTER.
#[instrument(level = "info", skip(session), fields(account_id))]
pub async fn submit_email(
    session: &JmapSessionManager,
    account_id: &str,
    if_in_state: Option<&str>,
    email_create: Option<serde_json::Value>,
) -> AppResult<serde_json::Value> {
    let mut method_calls = Vec::new();

    // Email/set MUST come first — it creates the email with id "c1"
    if let Some(email_create) = email_create {
        let mut email_args = serde_json::json!({
            "accountId": account_id,
            "create": { "c1": email_create }
        });
        if let Some(state) = if_in_state {
            email_args["ifInState"] = serde_json::json!(state);
        }
        method_calls.push(("Email/set".into(), email_args, "es1".into()));
    }

    // EmailSubmission/set comes AFTER — it references the email created as #c1
    let mut submission_args = serde_json::json!({
        "accountId": account_id,
        "create": {
            "s1": {
                "emailId": "#c1"
            }
        }
    });
    if let Some(state) = if_in_state {
        submission_args["ifInState"] = serde_json::json!(state);
    }
    method_calls.push(("EmailSubmission/set".into(), submission_args, "ms1".into()));

    let responses = session
        .request(
            &["urn:ietf:params:jmap:submission", "urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            method_calls,
        )
        .await?;

    // Collect results
    let mut results = serde_json::json!({});
    for (name, args, _) in &responses {
        results[name] = args.clone();
    }
    Ok(results)
}

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

pub async fn get_email_changes(
    session: &JmapSessionManager,
    account_id: &str,
    since_state: &str,
    max_changes: Option<u64>,
) -> AppResult<ChangesResult> {
    let mut args = serde_json::json!({
        "accountId": account_id,
        "sinceState": since_state,
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
            let result: ChangesResult =
                serde_json::from_value(args.clone()).unwrap_or(ChangesResult {
                    account_id: account_id.to_string(),
                    old_state: String::new(),
                    new_state: String::new(),
                    has_more_changes: false,
                    created: vec![],
                    updated: vec![],
                    destroyed: vec![],
                });
            if let Some(new_state) = args.get("newState").and_then(|v| v.as_str()) {
                session.set_email_state(new_state.to_string());
            }
            return Ok(result);
        }
    }
    Ok(ChangesResult {
        account_id: account_id.to_string(),
        old_state: String::new(),
        new_state: String::new(),
        has_more_changes: false,
        created: vec![],
        updated: vec![],
        destroyed: vec![],
    })
}

pub async fn get_mailbox_changes(
    session: &JmapSessionManager,
    account_id: &str,
    since_state: &str,
) -> AppResult<ChangesResult> {
    let args = serde_json::json!({
        "accountId": account_id,
        "sinceState": since_state,
    });
    let responses = session
        .request(
            &["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:core"],
            vec![("Mailbox/changes".into(), args, "mc1".into())],
        )
        .await?;

    for (name, args, _) in &responses {
        if name == "Mailbox/changes" {
            let result: ChangesResult =
                serde_json::from_value(args.clone()).unwrap_or(ChangesResult {
                    account_id: account_id.to_string(),
                    old_state: String::new(),
                    new_state: String::new(),
                    has_more_changes: false,
                    created: vec![],
                    updated: vec![],
                    destroyed: vec![],
                });
            if let Some(new_state) = args.get("newState").and_then(|v| v.as_str()) {
                session.set_mailbox_state(new_state.to_string());
            }
            return Ok(result);
        }
    }
    Ok(ChangesResult {
        account_id: account_id.to_string(),
        old_state: String::new(),
        new_state: String::new(),
        has_more_changes: false,
        created: vec![],
        updated: vec![],
        destroyed: vec![],
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangesResult {
    pub account_id: String,
    #[serde(rename = "oldState")]
    pub old_state: String,
    #[serde(rename = "newState")]
    pub new_state: String,
    #[serde(rename = "hasMoreChanges")]
    pub has_more_changes: bool,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub destroyed: Vec<String>,
}
