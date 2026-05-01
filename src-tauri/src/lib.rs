use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

// ===== 数据结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u32,
    pub content: String,
    pub timestamp: String,
    pub pinned: bool,
    pub content_type: String,  // "text", "image", "file"
}

// ===== 应用状态 =====

pub struct AppState {
    pub clipboard_history: Mutex<Vec<ClipboardEntry>>,
    pub next_id: Mutex<u32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            clipboard_history: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

// ===== Tauri Commands =====
// 这些函数会被前端调用，#[tauri::command] 是关键标记

/// 获取剪贴板历史记录
#[tauri::command]
fn get_clipboard_history(state: State<AppState>) -> Vec<ClipboardEntry> {
    let history = state.clipboard_history.lock().unwrap();
    history.clone()
}

/// 添加一条剪贴板记录
#[tauri::command]
fn add_clipboard_entry(state: State<AppState>, content: String, content_type: Option<String>) -> ClipboardEntry {
    let mut id_guard = state.next_id.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    let entry = ClipboardEntry {
        id,
        content,
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        pinned: false,
        content_type: content_type.unwrap_or_else(|| "text".to_string()),
    };

    let mut history = state.clipboard_history.lock().unwrap();
    history.insert(0, entry.clone());  // 最新的在前面

    // 最多保存 100 条
    if history.len() > 100 {
        history.truncate(100);
    }

    entry
}

/// 切换固定状态
#[tauri::command]
fn toggle_pin(state: State<AppState>, id: u32) -> bool {
    let mut history = state.clipboard_history.lock().unwrap();
    if let Some(entry) = history.iter_mut().find(|e| e.id == id) {
        entry.pinned = !entry.pinned;
        entry.pinned
    } else {
        false
    }
}

/// 删除一条记录
#[tauri::command]
fn delete_entry(state: State<AppState>, id: u32) -> bool {
    let mut history = state.clipboard_history.lock().unwrap();
    let len_before = history.len();
    history.retain(|e| e.id != id);
    history.len() < len_before
}

/// 清除所有未固定的记录
#[tauri::command]
fn clear_unpinned(state: State<AppState>) -> usize {
    let mut history = state.clipboard_history.lock().unwrap();
    let len_before = history.len();
    history.retain(|e| e.pinned);
    len_before - history.len()
}

// ===== 插件初始化 =====

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            add_clipboard_entry,
            toggle_pin,
            delete_entry,
            clear_unpinned,
        ])
        .run(tauri::generate_context!())
        .expect("error while running clipstash");
}