use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Mock API 接口定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockApi {
    pub id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    #[serde(rename = "responseBody")]
    pub response_body: String,
    pub logs: Vec<LogEntry>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl MockApi {
    pub fn new(name: String, method: String, url: String) -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            method,
            url,
            headers: HashMap::new(),
            response_body: String::new(),
            logs: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    pub fn add_log(&mut self, log: LogEntry) {
        self.logs.push(log);
        // 保持最多100条日志
        if self.logs.len() > 100 {
            self.logs.drain(0..self.logs.len() - 100);
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    #[serde(rename = "requestBody")]
    pub request_body: String,
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LogEntry {
    pub fn new(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        request_body: String,
        status_code: u16,
        error: Option<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            method,
            url,
            headers,
            request_body,
            status_code,
            error,
        }
    }
}

/// API保存请求
#[derive(Debug, Deserialize)]
pub struct SaveApiRequest {
    pub id: Option<String>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "responseBody")]
    pub response_body: Option<String>,
}

/// API删除请求
#[derive(Debug, Deserialize)]
pub struct DeleteApiRequest {
    pub id: String,
}

/// 清空日志请求
#[derive(Debug, Deserialize)]
pub struct ClearLogsRequest {
    pub id: String,
}

/// 重新排序请求
#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub ids: Vec<String>,
}

/// API保存响应
#[derive(Debug, Serialize)]
pub struct SaveApiResponse {
    pub success: bool,
    pub api: Option<MockApi>,
}

/// 通用成功响应
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

/// 日志查询参数
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub id: String,
}