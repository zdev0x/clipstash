use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};

// ===== 数据结构 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u32,
    pub content: String,
    pub timestamp: String,
    pub pinned: bool,
    pub content_type: String,
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

/// 初始化示例数据（后续会被真实剪贴板数据替代）
fn init_sample_data(state: &AppState) {
    let samples = vec![
        ("npm install @tauri-apps/api @tauri-apps/plugin-global-shortcut", "text", true),
        ("git commit -m \"feat: add global shortcut\"", "text", false),
        ("https://github.com/zdev0x/clipstash", "text", false),
        ("SELECT * FROM clipboard_entries ORDER BY created_at DESC;", "text", false),
        ("border-radius: 12px; backdrop-filter: blur(10px);", "text", false),
        ("const result = await invoke('get_clipboard_history');", "text", false),
    ];

    let mut history = state.clipboard_history.lock().unwrap();
    let mut id_guard = state.next_id.lock().unwrap();

    for (content, content_type, pinned) in samples {
        let entry = ClipboardEntry {
            id: *id_guard,
            content: content.to_string(),
            timestamp: chrono::Local::now()
                .format("%H:%M:%S")
                .to_string(),
            pinned,
            content_type: content_type.to_string(),
        };
        history.push(entry);
        *id_guard += 1;
    }
}

// ===== Tauri Commands =====

/// 获取剪贴板历史记录
#[tauri::command]
fn get_clipboard_history(state: State<AppState>) -> Vec<ClipboardEntry> {
    let history = state.clipboard_history.lock().unwrap();
    history.clone()
}

/// 添加一条剪贴板记录
#[tauri::command]
fn add_clipboard_entry(
    state: State<AppState>,
    content: String,
    content_type: Option<String>,
) -> ClipboardEntry {
    let mut id_guard = state.next_id.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    let entry = ClipboardEntry {
        id,
        content,
        timestamp: chrono::Local::now()
            .format("%H:%M:%S")
            .to_string(),
        pinned: false,
        content_type: content_type.unwrap_or_else(|| "text".to_string()),
    };

    let mut history = state.clipboard_history.lock().unwrap();
    history.insert(0, entry.clone());

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

// ===== 插件初始化 =====

pub fn run() {
    let app_state = AppState::default();
    init_sample_data(&app_state);

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
        .manage(app_state)
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
