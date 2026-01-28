#!/bin/bash

# MockAPI 性能测试脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 配置
MOCKAPI_BIN="./target/release/mockapi"
TEST_PORT=8345
TEST_HOST="127.0.0.1"
PID_FILE="benchmark.pid"

print_header() {
    echo -e "${CYAN}🚀 MockAPI 性能测试${NC}"
    echo -e "${BLUE}测试时间: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo ""
}

cleanup() {
    echo -e "${YELLOW}🧹 清理测试环境...${NC}"
    if [ -f "$PID_FILE" ]; then
        $MOCKAPI_BIN stop --pid-file "$PID_FILE" 2>/dev/null || true
        rm -f "$PID_FILE"
    fi
    pkill -f "mockapi.*$TEST_PORT" 2>/dev/null || true
}

start_server() {
    echo -e "${BLUE}🏗️  启动测试服务器...${NC}"
    $MOCKAPI_BIN start --port $TEST_PORT --host $TEST_HOST --daemon --pid-file "$PID_FILE"
    
    # 等待服务器启动
    echo -e "${YELLOW}⏳ 等待服务器启动...${NC}"
    for i in {1..10}; do
        if curl -s "http://$TEST_HOST:$TEST_PORT/" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ 服务器已启动${NC}"
            return 0
        fi
        sleep 1
    done
    
    echo -e "${RED}❌ 服务器启动失败${NC}"
    exit 1
}

create_test_api() {
    echo -e "${BLUE}📝 创建测试API...${NC}"
    
    # 创建一个简单的测试API
    curl -s -X POST "http://$TEST_HOST:$TEST_PORT/api/save" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "测试API",
            "method": "GET",
            "url": "/api/test",
            "headers": {"Content-Type": "application/json"},
            "responseBody": "{\"message\": \"Hello World\", \"timestamp\": \"2024-01-28T10:00:00Z\"}",
            "responseType": "json"
        }' >/dev/null
    
    echo -e "${GREEN}✅ 测试API已创建${NC}"
}

run_benchmark() {
    echo -e "${CYAN}🏃 开始性能测试...${NC}"
    echo ""
    
    # 检查是否安装了wrk
    if ! command -v wrk >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  wrk 未安装，使用 curl 进行简单测试${NC}"
        run_simple_test
        return
    fi
    
    # 使用wrk进行压力测试
    echo -e "${BLUE}📊 使用 wrk 进行压力测试...${NC}"
    echo -e "${YELLOW}测试参数: 12线程, 400连接, 30秒${NC}"
    echo ""
    
    wrk -t12 -c400 -d30s "http://$TEST_HOST:$TEST_PORT/api/test"
}

run_simple_test() {
    echo -e "${BLUE}📊 使用 curl 进行简单测试...${NC}"
    
    local total_requests=1000
    local start_time=$(date +%s.%N)
    local success_count=0
    
    echo -e "${YELLOW}发送 $total_requests 个请求...${NC}"
    
    for i in $(seq 1 $total_requests); do
        if curl -s "http://$TEST_HOST:$TEST_PORT/api/test" >/dev/null 2>&1; then
            ((success_count++))
        fi
        
        # 每100个请求显示进度
        if [ $((i % 100)) -eq 0 ]; then
            echo -ne "\r${BLUE}进度: $i/$total_requests${NC}"
        fi
    done
    
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc -l)
    local rps=$(echo "scale=2; $success_count / $duration" | bc -l)
    
    echo ""
    echo -e "${GREEN}✅ 测试完成${NC}"
    echo -e "${CYAN}📈 测试结果:${NC}"
    echo -e "   总请求数: $total_requests"
    echo -e "   成功请求: $success_count"
    echo -e "   测试时长: $(printf "%.2f" $duration) 秒"
    echo -e "   平均RPS: $rps"
}

check_memory_usage() {
    echo -e "${BLUE}💾 检查内存使用情况...${NC}"
    
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if ps -p $pid >/dev/null 2>&1; then
            local memory=$(ps -o rss= -p $pid | tr -d ' ')
            local memory_mb=$(echo "scale=2; $memory / 1024" | bc -l)
            echo -e "${CYAN}内存使用: ${memory_mb}MB${NC}"
        fi
    fi
}

show_file_size() {
    echo -e "${BLUE}📦 可执行文件大小:${NC}"
    ls -lh "$MOCKAPI_BIN" | awk '{print "   " $5 " (" $9 ")"}'
}

main() {
    # 检查依赖
    if [ ! -f "$MOCKAPI_BIN" ]; then
        echo -e "${RED}❌ 找不到可执行文件: $MOCKAPI_BIN${NC}"
        echo -e "${YELLOW}请先运行: cargo build --release${NC}"
        exit 1
    fi
    
    # 设置清理陷阱
    trap cleanup EXIT
    
    print_header
    show_file_size
    echo ""
    
    cleanup
    start_server
    create_test_api
    echo ""
    
    run_benchmark
    echo ""
    
    check_memory_usage
    echo ""
    
    echo -e "${GREEN}🎉 性能测试完成！${NC}"
}

# 运行主函数
main "$@"