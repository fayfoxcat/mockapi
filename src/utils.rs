use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

/// 获取基础目录（可执行文件所在目录）
pub fn get_base_dir() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let base_dir = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?
        .to_path_buf();
    Ok(base_dir)
}

/// 初始化目录
pub async fn init_dirs(state: &crate::AppState) -> Result<()> {
    fs::create_dir_all(&state.data_dir).await?;
    info!("目录初始化完成");
    Ok(())
}