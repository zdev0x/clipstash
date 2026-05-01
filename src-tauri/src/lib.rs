mod db;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

pub use db::Database;

// ===== 数据结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u32,
    pub content: String,
    pub timestamp: String,
    pub pinned: bool,
    pub content_type: String,
    pub image_path: Option<String>,
}

// ===== Tauri Commands =====

#[tauri::command]
fn get_clipboard_history(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ClipboardEntry>, String> {
    db.get_all()
}

#[tauri::command]
fn add_clipboard_entry(
    db: tauri::State<'_, Arc<Database>>,
    content: String,
    content_type: Option<String>,
) -> Result<ClipboardEntry, String> {
    let ct = content_type.unwrap_or_else(|| "text".to_string());
    db.insert_text(&content, &ct)
}

#[tauri::command]
async fn capture_clipboard(
    app: tauri::AppHandle,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<ClipboardEntry, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    match app.clipboard().read_text() {
        Ok(text) => {
            if !text.is_empty() {
                return db.insert_text(&text, "text");
            }
        }
        Err(_) => {}
    }

    Err("剪贴板为空，请先复制一些文本内容".to_string())
}

#[tauri::command]
async fn save_image_entry(
    db: tauri::State<'_, Arc<Database>>,
    file_path: String,
) -> Result<ClipboardEntry, String> {
    let image_data = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read image: {}", e))?;

    let filename = std::path::Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "uploaded image".to_string());

    db.insert_image(&format!("🖼️ {}", filename), &image_data)
}

#[tauri::command]
fn toggle_pin(db: tauri::State<'_, Arc<Database>>, id: u32) -> Result<bool, String> {
    db.toggle_pin(id)
}

#[tauri::command]
fn delete_entry(db: tauri::State<'_, Arc<Database>>, id: u32) -> Result<bool, String> {
    db.delete(id)
}

#[tauri::command]
fn clear_unpinned(db: tauri::State<'_, Arc<Database>>) -> Result<usize, String> {
    db.clear_unpinned()
}

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

// ===== 插件初始化 =====

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["ctrl+shift+v"])
                .unwrap()
                .with_handler(|app, _shortcut, event| {
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
            // 安全地获取数据库路径
            let app_dir = app.path().app_data_dir().unwrap_or_else(|_| {
                // 回退到当前目录
                std::env::current_dir().unwrap_or_default()
            });

            // 确保目录存在
            std::fs::create_dir_all(&app_dir).ok();

            let db_path = app_dir.join("clipstash.db");
            eprintln!("📂 Database path: {:?}", db_path);

            // 安全地初始化数据库
            match Database::new(&db_path.to_string_lossy()) {
                Ok(database) => {
                    app.manage(Arc::new(database));
                    eprintln!("✅ Database initialized successfully");
                }
                Err(e) => {
                    eprintln!("❌ Database init failed: {}", e);
                    // 创建一个空数据库作为后备
                    let database = Database::new_empty();
                    app.manage(Arc::new(database));
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            add_clipboard_entry,
            capture_clipboard,
            save_image_entry,
            toggle_pin,
            delete_entry,
            clear_unpinned,
            toggle_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running clipstash");
}
