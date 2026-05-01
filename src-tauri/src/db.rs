use rusqlite::{Connection, params};
use std::path::Path;
use crate::ClipboardEntry;

// ===== 数据库封装 =====

pub struct Database {
    pub conn: std::sync::Mutex<Connection>,
    pub images_dir: String,
}

impl Database {
    /// 创建数据库连接并初始化表结构
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text',
                pinned      INTEGER NOT NULL DEFAULT 0,
                image_path  TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )
        .map_err(|e| format!("Failed to create table: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_history(pinned)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;

        let db_dir = Path::new(db_path).parent().unwrap_or(Path::new("."));
        let images_dir = db_dir.join("images").to_string_lossy().to_string();
        std::fs::create_dir_all(&images_dir).ok();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| row.get(0))
            .unwrap_or(0);

        let db = Self {
            conn: std::sync::Mutex::new(conn),
            images_dir,
        };

        if count == 0 {
            db.seed_data();
        }

        Ok(db)
    }

    fn seed_data(&self) {
        let samples = vec![
            ("npm install @tauri-apps/api", "text", true),
            ("git commit -m \"feat: add image support\"", "text", false),
            ("https://github.com/zdev0x/clipstash", "text", false),
            ("fn main() { println!(\"Hello!\"); }", "text", false),
            ("SELECT * FROM clipboard_history WHERE pinned = 1;", "text", false),
            ("border-radius: 12px; backdrop-filter: blur(10px);", "text", false),
        ];

        let conn = self.conn.lock().unwrap();
        for (content, content_type, pinned) in samples {
            conn.execute(
                "INSERT INTO clipboard_history (content, content_type, pinned) VALUES (?1, ?2, ?3)",
                params![content, content_type, pinned as i32],
            )
            .ok();
        }
    }

    /// 获取所有记录
    pub fn get_all(&self) -> Result<Vec<ClipboardEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, content, content_type, pinned, COALESCE(image_path, ''), created_at 
                 FROM clipboard_history ORDER BY pinned DESC, id DESC",
            )
            .map_err(|e| format!("Prepare failed: {}", e))?;

        // 关键：显式标注返回类型，让 Rust 立即 collect 释放 stmt 的借用
        let rows: Result<Vec<_>, _> = stmt.query_map([], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                pinned: row.get::<_, i32>(3)? == 1,
                timestamp: row.get(5)?,
                image_path: Some(row.get::<_, String>(4)?).filter(|s| !s.is_empty()),
            })
        }).map_err(|e| format!("Query failed: {}", e))?.collect();

        rows.map_err(|e| format!("Row error: {}", e))
    }

    /// 插入文本记录
    pub fn insert_text(&self, content: &str, content_type: &str) -> Result<ClipboardEntry, String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO clipboard_history (content, content_type) VALUES (?1, ?2)",
            params![content, content_type],
        )
        .map_err(|e| format!("Insert failed: {}", e))?;

        let id = conn.last_insert_rowid() as u32;

        Ok(ClipboardEntry {
            id,
            content: content.to_string(),
            content_type: content_type.to_string(),
            pinned: false,
            image_path: None,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    /// 插入图片记录
    pub fn insert_image(&self, content: &str, image_data: &[u8]) -> Result<ClipboardEntry, String> {
        let filename = format!("{}.png", chrono::Local::now().timestamp_millis());
        let filepath = format!("{}/{}", self.images_dir, filename);

        std::fs::write(&filepath, image_data)
            .map_err(|e| format!("Failed to save image: {}", e))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO clipboard_history (content, content_type, image_path) VALUES (?1, 'image', ?2)",
            params![content, filepath],
        )
        .map_err(|e| format!("Insert failed: {}", e))?;

        let id = conn.last_insert_rowid() as u32;

        Ok(ClipboardEntry {
            id,
            content: content.to_string(),
            content_type: "image".to_string(),
            pinned: false,
            image_path: Some(filepath),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    /// 切换固定状态
    pub fn toggle_pin(&self, id: u32) -> Result<bool, String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE clipboard_history SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Toggle pin failed: {}", e))?;

        let pinned: i32 = conn
            .query_row(
                "SELECT pinned FROM clipboard_history WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Query failed: {}", e))?;

        Ok(pinned == 1)
    }

    /// 删除一条记录
    pub fn delete(&self, id: u32) -> Result<bool, String> {
        let image_path: Option<String> = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT image_path FROM clipboard_history WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok()
        };

        if let Some(Some(path)) = image_path {
            std::fs::remove_file(&path).ok();
        }

        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete failed: {}", e))?;

        Ok(affected > 0)
    }

    /// 清除所有未固定的记录
    pub fn clear_unpinned(&self) -> Result<usize, String> {
        // 先获取图片路径列表
        let image_paths: Vec<String> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT image_path FROM clipboard_history WHERE pinned = 0 AND image_path IS NOT NULL")
                .map_err(|e| format!("Prepare failed: {}", e))?;

            // 关键：显式标注类型，强制立即 collect
            let paths: Result<Vec<String>, _> = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| format!("Query failed: {}", e))?
                .collect();

            paths.map_err(|e| format!("Row error: {}", e))?
        };

        for path in &image_paths {
            std::fs::remove_file(path).ok();
        }

        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM clipboard_history WHERE pinned = 0", [])
            .map_err(|e| format!("Clear failed: {}", e))?;

        Ok(affected)
    }
}
