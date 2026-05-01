use rusqlite::{Connection, params};
use std::sync::Mutex;
use crate::ClipboardEntry;

// ===== 数据库封装 =====

pub struct Database {
    pub conn: Mutex<Connection>,
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
                created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )
        .map_err(|e| format!("Failed to create table: {}", e))?;

        // 创建索引加速查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_history(pinned)",
            [],
        )
        .map_err(|e| format!("Failed to create index: {}", e))?;

        // 检查是否需要插入初始数据
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| row.get(0))
            .unwrap_or(0);

        if count == 0 {
            Self::seed_data(&conn);
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 插入示例数据
    fn seed_data(conn: &Connection) {
        let samples = vec![
            ("npm install @tauri-apps/api @tauri-apps/plugin-global-shortcut", "text", true),
            ("git commit -m \"feat: add SQLite persistence\"", "text", false),
            ("https://github.com/zdev0x/clipstash", "text", false),
            ("SELECT * FROM clipboard_history WHERE pinned = 1;", "text", false),
            ("border-radius: 12px; backdrop-filter: blur(10px);", "text", false),
            ("const result = await invoke('get_clipboard_history');", "text", false),
        ];

        for (content, content_type, pinned) in samples {
            conn.execute(
                "INSERT INTO clipboard_history (content, content_type, pinned) VALUES (?1, ?2, ?3)",
                params![content, content_type, pinned as i32],
            )
            .ok();
        }
    }

    /// 获取所有剪贴板记录（固定的排前面）
    pub fn get_all(&self) -> Result<Vec<ClipboardEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, content, content_type, pinned, created_at 
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
                    timestamp: row.get(4)?,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// 插入一条新记录
    pub fn insert(&self, content: &str, content_type: &str) -> Result<ClipboardEntry, String> {
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
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        })
    }

    /// 切换固定状态，返回新的 pinned 值
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
        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete failed: {}", e))?;

        Ok(affected > 0)
    }

    /// 清除所有未固定的记录，返回删除数量
    pub fn clear_unpinned(&self) -> Result<usize, String> {
        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM clipboard_history WHERE pinned = 0", [])
            .map_err(|e| format!("Clear failed: {}", e))?;

        Ok(affected)
    }
}
