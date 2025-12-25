#!/bin/bash

# Mock API Server 构建脚本
# 支持多平台交叉编译

set -e

APP_NAME="mock-api"
VERSION="1.0.0"
BUILD_TIME=$(date '+%Y-%m-%d %H:%M:%S')
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# 构建信息
LDFLAGS="-X 'main.Version=${VERSION}' -X 'main.BuildTime=${BUILD_TIME}' -X 'main.GitCommit=${GIT_COMMIT}' -s -w"

echo "🚀 开始构建 Mock API Server v${VERSION}"
echo "📅 构建时间: ${BUILD_TIME}"
echo "🔗 Git提交: ${GIT_COMMIT}"
echo ""

# 创建构建目录
mkdir -p dist

# 构建函数
build_for_platform() {
    local GOOS=$1
    local GOARCH=$2
    local EXT=$3
    local OUTPUT="dist/${APP_NAME}-${GOOS}-${GOARCH}${EXT}"
    
    echo "🔨 构建 ${GOOS}/${GOARCH}..."
    
    GOOS=${GOOS} GOARCH=${GOARCH} go build \
        -ldflags "${LDFLAGS}" \
        -o "${OUTPUT}" \
        main.go
    
    if [ $? -eq 0 ]; then
        echo "✅ ${OUTPUT} 构建成功"
        
        # 显示文件大小
        if command -v ls >/dev/null 2>&1; then
            SIZE=$(ls -lh "${OUTPUT}" | awk '{print $5}')
            echo "📦 文件大小: ${SIZE}"
        fi
    else
        echo "❌ ${OUTPUT} 构建失败"
        exit 1
    fi
    echo ""
}

# 检查Go环境
if ! command -v go >/dev/null 2>&1; then
    echo "❌ 错误: 未找到Go环境，请先安装Go"
    exit 1
fi

echo "🔍 Go版本: $(go version)"
echo ""

# 下载依赖
echo "📦 下载依赖..."
go mod tidy
echo ""

# 构建各平台版本
echo "🏗️  开始多平台构建..."
echo ""

# Windows
build_for_platform "windows" "amd64" ".exe"
build_for_platform "windows" "386" ".exe"

# Linux
build_for_platform "linux" "amd64" ""
build_for_platform "linux" "386" ""
build_for_platform "linux" "arm64" ""

# macOS
build_for_platform "darwin" "amd64" ""
build_for_platform "darwin" "arm64" ""

# 本地构建（当前平台）
echo "🏠 构建本地版本..."
LOCAL_OUTPUT="${APP_NAME}"
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    LOCAL_OUTPUT="${APP_NAME}.exe"
fi

go build -ldflags "${LDFLAGS}" -o "${LOCAL_OUTPUT}" main.go

if [ $? -eq 0 ]; then
    echo "✅ ${LOCAL_OUTPUT} 构建成功"
else
    echo "❌ ${LOCAL_OUTPUT} 构建失败"
    exit 1
fi

echo ""
echo "🎉 所有构建完成！"
echo ""
echo "📁 构建产物:"
ls -la dist/ 2>/dev/null || dir dist\ 2>/dev/null || echo "dist目录为空"
echo ""
echo "🚀 本地运行:"
echo "  ./${LOCAL_OUTPUT}"
echo ""
echo "📖 更多信息请查看 README.md"