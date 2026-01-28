#!/bin/bash

# Mock API Server 构建脚本 - 优化版本
# 支持多平台交叉编译，优化构建速度和文件大小

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 项目信息
PROJECT_NAME="mockapi"
VERSION="1.0.0"
DIST_DIR="dist"

# 构建优化配置
export CARGO_INCREMENTAL=0      # 禁用增量编译以减小大小
export CARGO_NET_RETRY=10
export RUSTFLAGS="-C link-arg=-s"  # strip符号

# 并行构建数量（根据CPU核心数调整）
PARALLEL_JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "4")
export CARGO_BUILD_JOBS=$PARALLEL_JOBS

# 平台配置 - 专注于主要平台
declare -A PLATFORMS=(
    ["x86_64-unknown-linux-musl"]="linux-amd64"
    ["aarch64-unknown-linux-musl"]="linux-arm64"
    ["x86_64-pc-windows-gnu"]="windows-amd64.exe"
)

# 构建统计
SUCCESS_COUNT=0
FAILED_COUNT=0
BUILD_TIMES=()

print_header() {
    echo -e "${CYAN}🚀 开始构建 ${PROJECT_NAME} v${VERSION}${NC}"
    echo -e "${BLUE}📅 构建时间: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    
    if command -v git >/dev/null 2>&1 && [ -d .git ]; then
        local git_commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        echo -e "${BLUE}🔗 Git提交: ${git_commit}${NC}"
    fi
    echo ""
}

check_dependencies() {
    echo -e "${YELLOW}🔍 检查构建依赖...${NC}"
    
    # 检查基本工具
    if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}❌ 缺少必要工具: rustc/cargo${NC}"
        echo -e "${YELLOW}请先安装 Rust: https://rustup.rs/${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}🔧 Rust版本: $(rustc --version)${NC}"
    echo -e "${BLUE}📦 Cargo版本: $(cargo --version)${NC}"
    echo -e "${BLUE}⚡ 并行任务数: ${PARALLEL_JOBS}${NC}"
    
    # 检查并安装目标平台
    for target in "${!PLATFORMS[@]}"; do
        if rustup target list --installed | grep -q "^${target}$"; then
            echo -e "  ${GREEN}✅ ${target}: 已安装${NC}"
        else
            echo -e "  ${YELLOW}⚠️  ${target}: 未安装，正在安装...${NC}"
            if rustup target add "$target"; then
                echo -e "  ${GREEN}✅ ${target}: 安装成功${NC}"
            else
                echo -e "  ${RED}❌ ${target}: 安装失败${NC}"
                ((FAILED_COUNT++))
            fi
        fi
    done
    echo ""
}

setup_build_env() {
    echo -e "${YELLOW}🏗️  准备构建环境...${NC}"
    
    # 创建输出目录
    mkdir -p "$DIST_DIR"
    
    # 清理之前的构建产物
    if [ "$1" = "--clean" ]; then
        echo -e "${YELLOW}🧹 清理之前的构建...${NC}"
        cargo clean
        rm -rf "$DIST_DIR"/*
    fi
    
    # 预构建依赖
    echo -e "${BLUE}📦 预构建依赖...${NC}"
    cargo fetch --quiet
    echo ""
}

build_target() {
    local target="$1"
    local output_name="$2"
    local start_time=$(date +%s)
    
    echo -e "${CYAN}=== 构建 ${target} ===${NC}"
    
    # 设置特定目标的环境变量
    case "$target" in
        "x86_64-unknown-linux-musl")
            if command -v musl-gcc >/dev/null 2>&1; then
                export CC=musl-gcc
                export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
            else
                echo -e "  ${YELLOW}⚠️  musl-gcc 未找到，使用默认链接器${NC}"
            fi
            ;;
        "aarch64-unknown-linux-musl")
            if command -v aarch64-linux-musl-gcc >/dev/null 2>&1; then
                export CC=aarch64-linux-musl-gcc
                export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
            elif command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
                export CC=aarch64-linux-gnu-gcc
                export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
            else
                echo -e "  ${YELLOW}⚠️  ARM64 交叉编译器未找到，使用默认链接器${NC}"
            fi
            ;;
    esac
    
    local binary_name="${PROJECT_NAME}"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="${PROJECT_NAME}.exe"
    fi
    
    local source_path="target/${target}/release/${binary_name}"
    local dest_path="${DIST_DIR}/${PROJECT_NAME}-${output_name}"
    
    echo -e "  ${BLUE}🏗️  编译中...${NC}"
    
    if cargo build --release --target="$target"; then
        if [ -f "$source_path" ]; then
            cp "$source_path" "$dest_path"
            
            # 进一步压缩（如果可用）
            if command -v upx >/dev/null 2>&1 && [[ "$target" != *"musl"* ]]; then
                echo -e "  ${BLUE}📦 使用UPX压缩...${NC}"
                if upx --best --lzma "$dest_path" 2>/dev/null; then
                    echo -e "  ${GREEN}✅ UPX压缩成功${NC}"
                else
                    echo -e "  ${YELLOW}⚠️  UPX压缩失败，跳过${NC}"
                fi
            fi
            
            local file_size=$(get_file_size "$dest_path")
            echo -e "  ${GREEN}✅ 构建成功: ${dest_path}${NC}"
            echo -e "  ${BLUE}📦 文件大小: ${file_size}${NC}"
            ((SUCCESS_COUNT++))
            
            local end_time=$(date +%s)
            local build_time=$((end_time - start_time))
            BUILD_TIMES+=("${target}:${build_time}s")
        else
            echo -e "  ${RED}❌ 构建产物未找到: ${source_path}${NC}"
            ((FAILED_COUNT++))
        fi
    else
        echo -e "  ${RED}❌ 编译失败${NC}"
        ((FAILED_COUNT++))
    fi
    
    # 清理环境变量
    unset CC CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER
    echo ""
}

get_file_size() {
    local file="$1"
    if [ -f "$file" ]; then
        local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
        if [ "$size" -ge 1048576 ]; then
            # 使用shell算术而不是bc
            local mb=$((size / 1048576))
            echo "${mb}M"
        elif [ "$size" -ge 1024 ]; then
            local kb=$((size / 1024))
            echo "${kb}K"
        else
            echo "${size}B"
        fi
    else
        echo "0B"
    fi
}

print_summary() {
    echo ""
    echo -e "${CYAN}📊 构建结果摘要:${NC}"
    
    for target in "${!PLATFORMS[@]}"; do
        local output_name="${PLATFORMS[$target]}"
        local dest_path="${DIST_DIR}/${PROJECT_NAME}-${output_name}"
        if [ -f "$dest_path" ]; then
            local size=$(get_file_size "$dest_path")
            printf "  ${GREEN}✅ %-35s %8s${NC}\n" "${PROJECT_NAME}-${output_name}" "$size"
        else
            echo -e "  ${RED}❌ ${PROJECT_NAME}-${output_name}: 构建失败${NC}"
        fi
    done
    
    echo ""
    echo -e "${CYAN}📈 构建统计:${NC}"
    echo -e "   ${GREEN}✅ 成功: ${SUCCESS_COUNT}${NC}"
    echo -e "   ${RED}❌ 失败: ${FAILED_COUNT}${NC}"
    
    if [ ${#BUILD_TIMES[@]} -gt 0 ]; then
        echo ""
        echo -e "${CYAN}⏱️  构建时间:${NC}"
        for time_info in "${BUILD_TIMES[@]}"; do
            local target="${time_info%%:*}"
            local time="${time_info##*:}"
            echo -e "   ${BLUE}${target}: ${time}${NC}"
        done
    fi
    
    if [ $SUCCESS_COUNT -gt 0 ]; then
        echo ""
        echo -e "${GREEN}🎉 构建完成！可执行文件位于 ${DIST_DIR}/ 目录${NC}"
        echo -e "${BLUE}💡 提示: 使用 --clean 参数可以清理构建缓存${NC}"
    fi
}

# 主函数
main() {
    print_header
    check_dependencies
    setup_build_env "$@"
    
    echo -e "${YELLOW}🏗️  开始多平台构建...${NC}"
    echo ""
    
    for target in "${!PLATFORMS[@]}"; do
        build_target "$target" "${PLATFORMS[$target]}"
    done
    
    print_summary
    
    if [ $FAILED_COUNT -eq 0 ]; then
        echo ""
        echo -e "${GREEN}🚀 所有构建成功完成！${NC}"
        exit 0
    else
        echo ""
        echo -e "${YELLOW}⚠️  部分构建失败，但有 ${SUCCESS_COUNT} 个成功的构建${NC}"
        exit 0
    fi
}

# 运行主函数
main "$@"