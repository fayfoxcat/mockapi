#!/bin/bash

# Mock API Server Rust版本构建脚本
# 支持多平台交叉编译

set -e

APP_NAME="mock-api-server"
VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
BUILD_TIME=$(date '+%Y-%m-%d %H:%M:%S')
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

echo "🚀 开始构建 Mock API Server Rust版本 v${VERSION}"
echo "📅 构建时间: ${BUILD_TIME}"
echo "🔗 Git提交: ${GIT_COMMIT}"
echo ""

# 创建构建目录
mkdir -p dist

# 构建函数
build_for_target() {
    local TARGET=$1
    local OUTPUT_NAME=$2
    
    echo "🔨 构建 ${TARGET}..."
    
    # 检查目标是否已安装
    if ! rustup target list --installed | grep -q "^${TARGET}$"; then
        echo "📦 安装目标平台: ${TARGET}"
        rustup target add ${TARGET}
    fi
    
    # 构建
    cargo build --release --target ${TARGET}
    
    if [ $? -eq 0 ]; then
        # 复制到dist目录
        if [[ "$TARGET" == *"windows"* ]]; then
            cp "target/${TARGET}/release/${APP_NAME}.exe" "dist/${OUTPUT_NAME}.exe"
            echo "✅ dist/${OUTPUT_NAME}.exe 构建成功"
        else
            cp "target/${TARGET}/release/${APP_NAME}" "dist/${OUTPUT_NAME}"
            echo "✅ dist/${OUTPUT_NAME} 构建成功"
        fi
        
        # 显示文件大小
        if command -v ls >/dev/null 2>&1; then
            if [[ "$TARGET" == *"windows"* ]]; then
                SIZE=$(ls -lh "dist/${OUTPUT_NAME}.exe" | awk '{print $5}')
            else
                SIZE=$(ls -lh "dist/${OUTPUT_NAME}" | awk '{print $5}')
            fi
            echo "📦 文件大小: ${SIZE}"
        fi
    else
        echo "❌ ${OUTPUT_NAME} 构建失败"
        exit 1
    fi
    echo ""
}

# 检查Rust环境
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ 错误: 未找到Rust环境，请先安装Rust"
    echo "   访问 https://rustup.rs/ 安装Rust"
    exit 1
fi

echo "🔍 Rust版本: $(rustc --version)"
echo ""

echo "🏗️  开始多平台构建..."
echo ""

# Windows
build_for_target "x86_64-pc-windows-gnu" "${APP_NAME}-windows-amd64"
build_for_target "i686-pc-windows-gnu" "${APP_NAME}-windows-386"

# Linux
build_for_target "x86_64-unknown-linux-gnu" "${APP_NAME}-linux-amd64"
build_for_target "i686-unknown-linux-gnu" "${APP_NAME}-linux-386"
build_for_target "aarch64-unknown-linux-gnu" "${APP_NAME}-linux-arm64"

# macOS
build_for_target "x86_64-apple-darwin" "${APP_NAME}-darwin-amd64"
build_for_target "aarch64-apple-darwin" "${APP_NAME}-darwin-arm64"

# 本地构建（当前平台）
echo "🏠 构建本地版本..."
cargo build --release

if [ $? -eq 0 ]; then
    LOCAL_OUTPUT="${APP_NAME}"
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
        LOCAL_OUTPUT="${APP_NAME}.exe"
        cp "target/release/${APP_NAME}.exe" "${LOCAL_OUTPUT}"
    else
        cp "target/release/${APP_NAME}" "${LOCAL_OUTPUT}"
    fi
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