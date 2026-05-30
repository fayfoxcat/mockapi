use crate::{db, models::*, AppState};
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tokio::fs;
use tracing::{error, info};

/// 获取API列表
pub async fn list_apis_handler(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match db::load_all_apis(&conn) {
        Ok(apis) => Json(apis).into_response(),
        Err(e) => {
            error!("加载API列表失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
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

    let conn = state.db.lock().unwrap();

    let result = match req.id.as_ref() {
        Some(id) => {
            // 尝试查找现有 API
            let existing = {
                let mut stmt = conn.prepare("SELECT id FROM mock_apis WHERE id = ?1").ok();
                stmt.as_mut()
                    .and_then(|s| s.query_row(rusqlite::params![id], |_| Ok(())).ok())
                    .is_some()
            };

            if existing {
                // 更新现有 API
                let mut api = MockApi::new(req.name.clone(), req.method.clone(), url.clone());
                api.id = id.clone();
                api.headers = req.headers.clone().unwrap_or_default();
                api.response_body = req.response_body.clone().unwrap_or_default();
                api.response_type = req.response_type.clone().unwrap_or_default();
                api.file_name = req.file_name.clone();
                api.file_path = req.file_path.clone();
                api.content_type = req.content_type.clone();
                api.match_headers = req.match_headers.clone();
                api.created_at = now.clone();
                api.updated_at = now.clone();

                match db::upsert_api(&conn, &api) {
                    Ok(_) => {
                        info!("更新API: {} ({})", api.name, api.url);
                        Ok(api)
                    }
                    Err(e) => Err(e),
                }
            } else {
                // ID 存在但未找到，创建新 API
                let mut api = MockApi::new(req.name.clone(), req.method.clone(), url);
                api.id = id.clone();
                api.headers = req.headers.clone().unwrap_or_default();
                api.response_body = req.response_body.clone().unwrap_or_default();
                api.response_type = req.response_type.clone().unwrap_or_default();
                api.file_name = req.file_name.clone();
                api.file_path = req.file_path.clone();
                api.content_type = req.content_type.clone();
                api.match_headers = req.match_headers.clone();
                api.updated_at = now;

                match db::insert_api_at_top(&conn, &api) {
                    Ok(_) => {
                        info!("新增API: {} ({})", api.name, api.url);
                        Ok(api)
                    }
                    Err(e) => Err(e),
                }
            }
        }
        None => {
            // 创建新 API
            let mut api = MockApi::new(req.name.clone(), req.method.clone(), url);
            api.headers = req.headers.clone().unwrap_or_default();
            api.response_body = req.response_body.clone().unwrap_or_default();
            api.response_type = req.response_type.clone().unwrap_or_default();
            api.file_name = req.file_name.clone();
            api.file_path = req.file_path.clone();
            api.content_type = req.content_type.clone();
            api.match_headers = req.match_headers.clone();
            api.updated_at = now;

            match db::insert_api_at_top(&conn, &api) {
                Ok(_) => {
                    info!("新增API: {} ({})", api.name, api.url);
                    Ok(api)
                }
                Err(e) => Err(e),
            }
        }
    };

    match result {
        Ok(api) => Json(SaveApiResponse {
            success: true,
            api: Some(api),
        })
        .into_response(),
        Err(e) => {
            error!("保存API失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 删除API配置
pub async fn delete_api_handler(
    State(state): State<AppState>,
    Json(req): Json<DeleteApiRequest>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match db::delete_api(&conn, &req.id) {
        Ok(true) => {
            info!("删除API: {}", req.id);
            Json(SuccessResponse { success: true }).into_response()
        }
        Ok(false) => {
            error!("未找到要删除的API: {}", req.id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            error!("删除API失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 获取API请求日志
pub async fn get_logs_handler(
    State(state): State<AppState>,
    Query(params): Query<LogsQuery>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match db::get_logs(&conn, &params.id) {
        Ok(logs) => Json(logs).into_response(),
        Err(e) => {
            error!("获取日志失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 清空API请求日志
pub async fn clear_logs_handler(
    State(state): State<AppState>,
    Json(req): Json<ClearLogsRequest>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match db::clear_logs(&conn, &req.id) {
        Ok(_) => {
            info!("清空日志: {}", req.id);
            Json(SuccessResponse { success: true }).into_response()
        }
        Err(e) => {
            error!("清空日志失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 重新排序API列表
pub async fn reorder_apis_handler(
    State(state): State<AppState>,
    Json(req): Json<ReorderRequest>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match db::reorder_apis(&conn, &req.ids) {
        Ok(_) => {
            info!("API列表重新排序");
            Json(SuccessResponse { success: true }).into_response()
        }
        Err(e) => {
            error!("重新排序失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
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
