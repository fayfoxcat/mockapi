use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 响应类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Json,
    File,
}

impl Default for ResponseType {
    fn default() -> Self {
        ResponseType::Json
    }
}

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
    #[serde(rename = "responseType", default)]
    pub response_type: ResponseType,
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(rename = "filePath", skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "matchHeaders", skip_serializing_if = "Option::is_none")]
    pub match_headers: Option<HashMap<String, String>>,
    pub logs: Vec<LogEntry>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl MockApi {
    /// 创建新的Mock API实例
    pub fn new(name: String, method: String, url: String) -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            method,
            url,
            headers: HashMap::new(),
            response_body: String::new(),
            response_type: ResponseType::Json,
            file_name: None,
            file_path: None,
            content_type: None,
            match_headers: None,
            logs: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    /// 添加请求日志
    pub fn add_log(&mut self, log: LogEntry) {
        self.logs.push(log);
        // 保持最多100条日志记录
        if self.logs.len() > 100 {
            self.logs.drain(0..self.logs.len() - 100);
        }
    }
}

/// 请求日志条目
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
    #[serde(rename = "clientIp")]
    pub client_ip: String,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        request_body: String,
        status_code: u16,
        client_ip: String,
        user_agent: String,
        error: Option<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            method,
            url,
            headers,
            request_body,
            status_code,
            client_ip,
            user_agent,
            error,
        }
    }
}

// API请求和响应模型

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
    #[serde(rename = "responseType")]
    pub response_type: Option<ResponseType>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "matchHeaders")]
    pub match_headers: Option<HashMap<String, String>>,
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

/// 文件上传响应
#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub success: bool,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
}