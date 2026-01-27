# Mock API Server 快速开始指南

## 🚀 快速构建

```bash
# 1. 确保在 rust 分支
git branch --show-current  # 应该显示 rust

# 2. 运行构建脚本
chmod +x build.sh
./build.sh

# 3. 查看构建产物
ls -lh dist/
```

## 📦 构建产物

构建成功后，`dist/` 目录包含：

```
dist/
├── mock-api-server-linux-amd64       # Linux 64位
├── mock-api-server-linux-arm64       # Linux ARM64
└── mock-api-server-windows-amd64.exe # Windows 64位
```

## 🎯 运行服务器

### Linux / macOS
```bash
# 使用默认端口 8344
./mock-api-server

# 指定端口
./mock-api-server --port 3000

# 查看帮助
./mock-api-server --help

# 查看版本
./mock-api-server --version
```

### Windows
```cmd
REM 使用默认端口
mock-api-server-windows-amd64.exe

REM 指定端口
mock-api-server-windows-amd64.exe --port 3000
```

## 🌐 访问 Web 界面

启动服务器后，在浏览器中打开：

```
http://localhost:8344
```

## 🔧 构建环境要求

### 必需
- Rust 1.92.0 或更高版本
- Cargo 1.92.0 或更高版本

### Linux 交叉编译工具 (自动安装)
- `mingw-w64` - Windows 交叉编译
- `gcc-aarch64-linux-gnu` - ARM64 Linux 交叉编译

### macOS 交叉编译 (可选)
- `cargo-zigbuild` + macOS SDK
- 或在 macOS 系统上直接构建

## 📋 构建脚本功能

`build.sh` 脚本会自动：

1. ✅ 检测缺失的交叉编译工具链
2. ✅ 自动安装缺失的工具 (需要 sudo 权限)
3. ✅ 安装 Rust 目标平台
4. ✅ 编译所有支持的平台
5. ✅ 显示详细的构建日志
6. ✅ 生成构建统计报告

## 🛠️ 手动构建特定平台

```bash
# Linux 64位
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Windows 64位
cargo build --release --target x86_64-pc-windows-gnu

# macOS x86 64 (需要 SDK)
cargo zigbuild --release --target x86_64-apple-darwin

# macOS ARM 64 (需要 SDK)
cargo zigbuild --release --target aarch64-apple-darwin
```

构建产物位于：`target/<target>/release/mock-api-server[.exe]`

## 🔍 验证构建

```bash
# 检查文件类型
file dist/mock-api-server-*

# 检查文件大小
ls -lh dist/

# 测试可执行文件
./mock-api-server --version
```

## 📖 API 功能

Mock API Server 提供以下功能：

1. **创建 Mock API** - 快速创建测试接口
2. **管理接口** - 编辑、删除、排序接口
3. **请求日志** - 查看接口调用历史
4. **自定义响应** - 设置响应头和响应体
5. **方法支持** - GET、POST、PUT、DELETE 等

## 🐛 常见问题

### Q: 构建失败怎么办？
A: 检查错误信息，通常是缺少交叉编译工具。运行 `./build.sh` 会自动安装。

### Q: macOS 版本构建失败？
A: 需要 macOS SDK。参考 `BUILD_STATUS.md` 中的解决方案。

### Q: 如何只构建 Linux 版本？
A: 使用 `cargo build --release --target x86_64-unknown-linux-gnu`

### Q: 端口被占用怎么办？
A: 使用 `--port` 参数指定其他端口，如 `./mock-api-server --port 3000`

## 📚 更多文档

- `BUILD_STATUS.md` - 详细的构建状态报告
- `COMPLETION_REPORT.md` - 完整的项目完善报告
- `完成报告.md` - 中文完成报告
- `README.md` - 项目说明文档

## 🎉 开始使用

```bash
# 1. 构建
./build.sh

# 2. 运行
./mock-api-server

# 3. 访问
# 浏览器打开 http://localhost:8344

# 4. 创建你的第一个 Mock API！
```

---

**提示**: 首次构建可能需要 5-10 分钟下载依赖和编译。后续构建会更快。
