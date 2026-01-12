use crate::{models::*, AppState};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, Method, StatusCode, Uri},
    response::Response,
};
use rust_embed::RustEmbed;
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

/// 服务静态文件
pub async fn serve_static(Path(path): Path<String>) -> Result<Response<Body>, StatusCode> {
    let path = path.trim_start_matches('/');
    
    if let Some(content) = StaticAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data))
            .unwrap())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// 动态路由处理器
pub async fn dynamic_handler(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: String,
) -> Response<Body> {
    let path = uri.path();
    
    // 根路径返回主页
    if path == "/" {
        if let Some(content) = StaticAssets::get("index.html") {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(content.data))
                .unwrap();
        } else {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Page not found"))
                .unwrap();
        }
    }
    
    // 查找匹配的API
    let matched_api = {
        let apis = state.apis.read().unwrap();
        apis.iter().find(|api| api.url == path).cloned()
    };
    
    if let Some(api) = matched_api {
        handle_mock_request(state, method, uri, headers, body, api).await
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
    }
}

/// 处理Mock请求
async fn handle_mock_request(
    state: AppState,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: String,
    api: MockApi,
) -> Response<Body> {
    info!("Mock请求: {} {}", method, uri.path());
    
    // 检查HTTP方法是否匹配
    if method.as_str() != api.method {
        warn!("方法不匹配: 期望 {}, 实际 {}", api.method, method);
        
        let error_msg = format!(
            r#"{{"error": "Method not allowed. Expected {}, got {}"}}"#,
            api.method, method
        );
        
        // 记录错误日志
        let mut request_headers = HashMap::new();
        for (key, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                request_headers.insert(key.to_string(), value_str.to_string());
            }
        }
        
        let log_entry = LogEntry::new(
            method.to_string(),
            uri.to_string(),
            request_headers,
            body,
            405,
            Some(format!("Method not allowed. Expected {}, got {}", api.method, method)),
        );
        
        // 更新日志
        {
            let mut apis = state.apis.write().unwrap();
            if let Some(api_mut) = apis.iter_mut().find(|a| a.id == api.id) {
                api_mut.add_log(log_entry);
            }
        }
        let _ = state.save_apis().await;
        
        return Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(error_msg))
            .unwrap();
    }
    
    // 记录成功日志
    let mut request_headers = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            request_headers.insert(key.to_string(), value_str.to_string());
        }
    }
    
    let log_entry = LogEntry::new(
        method.to_string(),
        uri.to_string(),
        request_headers,
        body,
        200,
        None,
    );
    
    // 更新日志
    {
        let mut apis = state.apis.write().unwrap();
        if let Some(api_mut) = apis.iter_mut().find(|a| a.id == api.id) {
            api_mut.add_log(log_entry);
        }
    }
    let _ = state.save_apis().await;
    
    // 构建响应
    let mut response_builder = Response::builder().status(StatusCode::OK);
    
    // 设置响应头
    for (key, value) in &api.headers {
        response_builder = response_builder.header(key, value);
    }
    
    // 如果没有设置Content-Type，默认设置为application/json
    if !api.headers.contains_key("Content-Type") && !api.headers.contains_key("content-type") {
        response_builder = response_builder.header(header::CONTENT_TYPE, "application/json");
    }
    
    response_builder
        .body(Body::from(api.response_body))
        .unwrap()
}