# Mock API Server 🚀

一个功能简单、易于使用的MockAPI工具，支持快速创建、管理和测试API接口。

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Go](https://img.shields.io/badge/go-1.19+-00ADD8.svg)
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

## 📦 快速开始

### 下载安装

#### 方式一：直接下载可执行文件
```bash
# Windows
curl -L -o mock-api-server.exe https://github.com/your-repo/mock-api-server/releases/latest/download/mock-api-server-windows.exe

# Linux
curl -L -o mock-api-server https://github.com/your-repo/mock-api-server/releases/latest/download/mock-api-server-linux
chmod +x mock-api-server

# macOS
curl -L -o mock-api-server https://github.com/your-repo/mock-api-server/releases/latest/download/mock-api-server-darwin
chmod +x mock-api-server
```

#### 方式二：从源码编译
```bash
# 克隆项目
git clone https://github.com/your-repo/mock-api-server.git
cd mock-api-server

# 编译
go build -o mock-api-server main.go

# Windows下编译
go build -o mock-api-server.exe main.go
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

### 创建Mock API

1. **点击"新增接口"按钮**
2. **填写基本信息**：
   - 服务名称：API的显示名称
   - 请求方法：GET、POST、PUT、DELETE
   - 请求URL：API的访问路径
3. **配置请求头**（JSON格式）：
   ```json
   {
     "Content-Type": "application/json",
     "Authorization": "Bearer token"
   }
   ```
4. **设置响应体**（JSON格式）：
   ```json
   {
     "code": 200,
     "data": {
       "message": "success"
     }
   }
   ```
5. **点击保存**

### 测试API

#### 方式一：使用CURL按钮
1. 点击API行中的"CURL"按钮
2. 自动复制完整的CURL命令到剪贴板
3. 在终端中粘贴执行

#### 方式二：直接请求
```bash
# GET请求示例
curl -X GET "http://localhost:8344/api/users" \
  -H "Content-Type: application/json"

# POST请求示例
curl -X POST "http://localhost:8344/api/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "张三", "age": 25}'
```

### 查看日志
1. 点击API行中的"日志"按钮
2. 查看详细的请求历史记录
3. 包含请求时间、方法、参数、响应状态等信息

## 📚 API文档

### 管理接口

#### 获取API列表
```http
GET /api/list
```

**响应示例**：
```json
[
  {
    "id": "1",
    "name": "用户信息",
    "method": "GET",
    "url": "/api/users",
    "headers": {
      "Content-Type": "application/json"
    },
    "responseBody": "{\"code\": 200, \"data\": []}",
    "logs": [],
    "createdAt": "2024-01-01 00:00:00",
    "updatedAt": "2024-01-01 00:00:00"
  }
]
```

#### 保存API配置
```http
POST /api/save
Content-Type: application/json

{
  "id": "1",
  "name": "用户信息",
  "method": "GET",
  "url": "/api/users",
  "headers": {
    "Content-Type": "application/json"
  },
  "responseBody": "{\"code\": 200, \"data\": []}"
}
```

#### 删除API
```http
POST /api/delete
Content-Type: application/json

{
  "id": "1"
}
```

#### 获取API日志
```http
GET /api/logs?id=1
```

#### 清空API日志
```http
POST /api/clear-logs
Content-Type: application/json

{
  "id": "1"
}
```

#### 重新排序
```http
POST /api/reorder
Content-Type: application/json

{
  "ids": ["3", "1", "2"]
}
```

### Mock接口

所有配置的Mock API都会根据设置的URL路径和HTTP方法提供服务。

**重要**：系统会严格验证HTTP方法，如果请求方法与配置不匹配，将返回405状态码。

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
│   ├── app_2024-01-01.log  # 应用日志
│   └── request_2024-01-01.log # 请求日志
└── static/                 # 静态资源(嵌入到可执行文件中)
    ├── index.html
    ├── css/style.css
    └── js/app.js
```

### 数据格式

API配置文件(`data/mock_apis.json`)格式：
```json
[
  {
    "id": "唯一标识符",
    "name": "API名称",
    "method": "HTTP方法",
    "url": "请求路径",
    "headers": {
      "请求头名": "请求头值"
    },
    "responseBody": "响应体内容",
    "logs": [
      {
        "timestamp": "请求时间",
        "method": "请求方法",
        "url": "请求URL",
        "headers": {},
        "requestBody": "请求体",
        "statusCode": 200,
        "error": "错误信息(如有)"
      }
    ],
    "createdAt": "创建时间",
    "updatedAt": "更新时间"
  }
]
```

## 🚀 部署指南

### Docker部署

创建`Dockerfile`：
```dockerfile
FROM golang:1.19-alpine AS builder
WORKDIR /app
COPY . .
RUN go build -o mock-api-server main.go

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/
COPY --from=builder /app/mock-api-server .
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
  -v $(pwd)/logs:/root/logs \
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

使用NSSM或类似工具将程序注册为Windows服务。

### Nginx反向代理

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:8344;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 🔍 故障排除

### 常见问题

#### 1. 端口被占用
```bash
# 查看端口占用
netstat -tulpn | grep 8344

# 或使用其他端口
./mock-api-server -p 9000
```

#### 2. 权限问题
```bash
# Linux/macOS 添加执行权限
chmod +x mock-api-server

# 确保数据目录可写
chmod 755 data logs
```

#### 3. 方法不匹配错误
确保请求的HTTP方法与API配置中的方法一致：
- 配置为POST的API不能用GET请求访问
- 系统会返回405 Method Not Allowed错误

#### 4. JSON格式错误
- 请求头和响应体必须是有效的JSON格式
- 使用界面中的"格式化"按钮检查JSON语法

### 日志查看

```bash
# 查看应用日志
tail -f logs/app_$(date +%Y-%m-%d).log

# 查看请求日志
tail -f logs/request_$(date +%Y-%m-%d).log
```

### 数据备份

```bash
# 备份配置
cp data/mock_apis.json data/mock_apis_backup_$(date +%Y%m%d).json

# 恢复配置
cp data/mock_apis_backup_20240101.json data/mock_apis.json
```

## 🤝 贡献指南

### 开发环境搭建

1. **克隆项目**：
   ```bash
   git clone https://github.com/your-repo/mock-api-server.git
   cd mock-api-server
   ```

2. **安装依赖**：
   ```bash
   go mod tidy
   ```

3. **运行开发服务器**：
   ```bash
   go run main.go
   ```

4. **构建项目**：
   ```bash
   # 本地构建
   go build -o mock-api-server main.go
   
   # 交叉编译
   GOOS=linux GOARCH=amd64 go build -o mock-api-server-linux main.go
   GOOS=windows GOARCH=amd64 go build -o mock-api-server.exe main.go
   GOOS=darwin GOARCH=amd64 go build -o mock-api-server-darwin main.go
   ```

### 代码结构

```
├── main.go              # 主程序入口
├── static/              # 前端资源
│   ├── index.html      # 主页面
│   ├── css/style.css   # 样式文件
│   └── js/app.js       # JavaScript逻辑
├── data/               # 数据存储
└── logs/               # 日志文件
``` 

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**Happy Mocking! 🎉**
