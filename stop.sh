#!/bin/bash

# Smart Forward 简单停止脚本
# 用于快速停止 smart-forward 服务

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 停止服务
stop_service() {
    print_info "停止 Smart Forward 服务..."

    # 查找 smart-forward 进程
    local pids=$(pgrep -f "smart-forward" 2>/dev/null || true)

    if [ -z "$pids" ]; then
        print_warning "没有找到运行中的 Smart Forward 服务"
        exit 0
    fi

    print_info "找到运行中的进程: $pids"

    # 优雅停止
    kill $pids 2>/dev/null || true

    # 等待进程停止
    local count=0
    while [ $count -lt 10 ]; do
        if ! ps -p "$pids" > /dev/null 2>&1; then
            break
        fi
        sleep 1
        count=$((count + 1))
    done

    # 检查是否还在运行
    if ps -p "$pids" > /dev/null 2>&1; then
        print_warning "进程没有响应，强制停止..."
        kill -9 $pids 2>/dev/null || true
        sleep 1
    fi

    # 最终检查
    if ps -p "$pids" > /dev/null 2>&1; then
        print_error "无法停止服务，请手动检查"
        exit 1
    else
        print_success "Smart Forward 服务已停止"
    fi
}

# 主函数
main() {
    echo "========================================"
    echo "  Smart Forward 快速停止脚本"
    echo "========================================"
    echo ""

    stop_service

    print_info "使用以下命令启动服务："
    print_info "  ./start.sh  # 启动服务"
}

# 运行主函数
main "$@"
