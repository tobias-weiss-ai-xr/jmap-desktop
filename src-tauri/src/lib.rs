mod commands;
mod error;
mod jmap;

use std::sync::Arc;

use jmap::JmapSessionManager;

pub fn run() {
    let session = Arc::new(JmapSessionManager::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(session)
        .invoke_handler(tauri::generate_handler![
            commands::get_preconfigured_settings,
            commands::connect_jmap,
            commands::disconnect_jmap,
            commands::get_session,
            commands::get_mailboxes,
            commands::create_mailbox,
            commands::query_emails,
            commands::get_emails,
            commands::search_emails,
            commands::get_threads,
            commands::set_email_keywords,
            commands::mark_seen,
            commands::toggle_flagged,
            commands::move_email,
            commands::delete_email,
            commands::send_email,
            commands::get_email_changes,
            commands::get_mailbox_changes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
