#!/bin/bash

# Mock API Server 构建脚本 - 优化版本
# 支持多平台交叉编译，优化构建速度

# 注意：不使用 set -e，而是手动处理错误以避免脚本过早退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 项目信息
PROJECT_NAME="mock-api-server"
VERSION="1.0.0"
DIST_DIR="dist"

# 构建优化配置
export CARGO_INCREMENTAL=1
export CARGO_NET_RETRY=10
export RUSTC_WRAPPER=""

# 并行构建数量（根据CPU核心数调整）
PARALLEL_JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "4")
export CARGO_BUILD_JOBS=$PARALLEL_JOBS

# 平台配置
declare -A PLATFORMS=(
    ["x86_64-pc-windows-gnu"]="windows-amd64.exe"
    ["x86_64-unknown-linux-musl"]="linux-amd64"
    ["aarch64-unknown-linux-musl"]="linux-arm64"
)

# macOS平台（需要特殊处理）
MACOS_PLATFORMS=(
    "x86_64-apple-darwin:darwin-amd64"
    "aarch64-apple-darwin:darwin-arm64"
)

# 构建统计
SUCCESS_COUNT=0
FAILED_COUNT=0
SKIPPED_COUNT=0
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

print_build_info() {
    echo -e "${BLUE}🔍 构建工具版本: $(rustc --version)${NC}"
    echo -e "${BLUE}🔧 Cargo版本: $(cargo --version)${NC}"
    echo -e "${BLUE}⚡ 并行任务数: ${PARALLEL_JOBS}${NC}"
    echo ""
}

check_dependencies() {
    echo -e "${YELLOW}🔍 检查构建依赖...${NC}"
    
    # 检查基本工具
    local missing_tools=()
    
    if ! command -v rustc >/dev/null 2>&1; then
        missing_tools+=("rustc")
    fi
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing_tools+=("cargo")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo -e "${RED}❌ 缺少必要工具: ${missing_tools[*]}${NC}"
        echo -e "${YELLOW}请先安装 Rust: https://rustup.rs/${NC}"
        exit 1
    fi
    
    # 检查交叉编译工具
    echo -e "${BLUE}📋 检查交叉编译工具链...${NC}"
    
    for target in "${!PLATFORMS[@]}"; do
        if rustup target list --installed | grep -q "^${target}$"; then
            echo -e "  ${GREEN}✅ ${target}: 已安装${NC}"
        else
            echo -e "  ${YELLOW}⚠️  ${target}: 未安装，正在安装...${NC}"
            rustup target add "$target" || {
                echo -e "  ${RED}❌ ${target}: 安装失败${NC}"
                ((FAILED_COUNT++))
            }
        fi
    done
    
    # 检查macOS交叉编译
    if command -v cargo-zigbuild >/dev/null 2>&1; then
        echo -e "${GREEN}✅ cargo-zigbuild 已安装，将用于 macOS 交叉编译${NC}"
        
        for platform_info in "${MACOS_PLATFORMS[@]}"; do
            local target="${platform_info%%:*}"
            if rustup target list --installed | grep -q "^${target}$"; then
                echo -e "  ${GREEN}✅ ${target}: 已安装${NC}"
            else
                echo -e "  ${YELLOW}⚠️  ${target}: 未安装，正在安装...${NC}"
                rustup target add "$target" || {
                    echo -e "  ${RED}❌ ${target}: 安装失败${NC}"
                }
            fi
        done
    else
        echo -e "${YELLOW}⚠️  cargo-zigbuild 未安装，macOS 构建将被跳过${NC}"
        echo -e "${BLUE}💡 安装命令: cargo install cargo-zigbuild${NC}"
    fi
    
    echo ""
}

setup_build_env() {
    echo -e "${YELLOW}🏗️  准备构建环境...${NC}"
    
    # 创建输出目录
    mkdir -p "$DIST_DIR"
    
    # 清理之前的构建产物（可选）
    if [ "$1" = "--clean" ]; then
        echo -e "${YELLOW}🧹 清理之前的构建...${NC}"
        cargo clean
        rm -rf "$DIST_DIR"/*
    fi
    
    # 预构建依赖（加速后续构建）
    echo -e "${BLUE}📦 预构建依赖...${NC}"
    cargo fetch --quiet
    
    echo ""
}

build_target() {
    local target="$1"
    local output_name="$2"
    local start_time=$(date +%s)
    
    echo -e "${CYAN}=== ${target} ===${NC}"
    echo -e "${YELLOW}🔨 构建 ${target}...${NC}"
    
    # 设置特定目标的环境变量和构建选项
    local build_cmd="cargo build --release --target=${target}"
    local binary_name="${PROJECT_NAME}"
    
    # 为 musl 目标设置正确的链接器
    if [[ "$target" == "x86_64-unknown-linux-musl" ]]; then
        export CC_x86_64_unknown_linux_musl=musl-gcc
        export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
    elif [[ "$target" == "aarch64-unknown-linux-musl" ]]; then
        # 检查是否有 aarch64 musl 交叉编译工具
        if command -v aarch64-linux-musl-gcc >/dev/null 2>&1; then
            export CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
        else
            echo -e "  ${YELLOW}⚠️  aarch64-linux-musl-gcc 未找到，尝试使用系统交叉编译器${NC}"
            export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
        fi
    fi
    
    if [[ "$target" == *"windows"* ]]; then
        binary_name="${PROJECT_NAME}.exe"
    fi
    
    local source_path="target/${target}/release/${binary_name}"
    local dest_path="${DIST_DIR}/${PROJECT_NAME}-${output_name}"
    
    echo -e "  ${BLUE}🏗️  编译中...${NC}"
    
    if $build_cmd; then
        if [ -f "$source_path" ]; then
            # 尝试复制文件，如果失败则重试
            if cp "$source_path" "$dest_path" 2>/dev/null || sleep 1 && cp "$source_path" "$dest_path"; then
                local file_size=$(get_file_size "$dest_path")
                echo -e "  ${GREEN}✅ 构建成功: ${dest_path}${NC}"
                echo -e "  ${BLUE}📦 文件大小: ${file_size}${NC}"
                ((SUCCESS_COUNT++))
                
                local end_time=$(date +%s)
                local build_time=$((end_time - start_time))
                BUILD_TIMES+=("${target}:${build_time}s")
            else
                echo -e "  ${RED}❌ 文件复制失败: ${source_path} -> ${dest_path}${NC}"
                ((FAILED_COUNT++))
                return 1
            fi
        else
            echo -e "  ${RED}❌ 构建产物未找到: ${source_path}${NC}"
            ((FAILED_COUNT++))
            return 1
        fi
    else
        echo -e "  ${RED}❌ 编译失败${NC}"
        ((FAILED_COUNT++))
        return 1
    fi
    
    echo -e "${GREEN}✅ 构建完成${NC}"
    echo ""
    
    # 清理目标特定的环境变量
    if [[ "$target" == "x86_64-unknown-linux-musl" ]]; then
        unset CC_x86_64_unknown_linux_musl
        unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER
    elif [[ "$target" == "aarch64-unknown-linux-musl" ]]; then
        unset CC_aarch64_unknown_linux_musl
        unset CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER
    fi
    
    return 0
}

build_macos_target() {
    local target="$1"
    local output_name="$2"
    local start_time=$(date +%s)
    
    echo -e "${CYAN}=== ${target} ===${NC}"
    echo -e "${YELLOW}🔨 构建 ${target}...${NC}"
    
    # 检查是否有macOS SDK
    if [ -n "$SDKROOT" ] || command -v xcrun >/dev/null 2>&1; then
        echo -e "  ${BLUE}ℹ️  使用系统 macOS SDK${NC}"
        local build_cmd="cargo build --release --target=${target}"
    elif command -v cargo-zigbuild >/dev/null 2>&1; then
        echo -e "  ${BLUE}ℹ️  使用 cargo-zigbuild 进行交叉编译${NC}"
        local build_cmd="cargo zigbuild --release --target=${target}"
    else
        echo -e "  ${YELLOW}⚠️  跳过 macOS 构建：缺少 SDK 或 cargo-zigbuild${NC}"
        echo -e "  ${BLUE}💡 解决方案:${NC}"
        echo -e "     1. 安装 cargo-zigbuild: cargo install cargo-zigbuild"
        echo -e "     2. 或在 macOS 系统上构建"
        ((SKIPPED_COUNT++))
        return 0
    fi
    
    local source_path="target/${target}/release/${PROJECT_NAME}"
    local dest_path="${DIST_DIR}/${PROJECT_NAME}-${output_name}"
    
    echo -e "  ${BLUE}🏗️  编译中...${NC}"
    
    if $build_cmd 2>/dev/null; then
        if [ -f "$source_path" ]; then
            # 尝试复制文件，如果失败则重试
            if cp "$source_path" "$dest_path" 2>/dev/null || sleep 1 && cp "$source_path" "$dest_path"; then
                local file_size=$(get_file_size "$dest_path")
                echo -e "  ${GREEN}✅ 构建成功: ${dest_path}${NC}"
                echo -e "  ${BLUE}📦 文件大小: ${file_size}${NC}"
                ((SUCCESS_COUNT++))
                
                local end_time=$(date +%s)
                local build_time=$((end_time - start_time))
                BUILD_TIMES+=("${target}:${build_time}s")
            else
                echo -e "  ${RED}❌ 文件复制失败${NC}"
                ((FAILED_COUNT++))
                return 1
            fi
        else
            echo -e "  ${RED}❌ 构建产物未找到${NC}"
            ((FAILED_COUNT++))
            return 1
        fi
    else
        echo -e "  ${YELLOW}⚠️  macOS 构建失败，跳过${NC}"
        ((SKIPPED_COUNT++))
        return 0
    fi
    
    echo -e "${GREEN}✅ 构建完成${NC}"
    echo ""
    return 0
}

build_local() {
    echo -e "${CYAN}=== 本地构建 ===${NC}"
    echo -e "${YELLOW}🏠 构建本地版本...${NC}"
    
    local start_time=$(date +%s)
    
    if cargo build --release; then
        local local_binary="target/release/${PROJECT_NAME}"
        if [ -f "$local_binary" ]; then
            echo -e "${GREEN}✅ ${PROJECT_NAME} 构建成功${NC}"
            ((SUCCESS_COUNT++))
            
            local end_time=$(date +%s)
            local build_time=$((end_time - start_time))
            BUILD_TIMES+=("local:${build_time}s")
            
            return 0
        else
            echo -e "${RED}❌ 本地构建产物未找到${NC}"
            ((FAILED_COUNT++))
            return 1
        fi
    else
        echo -e "${RED}❌ 本地构建失败${NC}"
        ((FAILED_COUNT++))
        return 1
    fi
}

get_file_size() {
    local file="$1"
    if [ -f "$file" ]; then
        local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
        if [ "$size" -ge 1073741824 ]; then
            echo "$(echo "scale=1; $size/1073741824" | bc 2>/dev/null || echo "$(($size/1073741824))")G"
        elif [ "$size" -ge 1048576 ]; then
            echo "$(echo "scale=1; $size/1048576" | bc 2>/dev/null || echo "$(($size/1048576))")M"
        elif [ "$size" -ge 1024 ]; then
            echo "$(echo "scale=1; $size/1024" | bc 2>/dev/null || echo "$(($size/1024))")K"
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
    
    # 显示构建结果
    for target in "${!PLATFORMS[@]}"; do
        local output_name="${PLATFORMS[$target]}"
        local dest_path="${DIST_DIR}/${PROJECT_NAME}-${output_name}"
        if [ -f "$dest_path" ]; then
            echo -e "  ${GREEN}✅ ${PROJECT_NAME}-${output_name}: 构建成功${NC}"
        else
            echo -e "  ${RED}❌ ${PROJECT_NAME}-${output_name}: 构建失败${NC}"
        fi
    done
    
    # macOS平台
    for platform_info in "${MACOS_PLATFORMS[@]}"; do
        local output_name="${platform_info##*:}"
        local dest_path="${DIST_DIR}/${PROJECT_NAME}-${output_name}"
        if [ -f "$dest_path" ]; then
            echo -e "  ${GREEN}✅ ${PROJECT_NAME}-${output_name}: 构建成功${NC}"
        else
            echo -e "  ${YELLOW}⚠️  ${PROJECT_NAME}-${output_name}: 构建跳过${NC}"
        fi
    done
    
    # 本地构建
    if [ -f "target/release/${PROJECT_NAME}" ]; then
        echo -e "  ${GREEN}✅ ${PROJECT_NAME}: 本地构建成功${NC}"
    else
        echo -e "  ${RED}❌ ${PROJECT_NAME}: 本地构建失败${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}📈 构建统计:${NC}"
    echo -e "   ${GREEN}✅ 成功: ${SUCCESS_COUNT}${NC}"
    echo -e "   ${RED}❌ 失败: ${FAILED_COUNT}${NC}"
    echo -e "   ${YELLOW}⏭️  跳过: ${SKIPPED_COUNT}${NC}"
    
    # 显示构建时间
    if [ ${#BUILD_TIMES[@]} -gt 0 ]; then
        echo ""
        echo -e "${CYAN}⏱️  构建时间:${NC}"
        for time_info in "${BUILD_TIMES[@]}"; do
            local target="${time_info%%:*}"
            local time="${time_info##*:}"
            echo -e "   ${BLUE}${target}: ${time}${NC}"
        done
    fi
    
    # 显示可用文件
    echo ""
    echo -e "${CYAN}� 构建产物:${NC}"
    if [ -d "$DIST_DIR" ] && [ "$(ls -A $DIST_DIR 2>/dev/null)" ]; then
        for file in "$DIST_DIR"/*; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                local size=$(get_file_size "$file")
                printf "   ${GREEN}%-35s %8s${NC}\n" "$filename" "$size"
            fi
        done
        
        echo ""
        echo -e "${GREEN}🚀 本地运行:${NC}"
        echo -e "  ${BLUE}./target/release/${PROJECT_NAME}${NC}"
    else
        echo -e "   ${YELLOW}无构建产物${NC}"
    fi
}

# 主函数
main() {
    print_header
    print_build_info
    check_dependencies
    setup_build_env "$@"
    
    echo -e "${YELLOW}🏗️  开始多平台构建...${NC}"
    echo ""
    
    # 构建常规平台
    for target in "${!PLATFORMS[@]}"; do
        build_target "$target" "${PLATFORMS[$target]}" || true  # 继续构建其他平台
    done
    
    # 构建macOS平台
    for platform_info in "${MACOS_PLATFORMS[@]}"; do
        local target="${platform_info%%:*}"
        local output_name="${platform_info##*:}"
        build_macos_target "$target" "$output_name" || true  # 继续构建其他平台
    done
    
    # 本地构建
    build_local || true  # 即使本地构建失败也显示摘要
    
    # 显示摘要
    print_summary
    
    # 退出状态
    if [ $FAILED_COUNT -eq 0 ]; then
        echo ""
        echo -e "${GREEN}🎉 构建完成！${NC}"
        echo -e "${BLUE}� 更多信息请查看 README.md${NC}"
        exit 0
    else
        echo ""
        echo -e "${YELLOW}⚠️  部分构建失败，但有成功的构建产物${NC}"
        exit 0
    fi
}

# 运行主函数
main "$@"