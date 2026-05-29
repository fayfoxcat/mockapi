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
        PRAGMA foreign_keys = ON;
        "
    )?;
    info!("数据库表初始化完成");
    Ok(())
}

/// 从 JSON 文件迁移数据到 SQLite
pub fn migrate_from_json(conn: &Connection, json_path: &Path) -> Result<bool> {
    if !json_path.exists() {
        return Ok(false);
    }

    // 检查数据库是否已有数据
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
        // 插入 API
        tx.execute(
            "INSERT INTO mock_apis (id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                i as i64,
                api.created_at,
                api.updated_at,
            ],
        )?;

        // 插入日志
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

    // 重命名 JSON 文件为备份
    let backup_path = json_path.with_extension("json.bak");
    std::fs::rename(json_path, &backup_path)?;
    info!(
        "成功迁移 {} 个 API 配置到 SQLite，JSON 已备份为 {:?}",
        apis.len(),
        backup_path
    );

    Ok(true)
}

/// 加载所有 API（含日志）
pub fn load_all_apis(conn: &Connection) -> Result<Vec<MockApi>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, created_at, updated_at
         FROM mock_apis ORDER BY sort_order, created_at DESC"
    )?;

    let apis = stmt.query_map([], |row| {
        let headers_str: String = row.get(4)?;
        let response_type_str: String = row.get(6)?;
        Ok((
            row.get::<_, String>(0)?,   // id
            row.get::<_, String>(1)?,   // name
            row.get::<_, String>(2)?,   // method
            row.get::<_, String>(3)?,   // url
            headers_str,
            row.get::<_, String>(5)?,   // response_body
            response_type_str,
            row.get::<_, Option<String>>(7)?,  // file_name
            row.get::<_, Option<String>>(8)?,  // file_path
            row.get::<_, Option<String>>(9)?,  // content_type
            row.get::<_, String>(10)?, // created_at
            row.get::<_, String>(11)?, // updated_at
        ))
    })?;

    let mut result = Vec::new();
    for api_row in apis {
        let (id, name, method, url, headers_str, response_body, response_type_str, file_name, file_path, content_type, created_at, updated_at) = api_row?;

        let headers: HashMap<String, String> = serde_json::from_str(&headers_str).unwrap_or_default();
        let response_type = match response_type_str.as_str() {
            "file" => ResponseType::File,
            _ => ResponseType::Json,
        };

        // 加载该 API 的日志
        let logs = load_logs_for_api(conn, &id)?;

        result.push(MockApi {
            id,
            name,
            method,
            url,
            headers,
            response_body,
            response_type,
            file_name,
            file_path,
            content_type,
            logs,
            created_at,
            updated_at,
        });
    }

    Ok(result)
}

/// 按 URL 查询单个 API（用于请求匹配，不加载日志）
pub fn get_api_by_url(conn: &Connection, url: &str) -> Result<Option<MockApi>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, created_at, updated_at
         FROM mock_apis WHERE url = ?1 LIMIT 1"
    )?;

    let mut rows = stmt.query_map(params![url], |row| {
        let headers_str: String = row.get(4)?;
        let response_type_str: String = row.get(6)?;
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            headers_str,
            row.get::<_, String>(5)?,
            response_type_str,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, Option<String>>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, String>(11)?,
        ))
    })?;

    match rows.next() {
        Some(row) => {
            let (id, name, method, url, headers_str, response_body, response_type_str, file_name, file_path, content_type, created_at, updated_at) = row?;
            let headers: HashMap<String, String> = serde_json::from_str(&headers_str).unwrap_or_default();
            let response_type = match response_type_str.as_str() {
                "file" => ResponseType::File,
                _ => ResponseType::Json,
            };
            Ok(Some(MockApi {
                id,
                name,
                method,
                url,
                headers,
                response_body,
                response_type,
                file_name,
                file_path,
                content_type,
                logs: Vec::new(),
                created_at,
                updated_at,
            }))
        }
        None => Ok(None),
    }
}

/// 插入或更新 API
pub fn upsert_api(conn: &Connection, api: &MockApi) -> Result<()> {
    conn.execute(
        "INSERT INTO mock_apis (id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM mock_apis), ?11, ?12)
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
            api.created_at,
            api.updated_at,
        ],
    )?;
    Ok(())
}

/// 插入 API 到顶部（新创建时使用）
pub fn insert_api_at_top(conn: &Connection, api: &MockApi) -> Result<()> {
    // 先将所有 sort_order +1
    conn.execute("UPDATE mock_apis SET sort_order = sort_order + 1", [])?;
    // 插入新 API，sort_order = 0
    conn.execute(
        "INSERT INTO mock_apis (id, name, method, url, headers, response_body, response_type, file_name, file_path, content_type, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11, ?12)",
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
    // 未在列表中的放到末尾
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

    // 保持每个 API 最多 100 条日志
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

/// 内部函数：加载指定 API 的日志
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
