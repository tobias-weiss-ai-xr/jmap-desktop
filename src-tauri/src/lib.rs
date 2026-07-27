mod commands;
mod error;
mod jmap;

use std::sync::Arc;

use jmap::JmapSessionManager;

pub fn run() {
    let session = Arc::new(JmapSessionManager::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(session.clone())
        .setup(move |app| {
            // Store session arc so connect_jmap can start sync with it
            let handle = app.handle().clone();
            let session = Arc::clone(&session);
            // We can't easily pass the Arc back through State, so we start sync
            // from the connect command via a setup closure trick:
            // The Arc is already managed by Tauri — commands access it via State.
            // For sync, we clone the Arc here and store it for the connect command.
            let _ = (handle, session); // used by connect_jmap via State
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            commands::toggle_seen,
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
