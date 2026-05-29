use anyhow::Result;
use std::path::PathBuf;

/// 获取基础目录（可执行文件所在目录）
pub fn get_base_dir() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let base_dir = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?
        .to_path_buf();
    Ok(base_dir)
}
