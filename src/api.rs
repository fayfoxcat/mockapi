use crate::{models::*, AppState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    Json as JsonExtractor,
};
use std::collections::HashMap;
use tracing::{error, info};

/// 获取API列表
pub async fn list_apis_handler(State(state): State<AppState>) -> Json<Vec<MockApi>> {
    let apis = state.apis.read().unwrap().clone();
    Json(apis)
}

/// 保存API
pub async fn save_api_handler(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<SaveApiRequest>,
) -> Result<Json<SaveApiResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut url = req.url;
    if !url.starts_with('/') {
        url = format!("/{}", url);
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut apis = state.apis.write().unwrap();
    
    if let Some(id) = &req.id {
        // 更新现有API
        if let Some(api) = apis.iter_mut().find(|a| &a.id == id) {
            api.name = req.name;
            api.method = req.method;
            api.url = url;
            api.headers = req.headers.unwrap_or_default();
            api.response_body = req.response_body.unwrap_or_default();
            api.updated_at = now;
            
            info!("更新API: {} ({})", api.name, api.url);
            
            let response = SaveApiResponse {
                success: true,
                api: Some(api.clone()),
            };
            
            drop(apis);
            if let Err(e) = state.save_apis().await {
                error!("保存数据文件失败: {}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "保存失败".to_string() })));
            }
            
            return Ok(Json(response));
        }
    }
    
    // 创建新API
    let mut new_api = MockApi::new(req.name, req.method, url);
    new_api.headers = req.headers.unwrap_or_default();
    new_api.response_body = req.response_body.unwrap_or_default();
    
    info!("新增API: {} ({})", new_api.name, new_api.url);
    
    let response_api = new_api.clone();
    apis.insert(0, new_api);
    
    drop(apis);
    if let Err(e) = state.save_apis().await {
        error!("保存数据文件失败: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "保存失败".to_string() })));
    }
    
    Ok(Json(SaveApiResponse {
        success: true,
        api: Some(response_api),
    }))
}

/// 删除API
pub async fn delete_api_handler(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<DeleteApiRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut apis = state.apis.write().unwrap();
    
    if let Some(pos) = apis.iter().position(|a| a.id == req.id) {
        let removed = apis.remove(pos);
        info!("删除API: {} ({})", removed.name, removed.url);
    }
    
    drop(apis);
    if let Err(e) = state.save_apis().await {
        error!("保存数据文件失败: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "保存失败".to_string() })));
    }
    
    Ok(Json(SuccessResponse { success: true }))
}

/// 获取API日志
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

/// 清空API日志
pub async fn clear_logs_handler(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<ClearLogsRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut apis = state.apis.write().unwrap();
    
    if let Some(api) = apis.iter_mut().find(|a| a.id == req.id) {
        api.logs.clear();
        info!("清空日志: {}", api.name);
    }
    
    drop(apis);
    if let Err(e) = state.save_apis().await {
        error!("保存数据文件失败: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "保存失败".to_string() })));
    }
    
    Ok(Json(SuccessResponse { success: true }))
}

/// 重新排序API
pub async fn reorder_apis_handler(
    State(state): State<AppState>,
    JsonExtractor(req): JsonExtractor<ReorderRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    for (_, api) in api_map {
        apis.push(api);
    }
    
    info!("API列表重新排序");
    
    drop(apis);
    if let Err(e) = state.save_apis().await {
        error!("保存数据文件失败: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "保存失败".to_string() })));
    }
    
    Ok(Json(SuccessResponse { success: true }))
}