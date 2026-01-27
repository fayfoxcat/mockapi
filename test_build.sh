#!/bin/bash

echo "========================================="
echo "Mock API Server 构建环境检查"
echo "========================================="
echo ""

echo "1. 用户信息:"
echo "   当前用户: $(whoami)"
echo ""

echo "2. Git 信息:"
echo "   当前分支: $(git branch --show-current)"
echo "   最新提交: $(git rev-parse --short HEAD)"
echo ""

echo "3. Rust 环境:"
echo "   Rustc: $(rustc --version)"
echo "   Cargo: $(cargo --version)"
echo ""

echo "4. 已安装的 Rust 目标平台:"
rustup target list --installed | sed 's/^/   /'
echo ""

echo "5. 交叉编译工具链:"
echo "   mingw-w64 (Windows): $(command -v x86_64-w64-mingw32-gcc && echo '✅ 已安装' || echo '❌ 未安装')"
echo "   aarch64-linux-gnu (ARM64 Linux): $(command -v aarch64-linux-gnu-gcc && echo '✅ 已安装' || echo '❌ 未安装')"
echo "   cargo-zigbuild (macOS): $(command -v cargo-zigbuild && echo '✅ 已安装' || echo '❌ 未安装')"
echo ""

echo "6. 当前构建产物:"
if [ -d "dist" ] && [ -n "$(ls -A dist/*.exe dist/mock-api-server-* 2>/dev/null)" ]; then
    ls -lh dist/*.exe dist/mock-api-server-* 2>/dev/null | awk '{print "   " $9 " - " $5}'
else
    echo "   无构建产物"
fi
echo ""

echo "========================================="
echo "开始完整构建测试..."
echo "========================================="
echo ""

# 清理旧的构建产物
echo "清理旧的构建产物..."
rm -f dist/mock-api-server-*
rm -f dist/*.exe

# 运行构建脚本
./build.sh
