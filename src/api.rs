use crate::{models::*, AppState};
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::collections::HashMap;
use tokio::fs;
use tracing::{error, info};

/// 获取API列表
pub async fn list_apis_handler(State(state): State<AppState>) -> Json<Vec<MockApi>> {
    let apis = state.apis.read().unwrap().clone();
    Json(apis)
}

/// 保存API配置
#[axum::debug_handler]
pub async fn save_api_handler(
    State(state): State<AppState>,
    Json(req): Json<SaveApiRequest>,
) -> impl IntoResponse {
    let mut url = req.url.clone();
    if !url.starts_with('/') {
        url = format!("/{}", url);
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let response = {
        let mut apis = state.apis.write().unwrap();
        
        match req.id.as_ref() {
            Some(id) => {
                // 更新现有API或创建新API
                if let Some(api) = apis.iter_mut().find(|a| &a.id == id) {
                    update_api_fields(api, &req, &now, &url);
                    info!("更新API: {} ({})", api.name, api.url);
                    SaveApiResponse { success: true, api: Some(api.clone()) }
                } else {
                    let new_api = create_new_api(&req, url, now);
                    info!("新增API: {} ({})", new_api.name, new_api.url);
                    let response_api = new_api.clone();
                    apis.insert(0, new_api);
                    SaveApiResponse { success: true, api: Some(response_api) }
                }
            }
            None => {
                // 创建新API
                let new_api = create_new_api(&req, url, now);
                info!("新增API: {} ({})", new_api.name, new_api.url);
                let response_api = new_api.clone();
                apis.insert(0, new_api);
                SaveApiResponse { success: true, api: Some(response_api) }
            }
        }
    };
    
    if let Err(e) = state.save_apis().await {
        error!("保存数据文件失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    Json(response).into_response()
}

/// 更新API字段的辅助函数
fn update_api_fields(api: &mut MockApi, req: &SaveApiRequest, now: &str, url: &str) {
    api.name = req.name.clone();
    api.method = req.method.clone();
    api.url = url.to_string();
    api.headers = req.headers.clone().unwrap_or_default();
    api.response_body = req.response_body.clone().unwrap_or_default();
    api.response_type = req.response_type.clone().unwrap_or_default();
    api.file_name = req.file_name.clone();
    api.file_path = req.file_path.clone();
    api.content_type = req.content_type.clone();
    api.updated_at = now.to_string();
}

/// 创建新API的辅助函数
fn create_new_api(req: &SaveApiRequest, url: String, now: String) -> MockApi {
    let mut new_api = MockApi::new(req.name.clone(), req.method.clone(), url);
    new_api.headers = req.headers.clone().unwrap_or_default();
    new_api.response_body = req.response_body.clone().unwrap_or_default();
    new_api.response_type = req.response_type.clone().unwrap_or_default();
    new_api.file_name = req.file_name.clone();
    new_api.file_path = req.file_path.clone();
    new_api.content_type = req.content_type.clone();
    new_api.updated_at = now;
    new_api
}

/// 删除API配置
pub async fn delete_api_handler(
    State(state): State<AppState>,
    Json(req): Json<DeleteApiRequest>,
) -> impl IntoResponse {
    {
        let mut apis = state.apis.write().unwrap();
        
        if let Some(pos) = apis.iter().position(|a| a.id == req.id) {
            let removed = apis.remove(pos);
            info!("删除API: {} ({})", removed.name, removed.url);
        }
    }
    
    save_and_respond(&state).await
}

/// 获取API请求日志
pub async fn get_logs_handler(
    State(state): State<AppState>,
    Query(params): Query<LogsQuery>,
) -> Json<Vec<LogEntry>> {
    let apis = state.apis.read().unwrap();
    
    if let Some(api) = apis.iter().find(|a| a.id == params.id) {
        Json(api.logs.clone())
    } else {
        Json(Vec::new())
    }
}

/// 清空API请求日志
pub async fn clear_logs_handler(
    State(state): State<AppState>,
    Json(req): Json<ClearLogsRequest>,
) -> impl IntoResponse {
    {
        let mut apis = state.apis.write().unwrap();
        
        if let Some(api) = apis.iter_mut().find(|a| a.id == req.id) {
            api.logs.clear();
            info!("清空日志: {}", api.name);
        }
    }
    
    save_and_respond(&state).await
}

/// 重新排序API列表
pub async fn reorder_apis_handler(
    State(state): State<AppState>,
    Json(req): Json<ReorderRequest>,
) -> impl IntoResponse {
    {
        let mut apis = state.apis.write().unwrap();
        
        // 创建ID到API的映射
        let mut api_map: HashMap<String, MockApi> = HashMap::new();
        for api in apis.drain(..) {
            api_map.insert(api.id.clone(), api);
        }
        
        // 按新顺序重新排列
        for id in &req.ids {
            if let Some(api) = api_map.remove(id) {
                apis.push(api);
            }
        }
        
        // 添加剩余的API
        apis.extend(api_map.into_values());
        
        info!("API列表重新排序");
    }
    
    save_and_respond(&state).await
}

/// 文件上传处理器
pub async fn upload_file_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let uploads_dir = state.data_dir.join("uploads");
    
    // 确保上传目录存在
    if let Err(e) = fs::create_dir_all(&uploads_dir).await {
        error!("创建上传目录失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        let filename = field.file_name().unwrap_or("").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        if name == "file" && !filename.is_empty() {
            let data = match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    error!("读取文件数据失败: {}", e);
                    return StatusCode::BAD_REQUEST.into_response();
                }
            };

            // 生成唯一文件名
            let file_id = uuid::Uuid::new_v4().to_string();
            let file_extension = std::path::Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            let stored_filename = if file_extension.is_empty() {
                file_id
            } else {
                format!("{}.{}", file_id, file_extension)
            };
            
            let file_path = uploads_dir.join(&stored_filename);
            
            // 保存文件
            if let Err(e) = fs::write(&file_path, &data).await {
                error!("保存文件失败: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            info!("文件上传成功: {} -> {}", filename, stored_filename);

            let response = FileUploadResponse {
                success: true,
                file_name: filename,
                file_path: format!("uploads/{}", stored_filename),
                content_type,
            };

            return Json(response).into_response();
        }
    }

    error!("未找到有效的文件字段");
    StatusCode::BAD_REQUEST.into_response()
}

/// 保存数据并返回成功响应的辅助函数
async fn save_and_respond(state: &AppState) -> impl IntoResponse {
    if let Err(e) = state.save_apis().await {
        error!("保存数据文件失败: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    
    Json(SuccessResponse { success: true }).into_response()
}