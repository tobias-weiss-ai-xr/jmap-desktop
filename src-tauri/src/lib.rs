mod commands;
mod error;
mod jmap;

use jmap::JmapSessionManager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(JmapSessionManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::connect_jmap,
            commands::disconnect_jmap,
            commands::get_session,
            commands::get_mailboxes,
            commands::query_emails,
            commands::get_emails,
            commands::get_threads,
            commands::get_email_changes,
            commands::get_mailbox_changes,
            commands::set_email_keywords,
            commands::move_email,
            commands::delete_email,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
