# MockAPI - 高性能API模拟服务器

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)](https://github.com/your-repo/mockapi)

MockAPI 是一个轻量级、高性能的API模拟服务器，专为开发和测试环境设计。基于 Rust + Axum 构建，支持动态API管理、请求日志记录、文件响应等功能。

## ✨ 特性

- 🚀 **高性能**: 基于 Rust + Axum 构建，支持高并发请求
- 🎯 **零依赖**: 静态链接构建，单文件部署，无需额外依赖
- 🌐 **Web管理**: 直观的Web界面，支持实时API管理和监控
- 📝 **智能日志**: 详细的请求日志记录，包含客户端IP、浏览器信息和错误追踪
- 📁 **文件响应**: 支持文件上传和文件流响应，适用于各种场景
- 🔄 **动态路由**: 支持动态添加、修改、删除API接口，无需重启
- 🎨 **多种响应**: 支持JSON和文件两种响应类型
- 📊 **数据持久化**: SQLite 数据库存储，支持 JSON 迁移
- 📋 **大内容编辑**: 支持超大 JSON 响应体编辑（50MB+）
- 🛠️ **守护进程**: 支持后台运行、启停控制和状态监控
- 🔧 **跨平台**: 支持 Linux、Windows、macOS 多平台部署

## 🚀 快速开始

### 下载预编译版本

从 [Releases](https://github.com/your-repo/mockapi/releases) 页面下载对应平台的可执行文件：

```bash
# Linux x86_64 (推荐 - 静态链接，零依赖)
wget https://github.com/your-repo/mockapi/releases/latest/download/mockapi-linux-amd64
chmod +x mockapi-linux-amd64

# Linux ARM64
wget https://github.com/your-repo/mockapi/releases/latest/download/mockapi-linux-arm64
chmod +x mockapi-linux-arm64

# Windows
# 下载 mockapi-windows-amd64.exe
```

### 启动服务器

```bash
# 前台运行（默认端口 8344）
./mockapi-linux-amd64

# 指定端口和IP
./mockapi-linux-amd64 --port 8080 --host 127.0.0.1

# 后台运行（守护进程模式）
./mockapi-linux-amd64 --daemon

# 查看帮助
./mockapi-linux-amd64 --help
```

### 访问Web界面

启动后访问 http://localhost:8344 即可使用Web管理界面。

## 📖 使用指南

### 命令行选项

```bash
MockAPI - 高性能API模拟服务器

Usage: mockapi [OPTIONS] [COMMAND]

Commands:
  start    启动服务器（默认命令）
  stop     停止服务器
  restart  重启服务器
  status   查看服务器状态
  version  显示版本信息
  help     显示帮助信息

Options:
  -p, --port <PORT>        指定服务端口 [default: 8344]
  -H, --host <HOST>        指定绑定IP地址 [default: 0.0.0.0]
  -d, --daemon             后台运行（守护进程模式）
      --pid-file <FILE>    PID文件路径 [default: mockapi.pid]
  -h, --help               显示帮助信息
  -V, --version            显示版本信息
```

### 服务管理

```bash
# 启动服务器
./mockapi-linux-amd64 start --port 8080 --daemon

# 查看状态
./mockapi-linux-amd64 status

# 停止服务器
./mockapi-linux-amd64 stop

# 重启服务器
./mockapi-linux-amd64 restart --port 8080
```

### API管理

#### Web界面操作
1. **添加API**: 点击"新增接口"按钮，填写接口信息
2. **编辑API**: 点击接口行的"编辑"按钮，修改配置
3. **删除API**: 点击"删除"按钮或使用批量删除功能
4. **查看日志**: 点击"日志"按钮查看详细的请求记录
5. **拖拽排序**: 使用拖拽手柄调整接口顺序
6. **搜索过滤**: 使用搜索框和方法过滤器快速定位接口

#### 响应类型

**JSON响应**
```json
{
  "code": 200,
  "data": {
    "message": "Hello World",
    "timestamp": "2024-01-01T00:00:00Z"
  },
  "success": true
}
```

**文件响应**
- 支持任意文件类型（图片、文档、视频等）
- 自动设置正确的 Content-Type
- 支持文件下载和预览
- 支持大文件流式传输

### 高级功能

#### 请求日志分析
- **实时监控**: 查看每个API的请求统计和响应时间
- **客户端信息**: 记录客户端IP、浏览器类型和版本
- **错误追踪**: 详细记录请求错误和异常信息
- **请求详情**: 完整的请求头、请求体和响应信息

#### 批量操作
- **批量删除**: 选择多个接口进行批量删除
- **批量导出**: 导出API配置用于备份或迁移
- **批量导入**: 从配置文件快速导入多个接口

## 🛠️ 从源码构建

### 构建依赖

#### 基础依赖
- **Rust**: 1.70+ (推荐使用 rustup 安装)
- **Git**: 用于克隆代码仓库

#### Linux 构建依赖
```bash
# Debian/Ubuntu
sudo apt-get update
sudo apt-get install -y build-essential musl-tools musl-dev

# CentOS/RHEL/Fedora
sudo yum groupinstall "Development Tools"
sudo yum install musl-gcc musl-devel

# 或者使用 dnf (Fedora)
sudo dnf groupinstall "Development Tools"
sudo dnf install musl-gcc musl-devel
```

#### 交叉编译依赖（可选）
```bash
# ARM64 交叉编译
sudo apt-get install gcc-aarch64-linux-gnu

# Windows 交叉编译
sudo apt-get install gcc-mingw-w64-x86-64
```

### 构建步骤

```bash
# 1. 克隆代码
git clone https://github.com/your-repo/mockapi.git
cd mockapi

# 2. 安装 Rust 目标平台
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl
rustup target add x86_64-pc-windows-gnu

# 3. 构建所有平台
chmod +x build.sh
./build.sh

# 4. 或者只构建当前平台
cargo build --release
```

### 构建产物

构建完成后，可执行文件位于 `dist/` 目录：

```
dist/
├── mockapi-linux-amd64      # Linux x86_64 (静态链接)
├── mockapi-linux-arm64      # Linux ARM64 (静态链接)
└── mockapi-windows-amd64.exe # Windows x86_64
```

## 🐳 Docker 部署

### 使用预构建镜像

```bash
# 运行容器
docker run -d \
  --name mockapi \
  -p 8344:8344 \
  -v $(pwd)/data:/app/data \
  your-registry/mockapi:latest

# 使用 docker-compose
cat > docker-compose.yml << EOF
version: '3.8'
services:
  mockapi:
    image: your-registry/mockapi:latest
    ports:
      - "8344:8344"
    volumes:
      - ./data:/app/data
    restart: unless-stopped
EOF

docker-compose up -d
```

### 构建 Docker 镜像

```dockerfile
# Dockerfile
FROM scratch
COPY dist/mockapi-linux-amd64 /mockapi
EXPOSE 8344
ENTRYPOINT ["/mockapi"]
```

```bash
# 构建镜像
docker build -t mockapi:latest .

# 运行
docker run -d -p 8344:8344 mockapi:latest
```

## 📁 项目结构

```
mockapi/
├── src/                     # 源代码
│   ├── main.rs             # 主程序入口和服务管理
│   ├── api.rs              # API处理逻辑和路由
│   ├── db.rs               # SQLite 数据库操作层
│   ├── models.rs           # 数据模型定义
│   ├── utils.rs            # 工具函数
│   └── embedded.rs         # 静态资源嵌入和服务
├── static/                 # 前端资源
│   ├── index.html          # 主页面
│   ├── css/               # 样式文件
│   │   ├── style.css      # 主样式表
│   │   └── background.png # 背景图片
│   └── js/                # JavaScript文件
│       └── app.js         # 前端应用逻辑
├── build.sh               # 多平台构建脚本
├── build.bat              # Windows构建脚本
├── Cargo.toml             # Rust项目配置
└── README.md              # 项目文档
```

## 🔧 配置说明

### 数据存储

MockAPI 使用 SQLite 数据库存储所有 API 配置和请求日志。首次启动时会自动将 JSON 文件迁移至 SQLite。

```
data/
├── mockapi.db            # SQLite 数据库（API配置 + 请求日志）
└── uploads/              # 上传的文件
    ├── uuid1.jpg         # 上传的图片文件
    ├── uuid2.pdf         # 上传的PDF文件
    └── ...
```

### 环境变量

```bash
# 日志级别
export RUST_LOG=info
```

## 🚀 性能特性

### 技术优势
- **零拷贝**: 使用 Rust 的零拷贝特性，减少内存分配
- **异步IO**: 基于 Tokio 异步运行时，支持高并发
- **静态链接**: musl 静态链接，启动速度快，内存占用低
- **内存安全**: Rust 语言保证内存安全，避免常见的安全漏洞

### 性能指标

在标准测试环境下的性能表现：

```bash
# 使用 wrk 进行压力测试
wrk -t12 -c400 -d30s http://localhost:8344/api/test

# 典型结果（在 4核8GB 服务器上）
# Requests/sec: 50000+
# Latency: < 1ms (p99)
# Memory usage: < 10MB
```

## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 开发环境

```bash
# 安装开发依赖
cargo install cargo-watch cargo-edit

# 开发模式运行
cargo watch -x run

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test
```

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 编写单元测试和集成测试
- 更新相关文档

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Axum](https://github.com/tokio-rs/axum) - 现代化的 Rust Web 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [Serde](https://serde.rs/) - 序列化框架
- [Clap](https://clap.rs/) - 命令行参数解析
- [Rusqlite](https://github.com/rusqlite/rusqlite) - SQLite 绑定

## 📞 支持

- 📧 邮箱: support@mockapi.dev
- 🐛 问题反馈: [GitHub Issues](https://github.com/your-repo/mockapi/issues)
- 💬 讨论: [GitHub Discussions](https://github.com/your-repo/mockapi/discussions)
- 📖 文档: [在线文档](https://mockapi.dev/docs)

## 🔄 更新日志

### v1.0.0 (2024-01-28)
- ✨ 初始版本发布
- 🚀 支持基本的API模拟功能
- 🌐 Web管理界面
- 📝 请求日志记录
- 📁 文件响应支持
- 🛠️ 守护进程模式
- 🔧 多平台支持

### v1.1.0 (2025-05-29)
- 📊 数据存储从 JSON 迁移至 SQLite（自动迁移，无感切换）
- 📋 支持超大 JSON 响应体编辑（50MB body limit）
- 🔧 修复复制 CURL 命令使用实际访问 IP 而非 localhost
- 🛡️ 前端增加 XSS 防护（HTML 转义）
- ⚡ 大内容 textarea 使用 JS 赋值，避免页面卡死

---

**MockAPI** - 让API模拟变得简单高效！ 🚀