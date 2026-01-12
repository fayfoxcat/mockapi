use anyhow::Result;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Uri},
    response::Response,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use std::{
    net::SocketAddr,
    path::PathBuf,
    process,
    sync::{Arc, RwLock},
};
use tokio::{fs, signal};
use tower_http::cors::CorsLayer;
use tracing::info;

mod api;
mod embedded;
mod models;
mod utils;

use api::*;
use embedded::*;
use models::*;
use utils::*;

/// Mock API Server - 功能简单、易于使用的MockAPI工具
#[derive(Parser)]
#[command(name = "mock-api-server")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Mock API 管理平台")]
#[command(long_about = None)]
struct Cli {
    /// 指定服务端口
    #[arg(short, long, default_value = "8344")]
    port: u16,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示版本信息
    Version,
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub apis: Arc<RwLock<Vec<MockApi>>>,
    pub data_dir: PathBuf,
    pub data_file: PathBuf,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let base_dir = get_base_dir()?;
        let data_dir = base_dir.join("data");
        let data_file = data_dir.join("mock_apis.json");

        Ok(Self {
            apis: Arc::new(RwLock::new(Vec::new())),
            data_dir,
            data_file,
        })
    }

    pub async fn load_apis(&self) -> Result<()> {
        let content = match fs::read_to_string(&self.data_file).await {
            Ok(content) => content,
            Err(_) => {
                info!("数据文件不存在，初始化空列表");
                return Ok(());
            }
        };

        let apis: Vec<MockApi> = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("解析数据文件失败: {}", e))?;

        *self.apis.write().unwrap() = apis;
        info!("加载了 {} 个API配置", self.apis.read().unwrap().len());
        Ok(())
    }

    pub async fn save_apis(&self) -> Result<()> {
        let apis = self.apis.read().unwrap().clone();
        let content = serde_json::to_string_pretty(&apis)?;
        fs::write(&self.data_file, content).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 设置日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state = AppState::new()?;

    match cli.command {
        None => {
            // 前台运行
            run_server(cli.port, state).await
        }
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
    }
}

async fn run_server(port: u16, state: AppState) -> Result<()> {
    // 初始化目录
    init_dirs(&state).await?;

    // 打印启动信息
    print_banner();
    info!("========================================");
    info!("  {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("========================================");
    info!("  PID:      {}", process::id());
    info!("  端口:     {}", port);
    info!("  数据目录: {}", state.data_dir.display());
    info!("  数据文件: {}", state.data_file.display());
    info!("  访问地址: http://localhost:{}", port);
    info!("========================================");

    // 加载API数据
    state.load_apis().await?;

    // 创建路由
    let app = create_router(state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("服务启动成功，等待请求...");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // 优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        // 静态文件服务
        .route("/static/*path", get(serve_static))
        // API管理接口
        .route("/api/list", get(list_apis_handler))
        .route("/api/save", post(save_api_handler))
        .route("/api/delete", post(delete_api_handler))
        .route("/api/logs", get(get_logs_handler))
        .route("/api/clear-logs", post(clear_logs_handler))
        .route("/api/reorder", post(reorder_apis_handler))
        // 动态路由处理所有其他请求
        .fallback(dynamic_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn dynamic_handler(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response<Body> {
    let body_str = String::from_utf8_lossy(&body).to_string();
    embedded::dynamic_handler(State(state), method, uri, headers, body_str).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("收到关闭信号，正在优雅关闭...");
}

fn print_banner() {
    println!(
        r#"
  __  __            _       _    ____ ___ 
 |  \/  | ___   ___| | __  / \  |  _ \_ _|
 | |\/| |/ _ \ / __| |/ / / _ \ | |_) | | 
 | |  | | (_) | (__|   < / ___ \|  __/| | 
 |_|  |_|\___/ \___|_|\_\_/   \_\_|  |___|
                                          "#
    );
}

fn show_version() {
    println!("{} version {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("Build time: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
}