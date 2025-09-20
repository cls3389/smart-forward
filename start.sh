#!/bin/bash

# Smart Forward 简单启动脚本
# 用于快速启动 smart-forward 服务

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

# 检查配置文件
check_config() {
    if [ ! -f "config.yaml" ]; then
        print_error "配置文件 config.yaml 不存在"
        print_info "请创建 config.yaml 配置文件"
        print_info "参考 config.yaml.example 创建配置"
        exit 1
    fi
}

# 启动服务
start_service() {
    print_info "启动 Smart Forward 服务..."

    # 检查是否已经在运行
    if pgrep -f "smart-forward" > /dev/null; then
        print_warning "Smart Forward 已经在运行中"
        exit 0
    fi

    # 检查配置文件
    check_config

    # 启动服务
    ./smart-forward -c config.yaml &
    local pid=$!

    print_success "Smart Forward 已启动，PID: $pid"

    # 等待几秒后检查状态
    sleep 2
    if ps -p $pid > /dev/null; then
        print_success "服务启动成功"
        print_info "使用以下命令查看状态："
        print_info "  ps aux | grep smart-forward"
        print_info "  ./stop.sh  # 停止服务"
    else
        print_error "服务启动失败，请检查日志"
        exit 1
    fi
}

# 主函数
main() {
    echo "========================================"
    echo "  Smart Forward 快速启动脚本"
    echo "========================================"
    echo ""

    start_service
}

# 运行主函数
main "$@"
