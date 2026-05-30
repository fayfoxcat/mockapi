use crate::{db, models::*, AppState};
use axum::{
    body::Body,
    extract::{ConnectInfo, Path, State},
    http::{header, HeaderMap, Method, StatusCode, Uri},
    response::Response,
};
use rust_embed::RustEmbed;
use std::{collections::HashMap, net::SocketAddr};
use tokio::fs;
use tracing::{error, info, warn};

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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
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

    // 从数据库查找匹配的API
    let matched_api = {
        let conn = state.db.lock().unwrap();
        db::get_api_by_url(&conn, path).ok().flatten()
    };

    if let Some(api) = matched_api {
        // 构建请求头 map（key 统一小写），用于 response_rules 匹配
        let mut req_headers = HashMap::new();
        for (key, value) in headers.iter() {
            if let Ok(v) = value.to_str() {
                req_headers.insert(key.to_string().to_lowercase(), v.to_string());
            }
        }
        handle_mock_request(state, addr, method, uri, headers, body, api, &req_headers).await
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
    addr: SocketAddr,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: String,
    api: MockApi,
    req_headers: &HashMap<String, String>,
) -> Response<Body> {
    info!("Mock请求: {} {} from {}", method, uri.path(), addr.ip());

    // 提取客户端信息
    let client_ip = extract_client_ip(&headers, addr);
    let user_agent = extract_user_agent(&headers);

    // 提取请求头
    let mut request_headers = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            request_headers.insert(key.to_string(), value_str.to_string());
        }
    }

    // 检查HTTP方法是否匹配
    if method.as_str() != api.method {
        warn!("方法不匹配: 期望 {}, 实际 {}", api.method, method);

        let error_msg = format!(
            r#"{{"error": "Method not allowed. Expected {}, got {}"}}"#,
            api.method, method
        );

        // 记录错误日志
        let log_entry = LogEntry::new(
            method.to_string(),
            uri.to_string(),
            request_headers,
            body,
            405,
            client_ip,
            user_agent,
            Some(format!("Method not allowed. Expected {}, got {}", api.method, method)),
        );

        // 写入数据库
        {
            let conn = state.db.lock().unwrap();
            let _ = db::add_log(&conn, &api.id, &log_entry);
        }

        return Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(error_msg))
            .unwrap();
    }

    // 记录成功日志
    let log_entry = LogEntry::new(
        method.to_string(),
        uri.to_string(),
        request_headers,
        body,
        200,
        client_ip,
        user_agent,
        None,
    );

    // 写入数据库
    {
        let conn = state.db.lock().unwrap();
        let _ = db::add_log(&conn, &api.id, &log_entry);
    }

    // 根据响应类型构建响应
    match api.response_type {
        ResponseType::Json => {
            // JSON响应
            let mut response_builder = Response::builder().status(StatusCode::OK);

            // 设置响应头
            for (key, value) in &api.headers {
                response_builder = response_builder.header(key, value);
            }

            // 如果没有设置Content-Type，默认设置为application/json
            if !api.headers.contains_key("Content-Type") && !api.headers.contains_key("content-type") {
                response_builder = response_builder.header(header::CONTENT_TYPE, "application/json");
            }

            // 根据 response_rules 匹配，无匹配使用默认 response_body
            let response_body = db::match_rule_body(&api, req_headers)
                .unwrap_or_else(|| api.response_body.clone());

            response_builder
                .body(Body::from(response_body))
                .unwrap()
        }
        ResponseType::File => {
            // 文件响应
            if let Some(file_path) = &api.file_path {
                let full_path = state.data_dir.join(file_path);

                match fs::read(&full_path).await {
                    Ok(file_data) => {
                        let mut response_builder = Response::builder().status(StatusCode::OK);

                        // 设置Content-Type
                        let content_type = api.content_type.as_deref()
                            .unwrap_or("application/octet-stream");
                        response_builder = response_builder.header(header::CONTENT_TYPE, content_type);

                        // 设置Content-Disposition，如果有文件名的话
                        if let Some(file_name) = &api.file_name {
                            let disposition = format!("attachment; filename=\"{}\"", file_name);
                            response_builder = response_builder.header(header::CONTENT_DISPOSITION, disposition);
                        }

                        // 设置其他自定义响应头
                        for (key, value) in &api.headers {
                            // 避免重复设置Content-Type
                            if key.to_lowercase() != "content-type" {
                                response_builder = response_builder.header(key, value);
                            }
                        }

                        response_builder
                            .body(Body::from(file_data))
                            .unwrap()
                    }
                    Err(e) => {
                        error!("读取文件失败: {} - {}", full_path.display(), e);

                        let error_msg = format!(
                            r#"{{"error": "File not found or cannot be read: {}"}}"#,
                            file_path
                        );

                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(error_msg))
                            .unwrap()
                    }
                }
            } else {
                let error_msg = r#"{"error": "File path not configured for this API"}"#;

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(error_msg))
                    .unwrap()
            }
        }
    }
}

/// 提取客户端IP地址
fn extract_client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    // 优先检查代理头
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // X-Forwarded-For 可能包含多个IP，取第一个
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }
    
    if let Some(forwarded) = headers.get("forwarded") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // 解析 Forwarded 头，格式如: for=192.0.2.60;proto=http;by=203.0.113.43
            for part in forwarded_str.split(';') {
                if let Some(for_part) = part.trim().strip_prefix("for=") {
                    return for_part.to_string();
                }
            }
        }
    }
    
    // 如果没有代理头，使用连接地址
    addr.ip().to_string()
}

/// 提取User-Agent信息
fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .unwrap_or("Unknown")
        .to_string()
}