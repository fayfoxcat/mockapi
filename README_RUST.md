# Mock API Server (Rust版本) 🚀

一个功能简单、易于使用的MockAPI工具，支持快速创建、管理和测试API接口。本版本使用Rust重写，提供更好的性能和内存安全性。

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## ✨ 功能特性

### 🎯 核心功能
- **可视化管理界面** - 直观的Web界面，支持拖拽排序
- **多种HTTP方法** - 支持GET、POST、PUT、DELETE请求
- **严格方法验证** - 确保请求方法与配置匹配，返回正确的HTTP状态码
- **实时日志记录** - 详细记录每个API的请求历史和错误信息
- **一键CURL复制** - 自动生成完整的CURL测试命令
- **批量操作** - 支持批量删除、全选等操作

### 🛠️ 管理功能
- **拖拽排序** - 支持通过拖拽调整API显示顺序
- **搜索过滤** - 按名称、URL、请求方法快速筛选
- **分页显示** - 支持大量API数据的分页浏览
- **数据持久化** - 自动保存配置到JSON文件
- **响应体编辑** - 支持JSON格式的响应体在线编辑

### 🔧 技术特性
- **零依赖部署** - 单个可执行文件，无需额外安装
- **跨平台支持** - 支持Windows、Linux、macOS
- **后台运行** - 支持守护进程模式
- **端口配置** - 灵活的端口配置选项
- **日志系统** - 完整的应用和请求日志记录
- **内存安全** - Rust语言提供的内存安全保证
- **高性能** - 异步处理，支持高并发

## 🆚 Rust版本 vs Go版本

| 特性 | Rust版本 | Go版本 |
|------|----------|--------|
| 性能 | 更高 | 高 |
| 内存安全 | 编译时保证 | 运行时检查 |
| 并发模型 | async/await | goroutines |
| 编译速度 | 较慢 | 快 |
| 二进制大小 | 较小 | 小 |
| 生态系统 | 快速发展 | 成熟 |

## 📦 快速开始

### 环境要求

- Rust 1.70+ (用于编译)
- 或直接下载预编译的二进制文件

### 安装Rust (如需从源码编译)

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 下载安装

#### 方式一：直接下载可执行文件
```bash
# Windows
curl -L -o mock-api-server.exe https://github.com/your-repo/mock-api-server/releases/latest/download/mock-api-server-windows-amd64.exe

# Linux
curl -L -o mock-api-server https://github.com/your-repo/mock-api-server/releases/latest/download/mock-api-server-linux-amd64
chmod +x mock-api-server

# macOS
curl -L -o mock-api-server https://github.com/your-repo/mock-api-server/releases/latest/download/mock-api-server-darwin-amd64
chmod +x mock-api-server
```

#### 方式二：从源码编译
```bash
# 克隆项目
git clone https://github.com/your-repo/mock-api-server.git
cd mock-api-server

# 切换到rust分支
git checkout rust

# 编译
cargo build --release

# 可执行文件位于 target/release/mock-api-server
```

#### 方式三：使用构建脚本
```bash
# Linux/macOS
chmod +x build_rust.sh
./build_rust.sh

# Windows
build_rust.bat
```

### 启动服务

#### 前台运行（开发模式）
```bash
# 默认端口8344
./mock-api-server

# 指定端口
./mock-api-server -p 9000
```

#### 后台运行（生产模式）
```bash
# 后台启动
./mock-api-server start

# 指定端口后台启动
./mock-api-server -p 9000 start

# 查看状态
./mock-api-server status

# 停止服务
./mock-api-server stop

# 重启服务
./mock-api-server restart
```

### 访问界面
启动成功后，在浏览器中访问：
```
http://localhost:8344
```

## 🎮 使用指南

使用方法与Go版本完全相同，请参考主README.md文件中的详细说明。

## 🔧 配置说明

### 命令行参数

```bash
mock-api-server [选项] [命令]

命令:
  start     后台启动服务
  stop      停止服务
  restart   重启服务
  status    查看服务状态
  reset     重置数据(清空所有API配置)
  version   显示版本信息
  help      显示帮助信息

选项:
  -p <port> 指定服务端口(默认: 8344)

环境变量:
  PORT      服务端口(优先级低于 -p 参数)
```

### 目录结构

```
mock-api-server/
├── mock-api-server(.exe)    # 可执行文件
├── data/                    # 数据目录
│   └── mock_apis.json      # API配置文件
├── logs/                   # 日志目录
└── static/                 # 静态资源(嵌入到可执行文件中)
    ├── index.html
    ├── css/style.css
    └── js/app.js
```

## 🚀 部署指南

### Docker部署

创建`Dockerfile`：
```dockerfile
# 多阶段构建
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /root/
COPY --from=builder /app/target/release/mock-api-server .
EXPOSE 8344
CMD ["./mock-api-server"]
```

构建和运行：
```bash
# 构建镜像
docker build -t mock-api-server-rust .

# 运行容器
docker run -d -p 8344:8344 \
  -v $(pwd)/data:/root/data \
  -v $(pwd)/logs:/root/logs \
  --name mock-api-server-rust \
  mock-api-server-rust
```

### 系统服务部署

#### Linux (systemd)

创建服务文件`/etc/systemd/system/mock-api-server.service`：
```ini
[Unit]
Description=Mock API Server (Rust)
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/mock-api-server
ExecStart=/opt/mock-api-server/mock-api-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

启用服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable mock-api-server
sudo systemctl start mock-api-server
```

## 🔍 故障排除

### 常见问题

#### 1. 编译错误
```bash
# 更新Rust工具链
rustup update

# 清理构建缓存
cargo clean

# 重新构建
cargo build --release
```

#### 2. 交叉编译问题
```bash
# 安装目标平台
rustup target add x86_64-unknown-linux-gnu

# 安装交叉编译工具链
# Ubuntu/Debian
sudo apt-get install gcc-multilib

# macOS
xcode-select --install
```

#### 3. 运行时错误
```bash
# 设置日志级别
export RUST_LOG=debug
./mock-api-server

# 或者
RUST_LOG=debug ./mock-api-server
```

### 性能调优

#### 1. 编译优化
```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

#### 2. 运行时优化
```bash
# 设置环境变量
export RUST_LOG=warn  # 减少日志输出
export TOKIO_WORKER_THREADS=4  # 设置工作线程数
```

## 🤝 贡献指南

### 开发环境搭建

1. **安装Rust**：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆项目**：
   ```bash
   git clone https://github.com/your-repo/mock-api-server.git
   cd mock-api-server
   git checkout rust
   ```

3. **安装依赖**：
   ```bash
   cargo check
   ```

4. **运行开发服务器**：
   ```bash
   cargo run
   ```

5. **运行测试**：
   ```bash
   cargo test
   ```

6. **代码格式化**：
   ```bash
   cargo fmt
   ```

7. **代码检查**：
   ```bash
   cargo clippy
   ```

### 代码结构

```
src/
├── main.rs          # 主程序入口
├── api.rs           # API处理器
├── models.rs        # 数据模型
├── embedded.rs      # 静态文件和动态路由
├── daemon.rs        # 守护进程管理
├── utils.rs         # 工具函数
└── error.rs         # 错误处理
```

### 开发工具

推荐使用以下工具：
- **IDE**: VS Code + rust-analyzer插件
- **调试**: `cargo run` 或 VS Code调试器
- **性能分析**: `cargo flamegraph`
- **内存检查**: `cargo valgrind`

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**Happy Mocking with Rust! 🦀🎉**