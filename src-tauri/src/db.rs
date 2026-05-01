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

        // 创建剪贴板历史表
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

        // 确保旧数据库也有 image_path 列（兼容升级）
        let has_image_path: bool = conn
            .prepare("PRAGMA table_info(clipboard_history)")
            .ok()
            .mutate(|stmt| {
                stmt.map(|s| {
                    s.query_map([], |row| row.get::<_, String>(1))
                        .ok()
                        .map(|rows| {
                            rows.filter_map(|r| r.ok())
                                .any(|name| name == "image_path")
                        })
                })
            })
            .flatten()
            .flatten()
            .unwrap_or(true);

        if !has_image_path {
            conn.execute(
                "ALTER TABLE clipboard_history ADD COLUMN image_path TEXT",
                [],
            )
            .ok();
        }

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_history(pinned)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;

        // 计算图片存储目录
        let db_dir = Path::new(db_path).parent().unwrap_or(Path::new("."));
        let images_dir = db_dir.join("images").to_string_lossy().to_string();
        std::fs::create_dir_all(&images_dir).ok();

        // 检查是否需要插入初始数据
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

    /// 插入示例数据
    fn seed_data(&self) {
        let samples = vec![
            ("npm install @tauri-apps/api", "text", true, None),
            ("git commit -m \"feat: add image support\"", "text", false, None),
            ("https://github.com/zdev0x/clipstash", "text", false, None),
            ("fn main() { println!(\"Hello!\"); }", "text", false, None),
            ("SELECT * FROM clipboard_history WHERE pinned = 1;", "text", false, None),
            ("border-radius: 12px; backdrop-filter: blur(10px);", "text", false, None),
        ];

        let conn = self.conn.lock().unwrap();
        for (content, content_type, pinned, image_path) in samples {
            conn.execute(
                "INSERT INTO clipboard_history (content, content_type, pinned, image_path) VALUES (?1, ?2, ?3, ?4)",
                params![content, content_type, pinned as i32, image_path],
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
                 FROM clipboard_history 
                 ORDER BY pinned DESC, id DESC",
            )
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(ClipboardEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_type: row.get(2)?,
                    pinned: row.get::<_, i32>(3)? == 1,
                    timestamp: row.get(5)?,
                    image_path: Some(row.get::<_, String>(4)?)
                        .filter(|s| !s.is_empty()),
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
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
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        })
    }

    /// 插入图片记录
    pub fn insert_image(&self, content: &str, image_data: &[u8]) -> Result<ClipboardEntry, String> {
        // 生成唯一文件名
        let filename = format!("{}.png", chrono::Local::now().timestamp_millis());
        let filepath = format!("{}/{}", self.images_dir, filename);

        // 保存图片文件
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
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
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

    /// 删除一条记录（同时删除图片文件）
    pub fn delete(&self, id: u32) -> Result<bool, String> {
        // 先获取图片路径
        let image_path: Option<String> = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT image_path FROM clipboard_history WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok()
        };

        // 删除图片文件
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
        // 获取所有要删除的图片路径
        let image_paths: Vec<String> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT image_path FROM clipboard_history WHERE pinned = 0 AND image_path IS NOT NULL")
                .map_err(|e| format!("Prepare failed: {}", e))?;

            stmt.query_map([], |row| row.get(0))
                .map_err(|e| format!("Query failed: {}", e))?
                .filter_map(|r| r.ok())
                .collect()
        };

        // 删除图片文件
        for path in image_paths {
            std::fs::remove_file(&path).ok();
        }

        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM clipboard_history WHERE pinned = 0", [])
            .map_err(|e| format!("Clear failed: {}", e))?;

        Ok(affected)
    }
}
