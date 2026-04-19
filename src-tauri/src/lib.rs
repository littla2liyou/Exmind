//! WikiMind Tauri application

use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tauri_plugin_dialog::DialogExt;

// Re-export from wikimind-commands crate
pub use wikimind_commands::{chat, file_ops, ai_proxy, wiki_ops};

fn get_log_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").map(|h| format!("{}/.local/share", h)).unwrap_or_else(|_| ".".to_string())
    };
    PathBuf::from(base).join("WikiMind").join("logs")
}

fn init_logging() {
    let log_dir = get_log_dir();
    fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "wikimind.log",
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive for the lifetime of the application
    std::mem::forget(_guard);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(env_filter)
        .init();

    tracing::info!("WikiMind 日志初始化完成, 日志目录: {}", log_dir.display());
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let result = app.dialog()
        .file()
        .blocking_pick_folder();

    Ok(result.map(|p| p.to_string()))
}

pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            wikimind_commands::file_ops::list_dir,
            pick_folder,
            wikimind_commands::file_ops::read_file,
            wikimind_commands::file_ops::write_file,
            wikimind_commands::file_ops::create_dir,
            wikimind_commands::file_ops::delete_path,
            wikimind_commands::chat::chat,
            wikimind_commands::ai_proxy::list_skills,
            wikimind_commands::ai_proxy::execute_skill,
            wikimind_commands::wiki_ops::get_wiki_page,
            wikimind_commands::wiki_ops::update_wiki_page,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
