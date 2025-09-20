#!/bin/bash
# Smart Forward 统一安装脚本
# 支持 Linux、OpenWrt、Docker 环境自动检测和安装

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 版本信息
VERSION="v1.5.2"
GITHUB_REPO="cls3389/smart-forward"
BINARY_NAME="smart-forward"

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

# 检测系统类型
detect_system() {
    if [ -f /etc/openwrt_release ]; then
        echo "openwrt"
    elif command -v docker >/dev/null 2>&1; then
        echo "docker"
    elif [ -f /etc/os-release ]; then
        echo "linux"
    else
        echo "unknown"
    fi
}

# 检测架构
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l)
            echo "armv7"
            ;;
        *)
            print_error "不支持的架构: $arch"
            exit 1
            ;;
    esac
}

# 下载二进制文件
download_binary() {
    local system=$1
    local arch=$2
    local url=""
    
    case $system in
        openwrt|linux)
            url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/smart-forward-linux-${arch}-musl.tar.gz"
            ;;
        *)
            print_error "不支持的系统: $system"
            exit 1
            ;;
    esac
    
    print_info "下载 Smart Forward ${VERSION} for ${system}-${arch}..."
    
    if command -v wget >/dev/null 2>&1; then
        wget -O "/tmp/smart-forward.tar.gz" "$url"
    elif command -v curl >/dev/null 2>&1; then
        curl -L -o "/tmp/smart-forward.tar.gz" "$url"
    else
        print_error "需要 wget 或 curl 来下载文件"
        exit 1
    fi
    
    print_success "下载完成"
}

# 安装到 OpenWrt
install_openwrt() {
    print_info "在 OpenWrt 上安装 Smart Forward..."
    
    # 解压二进制文件
    cd /tmp
    tar -xzf smart-forward.tar.gz
    
    # 安装二进制文件
    cp smart-forward /usr/local/bin/
    chmod +x /usr/local/bin/smart-forward
    
    # 创建配置目录
    mkdir -p /etc/smart-forward
    
    # 创建默认配置文件
    if [ ! -f /etc/smart-forward/config.yaml ]; then
        cat > /etc/smart-forward/config.yaml << 'EOF'
# Smart Forward 配置文件
# 详细配置说明: https://github.com/cls3389/smart-forward

logging:
  level: "info"
  format: "text"  # OpenWrt推荐使用text格式

network:
  listen_addrs:
    - "0.0.0.0"

# 缓冲区大小 (仅用户态模式有效，内核态模式忽略)
buffer_size: 8192

# 全局动态更新配置
dynamic_update:
  check_interval: 5
  connection_timeout: 2
  auto_reconnect: true

# 转发规则
rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    buffer_size: 4096
    targets:
      - "192.168.1.100:443"  # 内网服务器
      - "example.com:443"    # 外网备用
    dynamic_update:
      check_interval: 5
      connection_timeout: 2
      auto_reconnect: true

  # 添加更多规则...
EOF
        print_info "创建了默认配置文件: /etc/smart-forward/config.yaml"
        print_warning "请编辑配置文件以适应您的需求"
    fi
    
    # 创建 procd 服务脚本
    cat > /etc/init.d/smart-forward << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10
USE_PROCD=1

PROG="/usr/local/bin/smart-forward"
CONF="/etc/smart-forward/config.yaml"
PID_FILE="/tmp/smart-forward.pid"

start_service() {
    echo "启动Smart Forward服务..."
    
    # 清理可能存在的旧PID文件
    [ -f "$PID_FILE" ] && {
        echo "清理旧的PID文件: $PID_FILE"
        rm -f "$PID_FILE"
    }
    
    # 检查配置文件
    [ ! -f "$CONF" ] && {
        echo "错误: 配置文件不存在: $CONF"
        return 1
    }
    
    # 检查二进制文件
    [ ! -x "$PROG" ] && {
        echo "错误: 二进制文件不存在或无执行权限: $PROG"
        return 1
    }
    
    # 启动服务 (自动模式，优先内核态)
    procd_open_instance
    procd_set_param command "$PROG" -c "$CONF"
    procd_set_param respawn 3600 5 5
    procd_set_param file "$CONF"
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
    
    echo "Smart Forward服务已启动"
}

stop_service() {
    echo "停止Smart Forward服务..."
    
    # 清理PID文件
    [ -f "$PID_FILE" ] && {
        echo "清理PID文件: $PID_FILE"
        rm -f "$PID_FILE"
    }
    
    echo "Smart Forward服务已停止"
}

restart() {
    stop
    sleep 2
    start
}

status() {
    echo "=== Smart Forward 状态 ==="
    local pids=$(pidof smart-forward)
    if [ -n "$pids" ]; then
        echo "✅ 正在运行, PID: $pids"
        echo "进程详情:"
        ps w | grep smart-forward | grep -v grep
        echo ""
        echo "运行模式:"
        if nft list table inet smart_forward >/dev/null 2>&1; then
            local rules=$(nft list table inet smart_forward 2>/dev/null | grep dnat | wc -l)
            echo "🚀 内核态转发 ($rules 条nftables规则)"
        else
            local ports=$(netstat -tulpn | grep smart-forward | wc -l)
            echo "👤 用户态转发 ($ports 个监听端口)"
        fi
    else
        echo "❌ 未运行"
    fi
    echo "配置文件: $CONF"
    echo "二进制文件: $PROG"
    echo "日志查看: logread | grep smart-forward"
}
EOF
    
    chmod +x /etc/init.d/smart-forward
    
    # 启用服务
    /etc/init.d/smart-forward enable
    
    print_success "OpenWrt 安装完成"
    print_info "使用方法:"
    print_info "  启动服务: /etc/init.d/smart-forward start"
    print_info "  查看状态: /etc/init.d/smart-forward status"
    print_info "  查看日志: logread | grep smart-forward"
    print_info "  编辑配置: vi /etc/smart-forward/config.yaml"
}

# 安装到 Linux
install_linux() {
    print_info "在 Linux 上安装 Smart Forward..."
    
    # 解压二进制文件
    cd /tmp
    tar -xzf smart-forward.tar.gz
    
    # 安装二进制文件
    sudo cp smart-forward /usr/local/bin/
    sudo chmod +x /usr/local/bin/smart-forward
    
    # 创建配置目录
    sudo mkdir -p /etc/smart-forward
    
    # 创建默认配置文件
    if [ ! -f /etc/smart-forward/config.yaml ]; then
        sudo tee /etc/smart-forward/config.yaml > /dev/null << 'EOF'
# Smart Forward 配置文件
# 详细配置说明: https://github.com/cls3389/smart-forward

logging:
  level: "info"
  format: "json"  # Linux推荐使用json格式

network:
  listen_addrs:
    - "0.0.0.0"

buffer_size: 8192

dynamic_update:
  check_interval: 5
  connection_timeout: 2
  auto_reconnect: true

rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    buffer_size: 4096
    targets:
      - "192.168.1.100:443"
      - "example.com:443"
    dynamic_update:
      check_interval: 5
      connection_timeout: 2
      auto_reconnect: true
EOF
        print_info "创建了默认配置文件: /etc/smart-forward/config.yaml"
        print_warning "请编辑配置文件以适应您的需求"
    fi
    
    # 创建 systemd 服务
    sudo tee /etc/systemd/system/smart-forward.service > /dev/null << 'EOF'
[Unit]
Description=Smart Forward - 智能端口转发服务
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/smart-forward -c /etc/smart-forward/config.yaml
Restart=always
RestartSec=5
User=root
Group=root

[Install]
WantedBy=multi-user.target
EOF
    
    # 启用服务
    sudo systemctl daemon-reload
    sudo systemctl enable smart-forward
    
    print_success "Linux 安装完成"
    print_info "使用方法:"
    print_info "  启动服务: sudo systemctl start smart-forward"
    print_info "  查看状态: sudo systemctl status smart-forward"
    print_info "  查看日志: sudo journalctl -u smart-forward -f"
    print_info "  编辑配置: sudo vi /etc/smart-forward/config.yaml"
}

# 显示 Docker 使用方法
show_docker_usage() {
    print_info "Docker 使用方法:"
    echo ""
    echo "1. 使用预构建镜像:"
    echo "   docker run -d --name smart-forward \\"
    echo "     --network host \\"
    echo "     --cap-add NET_ADMIN \\"
    echo "     -v /path/to/config.yaml:/etc/smart-forward/config.yaml \\"
    echo "     ghcr.io/cls3389/smart-forward:${VERSION}"
    echo ""
    echo "2. 使用 docker-compose:"
    echo "   参考项目中的 docker/docker-compose.yml"
    echo ""
    echo "3. 配置文件示例:"
    echo "   参考项目中的 config.yaml.example"
}

# 主函数
main() {
    echo "========================================"
    echo "  Smart Forward ${VERSION} 安装脚本"
    echo "  支持 Linux、OpenWrt、Docker"
    echo "========================================"
    echo ""
    
    local system=$(detect_system)
    local arch=$(detect_arch)
    
    print_info "检测到系统: $system"
    print_info "检测到架构: $arch"
    echo ""
    
    case $system in
        openwrt)
            download_binary "$system" "$arch"
            install_openwrt
            ;;
        linux)
            download_binary "$system" "$arch"
            install_linux
            ;;
        docker)
            show_docker_usage
            ;;
        *)
            print_error "不支持的系统: $system"
            print_info "支持的系统: Linux, OpenWrt, Docker"
            exit 1
            ;;
    esac
    
    echo ""
    print_success "安装完成！"
    echo ""
    print_info "特性说明:"
    print_info "  🚀 自动优先内核态转发 (Linux/OpenWrt)"
    print_info "  🔄 智能回退到用户态转发"
    print_info "  🌐 支持 IPv4/IPv6 混合网络"
    print_info "  💓 健康检查和故障转移"
    print_info "  📊 低资源占用，高性能"
    echo ""
    print_info "项目地址: https://github.com/${GITHUB_REPO}"
}

# 运行主函数
main "$@"
