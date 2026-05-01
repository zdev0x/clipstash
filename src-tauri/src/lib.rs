mod db;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub use db::Database;

// ===== 数据结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u32,
    pub content: String,
    pub timestamp: String,
    pub pinned: bool,
    pub content_type: String,
}

// ===== Tauri Commands =====

/// 获取剪贴板历史记录
#[tauri::command]
fn get_clipboard_history(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ClipboardEntry>, String> {
    db.get_all()
}

/// 添加一条剪贴板记录
#[tauri::command]
fn add_clipboard_entry(
    db: tauri::State<'_, Arc<Database>>,
    content: String,
    content_type: Option<String>,
) -> Result<ClipboardEntry, String> {
    let ct = content_type.unwrap_or_else(|| "text".to_string());
    db.insert(&content, &ct)
}

/// 切换固定状态
#[tauri::command]
fn toggle_pin(db: tauri::State<'_, Arc<Database>>, id: u32) -> Result<bool, String> {
    db.toggle_pin(id)
}

/// 删除一条记录
#[tauri::command]
fn delete_entry(db: tauri::State<'_, Arc<Database>>, id: u32) -> Result<bool, String> {
    db.delete(id)
}

/// 清除所有未固定的记录
#[tauri::command]
fn clear_unpinned(db: tauri::State<'_, Arc<Database>>) -> Result<usize, String> {
    db.clear_unpinned()
}

/// 切换窗口显隐
#[tauri::command]
fn toggle_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

// ===== 数据库路径 =====

fn get_db_path(app: &tauri::AppHandle) -> String {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .to_string_lossy()
        .to_string();

    // 确保目录存在
    std::fs::create_dir_all(&app_dir).ok();

    format!("{}/clipstash.db", app_dir)
}

// ===== 插件初始化 =====

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["super+shift+v", "ctrl+shift+v"])
                .unwrap()
                .with_handler(move |app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 初始化 SQLite 数据库
            let db_path = get_db_path(&app.handle());
            eprintln!("📂 Database path: {}", db_path);

            let database = Database::new(&db_path)
                .expect("Failed to initialize database");

            // 用 Arc 包装以便多线程共享
            app.manage(Arc::new(database));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            add_clipboard_entry,
            toggle_pin,
            delete_entry,
            clear_unpinned,
            toggle_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running clipstash");
}
