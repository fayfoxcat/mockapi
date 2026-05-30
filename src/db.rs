use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

use crate::models::*;

/// 初始化数据库表结构
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS mock_apis (
            id            TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            method        TEXT NOT NULL,
            url           TEXT NOT NULL,
            headers       TEXT NOT NULL DEFAULT '{}',
            response_body TEXT NOT NULL DEFAULT '',
            response_type TEXT NOT NULL DEFAULT 'json',
            file_name     TEXT,
            file_path     TEXT,
            content_type  TEXT,
            match_headers TEXT,
            sort_order    INTEGER NOT NULL DEFAULT 0,
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS request_logs (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            api_id       TEXT NOT NULL REFERENCES mock_apis(id) ON DELETE CASCADE,
            timestamp    TEXT NOT NULL,
            method       TEXT NOT NULL,
            url          TEXT NOT NULL,
            headers      TEXT NOT NULL DEFAULT '{}',
            request_body TEXT NOT NULL DEFAULT '',
            status_code  INTEGER NOT NULL,
            client_ip    TEXT NOT NULL,
            user_agent   TEXT NOT NULL,
            error        TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_logs_api_id ON request_logs(api_id);
        CREATE INDEX IF NOT EXISTS idx_apis_url ON mock_apis(url);
        PRAGMA foreign_keys = ON;
        "
    )?;

    // 兼容旧数据库：添加 match_headers 列
    let has_column: bool = conn
        .prepare("PRAGMA table_info(mock_apis)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|col| col == "match_headers");

    if !has_column {
        conn.execute("ALTER TABLE mock_apis ADD COLUMN match_headers TEXT", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_apis_url ON mock_apis(url)", [])?;
        info!("数据库升级：添加 match_headers 列");
    }

    info!("数据库表初始化完成");
    Ok(())
}

/// 从 JSON 文件迁移数据到 SQLite
pub fn migrate_from_json(conn: &Connection, json_path: &Path) -> Result<bool> {
    if !json_path.exists() {
        return Ok(false);
    }

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM mock_apis", [], |row| row.get(0))?;
    if count > 0 {
        info!("数据库已有数据，跳过 JSON 迁移");
        return Ok(false);
    }

    let content = std::fs::read_to_string(json_path)?;
    let apis: Vec<MockApi> = serde_json::from_str(&content)?;

    if apis.is_empty() {
        info!("JSON 文件为空，无需迁移");
        return Ok(false);
    }

    let tx = conn.unchecked_transaction()?;

    for (i, api) in apis.iter().enumerate() {
        tx.execute(
            "INSERT INTO mock_apis (id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, match_headers, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                api.id,
                api.name,
                api.method,
                api.url,
                serde_json::to_string(&api.headers)?,
                api.response_body,
                match api.response_type {
                    ResponseType::Json => "json",
                    ResponseType::File => "file",
                },
                api.file_name,
                api.file_path,
                api.content_type,
                api.match_headers.as_ref().map(|h| serde_json::to_string(h).unwrap_or_default()),
                i as i64,
                api.created_at,
                api.updated_at,
            ],
        )?;

        for log in &api.logs {
            tx.execute(
                "INSERT INTO request_logs (api_id, timestamp, method, url, headers, request_body, status_code, client_ip, user_agent, error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    api.id,
                    log.timestamp,
                    log.method,
                    log.url,
                    serde_json::to_string(&log.headers)?,
                    log.request_body,
                    log.status_code as i64,
                    log.client_ip,
                    log.user_agent,
                    log.error,
                ],
            )?;
        }
    }

    tx.commit()?;

    let backup_path = json_path.with_extension("json.bak");
    std::fs::rename(json_path, &backup_path)?;
    info!(
        "成功迁移 {} 个 API 配置到 SQLite，JSON 已备份为 {:?}",
        apis.len(),
        backup_path
    );

    Ok(true)
}

/// 内部函数：将一行数据库记录映射为 MockApi（不含日志和 match_headers）
fn row_to_api(row: &rusqlite::Row) -> rusqlite::Result<MockApi> {
    let headers_str: String = row.get(4)?;
    let response_type_str: String = row.get(6)?;
    let match_headers_str: Option<String> = row.get(10)?;

    let headers: HashMap<String, String> = serde_json::from_str(&headers_str).unwrap_or_default();
    let response_type = match response_type_str.as_str() {
        "file" => ResponseType::File,
        _ => ResponseType::Json,
    };
    let match_headers: Option<HashMap<String, String>> =
        match_headers_str.and_then(|s| serde_json::from_str(&s).ok());

    Ok(MockApi {
        id: row.get(0)?,
        name: row.get(1)?,
        method: row.get(2)?,
        url: row.get(3)?,
        headers,
        response_body: row.get(5)?,
        response_type,
        file_name: row.get(7)?,
        file_path: row.get(8)?,
        content_type: row.get(9)?,
        match_headers,
        logs: Vec::new(),
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const API_SELECT_COLS: &str =
    "id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, match_headers, created_at, updated_at";

/// 加载所有 API（含日志）
pub fn load_all_apis(conn: &Connection) -> Result<Vec<MockApi>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM mock_apis ORDER BY sort_order, created_at DESC",
        API_SELECT_COLS
    ))?;

    let apis = stmt.query_map([], row_to_api)?;

    let mut result = Vec::new();
    for api_row in apis {
        let mut api = api_row?;
        api.logs = load_logs_for_api(conn, &api.id)?;
        result.push(api);
    }

    Ok(result)
}

/// 按 URL 查询所有匹配的 API（用于请求头条件路由）
pub fn get_apis_by_url(conn: &Connection, url: &str) -> Result<Vec<MockApi>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM mock_apis WHERE url = ?1 ORDER BY sort_order",
        API_SELECT_COLS
    ))?;

    let apis = stmt.query_map(params![url], row_to_api)?;

    let mut result = Vec::new();
    for api in apis {
        result.push(api?);
    }
    Ok(result)
}

/// 根据 URL 和请求头匹配最佳 API（精确匹配优先，无规则兜底）
pub fn match_api_by_headers(
    conn: &Connection,
    url: &str,
    req_headers: &HashMap<String, String>,
) -> Result<Option<MockApi>> {
    let candidates = get_apis_by_url(conn, url)?;

    if candidates.is_empty() {
        return Ok(None);
    }

    // 第一轮：精确匹配（有 match_headers 且所有规则都满足）
    for api in &candidates {
        if let Some(ref rules) = api.match_headers {
            if !rules.is_empty() {
                let all_match = rules.iter().all(|(key, expected)| {
                    req_headers
                        .get(&key.to_lowercase())
                        .map(|v| v == expected)
                        .unwrap_or(false)
                });
                if all_match {
                    return Ok(Some(api.clone()));
                }
            }
        }
    }

    // 第二轮：兜底（无 match_headers 或为空的）
    for api in &candidates {
        match &api.match_headers {
            None => return Ok(Some(api.clone())),
            Some(rules) if rules.is_empty() => return Ok(Some(api.clone())),
            _ => {}
        }
    }

    // 没有任何匹配
    Ok(None)
}

/// 插入或更新 API
pub fn upsert_api(conn: &Connection, api: &MockApi) -> Result<()> {
    conn.execute(
        "INSERT INTO mock_apis (id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, match_headers, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM mock_apis), ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            method = excluded.method,
            url = excluded.url,
            headers = excluded.headers,
            response_body = excluded.response_body,
            response_type = excluded.response_type,
            file_name = excluded.file_name,
            file_path = excluded.file_path,
            content_type = excluded.content_type,
            match_headers = excluded.match_headers,
            updated_at = excluded.updated_at",
        params![
            api.id,
            api.name,
            api.method,
            api.url,
            serde_json::to_string(&api.headers)?,
            api.response_body,
            match api.response_type {
                ResponseType::Json => "json",
                ResponseType::File => "file",
            },
            api.file_name,
            api.file_path,
            api.content_type,
            api.match_headers.as_ref().map(|h| serde_json::to_string(h).unwrap_or_default()),
            api.created_at,
            api.updated_at,
        ],
    )?;
    Ok(())
}

/// 插入 API 到顶部（新创建时使用）
pub fn insert_api_at_top(conn: &Connection, api: &MockApi) -> Result<()> {
    conn.execute("UPDATE mock_apis SET sort_order = sort_order + 1", [])?;
    conn.execute(
        "INSERT INTO mock_apis (id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, match_headers, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, ?13)",
        params![
            api.id,
            api.name,
            api.method,
            api.url,
            serde_json::to_string(&api.headers)?,
            api.response_body,
            match api.response_type {
                ResponseType::Json => "json",
                ResponseType::File => "file",
            },
            api.file_name,
            api.file_path,
            api.content_type,
            api.match_headers.as_ref().map(|h| serde_json::to_string(h).unwrap_or_default()),
            api.created_at,
            api.updated_at,
        ],
    )?;
    Ok(())
}

/// 删除 API（级联删除日志）
pub fn delete_api(conn: &Connection, id: &str) -> Result<bool> {
    let rows = conn.execute("DELETE FROM mock_apis WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

/// 重新排序 API
pub fn reorder_apis(conn: &Connection, ids: &[String]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE mock_apis SET sort_order = ?1 WHERE id = ?2",
            params![i as i64, id],
        )?;
    }
    let max_order = ids.len() as i64;
    tx.execute(
        "UPDATE mock_apis SET sort_order = ?1 + rowid WHERE id NOT IN (SELECT value FROM json_each(?2))",
        params![max_order, serde_json::to_string(ids)?],
    )?;
    tx.commit()?;
    Ok(())
}

/// 添加请求日志
pub fn add_log(conn: &Connection, api_id: &str, log: &LogEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO request_logs (api_id, timestamp, method, url, headers, request_body, status_code, client_ip, user_agent, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            api_id,
            log.timestamp,
            log.method,
            log.url,
            serde_json::to_string(&log.headers)?,
            log.request_body,
            log.status_code as i64,
            log.client_ip,
            log.user_agent,
            log.error,
        ],
    )?;

    conn.execute(
        "DELETE FROM request_logs WHERE api_id = ?1 AND id NOT IN (
            SELECT id FROM request_logs WHERE api_id = ?1 ORDER BY id DESC LIMIT 100
        )",
        params![api_id],
    )?;

    Ok(())
}

/// 查询指定 API 的日志
pub fn get_logs(conn: &Connection, api_id: &str) -> Result<Vec<LogEntry>> {
    load_logs_for_api(conn, api_id)
}

/// 清空指定 API 的日志
pub fn clear_logs(conn: &Connection, api_id: &str) -> Result<()> {
    conn.execute("DELETE FROM request_logs WHERE api_id = ?1", params![api_id])?;
    Ok(())
}

fn load_logs_for_api(conn: &Connection, api_id: &str) -> Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, method, url, headers, request_body, status_code, client_ip, user_agent, error
         FROM request_logs WHERE api_id = ?1 ORDER BY id DESC LIMIT 100"
    )?;

    let logs = stmt.query_map(params![api_id], |row| {
        let headers_str: String = row.get(3)?;
        Ok(LogEntry {
            timestamp: row.get(0)?,
            method: row.get(1)?,
            url: row.get(2)?,
            headers: serde_json::from_str(&headers_str).unwrap_or_default(),
            request_body: row.get(4)?,
            status_code: row.get::<_, i64>(5)? as u16,
            client_ip: row.get(6)?,
            user_agent: row.get(7)?,
            error: row.get(8)?,
        })
    })?;

    let mut result = Vec::new();
    for log in logs {
        result.push(log?);
    }
    Ok(result)
}
