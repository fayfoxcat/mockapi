use anyhow::Result;
use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{HeaderMap, Method, Uri},
    response::Response,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    process,
    sync::{Arc, RwLock},
};

#[cfg(unix)]
use std::process::Command;
use tokio::{signal, time::{sleep, Duration}};
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

mod api;
mod embedded;
mod models;
mod utils;

use api::*;
use embedded::*;
use models::*;
use utils::*;

/// Mock API Server - 高性能的API模拟服务器
#[derive(Parser)]
#[command(name = "mockapi")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Mock API 管理平台 - 轻量级API模拟服务器")]
#[command(long_about = "Mock API Server 是一个高性能的API模拟服务器，支持动态API管理、请求日志记录、文件响应等功能。")]
struct Cli {
    /// 指定服务端口
    #[arg(short, long, default_value = "8344")]
    port: u16,

    /// 指定绑定IP地址
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: IpAddr,

    /// 后台运行（守护进程模式）
    #[arg(short, long)]
    daemon: bool,

    /// PID文件路径（守护进程模式下使用）
    #[arg(long, default_value = "mockapi.pid")]
    pid_file: PathBuf,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动服务器（默认命令）
    Start {
        /// 指定服务端口
        #[arg(short, long)]
        port: Option<u16>,
        /// 指定绑定IP地址
        #[arg(short = 'H', long)]
        host: Option<IpAddr>,
        /// 后台运行
        #[arg(short, long)]
        daemon: bool,
    },
    /// 停止服务器
    Stop {
        /// PID文件路径
        #[arg(long, default_value = "mockapi.pid")]
        pid_file: PathBuf,
    },
    /// 重启服务器
    Restart {
        /// 指定服务端口
        #[arg(short, long)]
        port: Option<u16>,
        /// 指定绑定IP地址
        #[arg(short = 'H', long)]
        host: Option<IpAddr>,
        /// PID文件路径
        #[arg(long, default_value = "mockapi.pid")]
        pid_file: PathBuf,
    },
    /// 查看服务器状态
    Status {
        /// PID文件路径
        #[arg(long, default_value = "mockapi.pid")]
        pid_file: PathBuf,
    },
    /// 显示版本信息
    Version,
}

/// 应用程序状态管理
#[derive(Clone)]
pub struct AppState {
    pub apis: Arc<RwLock<Vec<MockApi>>>,
    pub data_dir: PathBuf,
    pub data_file: PathBuf,
}

impl AppState {
    /// 创建新的应用状态实例
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

    /// 从文件加载API配置
    pub async fn load_apis(&self) -> Result<()> {
        let content = match tokio::fs::read_to_string(&self.data_file).await {
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

    /// 保存API配置到文件
    pub async fn save_apis(&self) -> Result<()> {
        let apis = self.apis.read().unwrap().clone();
        let content = serde_json::to_string_pretty(&apis)?;
        tokio::fs::write(&self.data_file, content).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state = AppState::new()?;

    match cli.command {
        None => {
            // 默认启动服务器
            if cli.daemon {
                run_daemon_sync(cli.port, cli.host, cli.pid_file)?;
                Ok(())
            } else {
                run_server(cli.port, cli.host, state).await
            }
        }
        Some(Commands::Start { port, host, daemon }) => {
            let port = port.unwrap_or(cli.port);
            let host = host.unwrap_or(cli.host);
            if daemon {
                run_daemon_sync(port, host, cli.pid_file)?;
                Ok(())
            } else {
                run_server(port, host, state).await
            }
        }
        Some(Commands::Stop { pid_file }) => {
            stop_server(pid_file).await
        }
        Some(Commands::Restart { port, host, pid_file }) => {
            let port = port.unwrap_or(cli.port);
            let host = host.unwrap_or(cli.host);
            restart_server(port, host, pid_file, state).await
        }
        Some(Commands::Status { pid_file }) => {
            show_status(pid_file).await
        }
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
    }
}

/// 运行HTTP服务器
async fn run_server(port: u16, host: IpAddr, state: AppState) -> Result<()> {
    // 初始化数据目录
    init_dirs(&state).await?;

    // 显示启动信息
    print_banner();
    info!("========================================");
    info!("  {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("========================================");
    info!("  PID:      {}", process::id());
    info!("  绑定地址: {}:{}", host, port);
    info!("  数据目录: {}", state.data_dir.display());
    info!("  数据文件: {}", state.data_file.display());
    info!("  访问地址: http://{}:{}", 
          if host == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) { 
              "localhost".to_string() 
          } else { 
              host.to_string() 
          }, 
          port);
    info!("========================================");

    // 加载API数据
    state.load_apis().await?;

    // 创建路由
    let app = create_router(state);

    // 启动服务器
    let addr = SocketAddr::new(host, port);
    info!("服务启动成功，等待请求...");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // 启动服务并支持优雅关闭
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// 运行HTTP服务器（带PID文件管理）
async fn run_server_with_pid(port: u16, host: IpAddr, pid_file: PathBuf, state: AppState) -> Result<()> {
    // 写入PID文件
    let pid = process::id();
    std::fs::write(&pid_file, pid.to_string())?;
    info!("PID文件已创建: {} (PID: {})", pid_file.display(), pid);
    
    // 运行服务器
    let result = run_server(port, host, state).await;
    
    // 清理PID文件
    if pid_file.exists() {
        let _ = std::fs::remove_file(&pid_file);
    }
    
    result
}

/// 创建HTTP路由
fn create_router(state: AppState) -> Router {
    Router::new()
        // 静态资源服务
        .route("/static/*path", get(serve_static))
        // API管理接口
        .route("/api/list", get(list_apis_handler))
        .route("/api/save", post(save_api_handler))
        .route("/api/delete", post(delete_api_handler))
        .route("/api/logs", get(get_logs_handler))
        .route("/api/clear-logs", post(clear_logs_handler))
        .route("/api/reorder", post(reorder_apis_handler))
        .route("/api/upload", post(upload_file_handler))
        // 动态路由处理Mock API请求
        .fallback(dynamic_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// 动态路由处理器，处理所有Mock API请求
async fn dynamic_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response<Body> {
    let body_str = String::from_utf8_lossy(&body).to_string();
    embedded::dynamic_handler(State(state), ConnectInfo(addr), method, uri, headers, body_str).await
}

/// 优雅关闭信号处理
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

/// 显示启动横幅
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

/// 守护进程模式运行（同步版本）
fn run_daemon_sync(port: u16, host: IpAddr, pid_file: PathBuf) -> Result<()> {
    info!("启动守护进程模式...");
    
    #[cfg(unix)]
    {
        // 使用 spawn 启动新进程而不是 fork
        let current_exe = std::env::current_exe()?;
        let mut cmd = std::process::Command::new(&current_exe);
        
        // 构建启动参数（不包含 --daemon）
        cmd.arg("--port").arg(port.to_string());
        cmd.arg("--host").arg(host.to_string());
        cmd.arg("--pid-file").arg(&pid_file);
        
        // 重定向输出到日志文件
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("mockapi.log")?;
            
        cmd.stdout(log_file.try_clone()?);
        cmd.stderr(log_file);
        cmd.stdin(std::process::Stdio::null());
        
        // 启动后台进程
        let child = cmd.spawn()?;
        let pid = child.id();
        
        // 写入PID文件
        std::fs::write(&pid_file, pid.to_string())?;
        
        println!("✅ 守护进程已启动");
        println!("   PID: {}", pid);
        println!("   PID文件: {}", pid_file.display());
        println!("   日志文件: mockapi.log");
        println!("   使用 'mockapi status' 查看状态");
        
        Ok(())
    }
    
    #[cfg(not(unix))]
    {
        // Windows 系统使用服务模式
        warn!("Windows系统不支持真正的守护进程，将在前台运行");
        
        let rt = tokio::runtime::Runtime::new()?;
        let state = AppState::new()?;
        rt.block_on(run_server_with_pid(port, host, pid_file, state))
    }
}

/// 停止服务器
async fn stop_server(pid_file: PathBuf) -> Result<()> {
    if !pid_file.exists() {
        error!("PID文件不存在: {}", pid_file.display());
        return Err(anyhow::anyhow!("服务器未运行或PID文件不存在"));
    }
    
    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse()?;
    
    info!("正在停止服务器 (PID: {})...", pid);
    
    // 发送SIGTERM信号
    #[cfg(unix)]
    {
        let result = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status();
            
        match result {
            Ok(status) if status.success() => {
                info!("停止信号已发送，等待服务器关闭...");
                
                // 等待进程结束
                for i in 0..30 {
                    sleep(Duration::from_secs(1)).await;
                    
                    let check_result = Command::new("kill")
                        .arg("-0")
                        .arg(pid.to_string())
                        .status();
                        
                    if check_result.is_err() || !check_result.unwrap().success() {
                        info!("服务器已成功停止");
                        let _ = std::fs::remove_file(&pid_file);
                        return Ok(());
                    }
                    
                    if i == 29 {
                        warn!("服务器未在30秒内停止，发送强制终止信号...");
                        let _ = Command::new("kill")
                            .arg("-KILL")
                            .arg(pid.to_string())
                            .status();
                    }
                }
                
                let _ = std::fs::remove_file(&pid_file);
                Ok(())
            }
            _ => {
                error!("无法停止服务器 (PID: {})", pid);
                Err(anyhow::anyhow!("停止服务器失败"))
            }
        }
    }
    
    #[cfg(not(unix))]
    {
        error!("Windows系统暂不支持停止命令，请手动终止进程");
        Err(anyhow::anyhow!("Windows系统暂不支持停止命令"))
    }
}

/// 重启服务器
async fn restart_server(port: u16, host: IpAddr, pid_file: PathBuf, state: AppState) -> Result<()> {
    info!("正在重启服务器...");
    
    // 先尝试停止
    if pid_file.exists() {
        if let Err(e) = stop_server(pid_file.clone()).await {
            warn!("停止服务器时出错: {}", e);
        }
        
        // 等待一下确保完全停止
        sleep(Duration::from_secs(2)).await;
    }
    
    // 重新启动
    info!("正在启动服务器...");
    run_server_with_pid(port, host, pid_file, state).await
}

/// 显示服务器状态
async fn show_status(pid_file: PathBuf) -> Result<()> {
    if !pid_file.exists() {
        println!("❌ 服务器未运行 (PID文件不存在)");
        return Ok(());
    }
    
    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str.trim().parse()?;
    
    #[cfg(unix)]
    {
        let result = Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .status();
            
        match result {
            Ok(status) if status.success() => {
                println!("✅ 服务器正在运行");
                println!("   PID: {}", pid);
                println!("   PID文件: {}", pid_file.display());
                
                // 尝试获取更多信息
                if let Ok(output) = Command::new("ps")
                    .arg("-p")
                    .arg(pid.to_string())
                    .arg("-o")
                    .arg("pid,ppid,cmd,etime")
                    .output() {
                    if let Ok(info) = String::from_utf8(output.stdout) {
                        println!("   进程信息:");
                        for line in info.lines().skip(1) {
                            println!("   {}", line);
                        }
                    }
                }
            }
            _ => {
                println!("❌ 服务器未运行 (进程不存在)");
                println!("   清理过期的PID文件: {}", pid_file.display());
                let _ = std::fs::remove_file(&pid_file);
            }
        }
    }
    
    #[cfg(not(unix))]
    {
        println!("✅ PID文件存在: {} (PID: {})", pid_file.display(), pid);
        println!("   注意: Windows系统无法验证进程状态");
    }
    
    Ok(())
}

/// 显示版本信息
fn show_version() {
    println!("{} version {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("Build time: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
}