#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::prelude::*;

fn main() {
    init_logging();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "jmap-desktop starting"
    );
    jmap_desktop_lib::run();
}

/// Initialise structured logging.
///
/// - **Debug builds**: `INFO` level, human-readable, colored, to stderr.
/// - **Release builds**: `INFO` level, JSON lines to rolling log files,
///   plus a human-readable copy on stderr.
///
/// Override at runtime with `RUST_LOG`, e.g.:
///   `RUST_LOG=jmap_desktop_lib=debug,reqwest=trace jmap-desktop`
fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .init();
    } else {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("jmap-desktop")
            .join("logs");

        if std::fs::create_dir_all(&log_dir).is_ok() {
            let file_appender =
                tracing_appender::rolling::daily(&log_dir, "jmap-desktop.log");

            // Use non-blocking writer to avoid blocking the async runtime
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .json()
                .with_ansi(false);

            let stderr_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_ansi(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .with(stderr_layer)
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .init();
        }
    }
}
