# Mock API Server 🚀

一个功能简单、易于使用的MockAPI工具，支持快速创建、管理和测试API接口。

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
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
- **高性能** - 异步处理，支持高并发
- **内存安全** - 现代化的内存管理
- **端口配置** - 灵活的端口配置选项
- **日志系统** - 完整的应用和请求日志记录

## 📦 快速开始

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

# 编译
cargo build --release

# 可执行文件位于 target/release/mock-api-server
```

#### 方式三：使用构建脚本
```bash
# Linux/macOS
chmod +x build.sh
./build.sh

# Windows
build.bat
```

### 启动服务

#### 基本使用
```bash
# 默认端口8344
./mock-api-server

# 指定端口
./mock-api-server -p 9000
```

### 访问界面
启动成功后，在浏览器中访问：
```
http://localhost:8344
```

## 🎮 使用指南

### 1. 创建Mock API

1. 打开Web界面
2. 点击"新增API"按钮
3. 填写API信息：
   - **名称**: API的显示名称
   - **请求方法**: GET、POST、PUT、DELETE
   - **URL路径**: API的访问路径（如：/api/users）
   - **响应头**: 自定义HTTP响应头（可选）
   - **响应体**: 返回的数据内容

4. 点击"保存"完成创建

### 2. 管理API

- **编辑**: 点击API卡片的"编辑"按钮
- **删除**: 点击"删除"按钮或使用批量删除
- **排序**: 拖拽API卡片调整显示顺序
- **搜索**: 使用顶部搜索框快速查找
- **分页**: 底部分页控件浏览大量数据

### 3. 测试API

#### 使用CURL
每个API卡片都提供一键复制CURL命令功能：

```bash
# GET请求示例
curl -X GET "http://localhost:8344/api/users"

# POST请求示例
curl -X POST "http://localhost:8344/api/users" \
  -H "Content-Type: application/json" \
  -d '{"name":"张三","age":25}'
```

#### 使用浏览器
对于GET请求，可以直接在浏览器中访问：
```
http://localhost:8344/api/users
```

#### 使用Postman
1. 导入API地址：`http://localhost:8344`
2. 设置请求方法和路径
3. 添加请求头和请求体
4. 发送请求

### 4. 查看日志

每个API都会记录详细的请求日志：
- 请求时间
- 请求方法
- 请求URL
- 请求头信息
- 请求体内容
- 响应状态码
- 错误信息（如有）

点击API卡片的"查看日志"按钮可以查看完整的请求历史。

## 🔧 配置说明

### 命令行参数

```bash
mock-api-server [选项] [命令]

命令:
  version   显示版本信息
  help      显示帮助信息

选项:
  -p <port> 指定服务端口(默认: 8344)
  -h        显示帮助信息
  -V        显示版本信息

环境变量:
  PORT      服务端口(优先级低于 -p 参数)
```

### 目录结构

```
mock-api-server/
├── mock-api-server(.exe)    # 可执行文件
├── data/                    # 数据目录
│   └── mock_apis.json      # API配置文件
└── static/                 # 静态资源(嵌入到可执行文件中)
    ├── index.html
    ├── css/style.css
    └── js/app.js
```

### 数据格式

API配置文件`data/mock_apis.json`的格式：

```json
[
  {
    "id": "uuid-string",
    "name": "用户列表API",
    "method": "GET",
    "url": "/api/users",
    "headers": {
      "Content-Type": "application/json"
    },
    "responseBody": "[{\"id\":1,\"name\":\"张三\"}]",
    "logs": [],
    "createdAt": "2024-01-01 12:00:00",
    "updatedAt": "2024-01-01 12:00:00"
  }
]
```

## 🚀 部署指南

### Docker部署

创建`Dockerfile`：
```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /root/
COPY mock-api-server .
EXPOSE 8344
CMD ["./mock-api-server"]
```

构建和运行：
```bash
# 构建镜像
docker build -t mock-api-server .

# 运行容器
docker run -d -p 8344:8344 \
  -v $(pwd)/data:/root/data \
  --name mock-api-server \
  mock-api-server
```

### 系统服务部署

#### Linux (systemd)

创建服务文件`/etc/systemd/system/mock-api-server.service`：
```ini
[Unit]
Description=Mock API Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/mock-api-server
ExecStart=/opt/mock-api-server/mock-api-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

启用服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable mock-api-server
sudo systemctl start mock-api-server
```

#### Windows服务

使用NSSM (Non-Sucking Service Manager)：
```cmd
# 下载并安装NSSM
nssm install MockAPIServer "C:\path\to\mock-api-server.exe"
nssm set MockAPIServer AppDirectory "C:\path\to"
nssm start MockAPIServer
```

## 🔍 故障排除

### 常见问题

#### 1. 端口被占用
```bash
# 检查端口占用
netstat -tulpn | grep 8344  # Linux
netstat -ano | findstr 8344  # Windows

# 使用其他端口
./mock-api-server -p 9000
```

#### 2. 权限问题
```bash
# Linux/macOS 添加执行权限
chmod +x mock-api-server

# 检查数据目录权限
ls -la data/
```

#### 3. 数据文件损坏
```bash
# 备份现有数据
cp data/mock_apis.json data/mock_apis.json.bak

# 重置数据（清空所有API）
rm data/mock_apis.json
```

#### 4. 无法访问Web界面
- 检查防火墙设置
- 确认服务正常启动
- 尝试使用`127.0.0.1`而不是`localhost`
- 检查浏览器控制台错误信息

### 性能优化

#### 1. 大量API管理
- 使用搜索功能快速定位
- 合理使用分页功能
- 定期清理不需要的API

#### 2. 高并发场景
- 适当增加系统资源
- 监控系统负载
- 考虑使用负载均衡

## 🤝 贡献指南

### 开发环境搭建

1. **安装构建工具**：
   ```bash
   # 安装Rust (如需从源码编译)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆项目**：
   ```bash
   git clone https://github.com/your-repo/mock-api-server.git
   cd mock-api-server
   ```

3. **编译运行**：
   ```bash
   cargo run
   ```

### 代码结构

```
src/
├── main.rs          # 主程序入口
├── api.rs           # API处理器
├── models.rs        # 数据模型
├── embedded.rs      # 静态文件和动态路由
├── utils.rs         # 工具函数
static/              # 前端资源
├── index.html       # 主页面
├── css/style.css    # 样式文件
└── js/app.js        # JavaScript逻辑
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**Happy Mocking! 🎉**