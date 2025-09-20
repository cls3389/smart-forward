#!/bin/bash

# OpenWrt Smart Forward 内核态转发测试脚本
# 用于快速部署和测试v1.5.0内核态转发功能

set -e

echo "🚀 Smart Forward v1.5.0 - OpenWrt内核态转发测试"
echo "================================================="

# 配置变量
SMART_FORWARD_DIR="/usr/local/bin"
CONFIG_DIR="/etc/smart-forward"
SERVICE_FILE="/etc/init.d/smart-forward"
BINARY_NAME="smart-forward"

# 检测架构
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "x86_64-musl"
            ;;
        aarch64)
            echo "aarch64-musl"
            ;;
        mips)
            echo "mips-musl"
            ;;
        mipsel)
            echo "mipsel-musl"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# 检查权限
check_permissions() {
    if [[ $EUID -ne 0 ]]; then
        echo "❌ 需要root权限来安装和配置服务"
        echo "请使用: sudo $0"
        exit 1
    fi
}

# 检查OpenWrt环境
check_openwrt() {
    if [[ ! -f /etc/openwrt_release ]]; then
        echo "⚠️  警告: 未检测到OpenWrt环境，但继续执行..."
    else
        echo "✅ 检测到OpenWrt环境"
        source /etc/openwrt_release
        echo "   版本: $DISTRIB_DESCRIPTION"
    fi
}

# 检查防火墙后端
check_firewall() {
    echo "🔍 检查防火墙后端支持..."
    
    local has_nftables=false
    local has_iptables=false
    
    if command -v nft >/dev/null 2>&1; then
        has_nftables=true
        echo "✅ nftables 支持: $(nft --version)"
    fi
    
    if command -v iptables >/dev/null 2>&1; then
        has_iptables=true
        echo "✅ iptables 支持: $(iptables --version | head -1)"
    fi
    
    if [[ "$has_nftables" == "true" ]]; then
        echo "🎯 推荐使用: nftables (Firewall4兼容)"
        FIREWALL_BACKEND="nftables"
    elif [[ "$has_iptables" == "true" ]]; then
        echo "🎯 使用: iptables (传统模式)"
        FIREWALL_BACKEND="iptables"
    else
        echo "❌ 未找到支持的防火墙后端"
        exit 1
    fi
}

# 下载二进制文件
download_binary() {
    local arch=$(detect_arch)
    echo "📦 检测到架构: $arch"
    
    if [[ "$arch" == "unknown" ]]; then
        echo "❌ 不支持的架构: $(uname -m)"
        echo "请手动下载适合的二进制文件"
        exit 1
    fi
    
    local download_url="https://github.com/cls3389/smart-forward/releases/download/v1.5.0/smart-forward-linux-${arch}.tar.gz"
    
    echo "📥 下载二进制文件..."
    echo "   URL: $download_url"
    
    # 创建临时目录
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # 下载并解压
    if command -v wget >/dev/null 2>&1; then
        wget -O smart-forward.tar.gz "$download_url"
    elif command -v curl >/dev/null 2>&1; then
        curl -L -o smart-forward.tar.gz "$download_url"
    else
        echo "❌ 需要wget或curl来下载文件"
        exit 1
    fi
    
    tar -xzf smart-forward.tar.gz
    
    # 安装二进制文件
    echo "📦 安装二进制文件到 $SMART_FORWARD_DIR"
    mkdir -p "$SMART_FORWARD_DIR"
    cp smart-forward "$SMART_FORWARD_DIR/"
    chmod +x "$SMART_FORWARD_DIR/smart-forward"
    
    # 清理临时文件
    cd /
    rm -rf "$temp_dir"
    
    echo "✅ 二进制文件安装完成"
}

# 安装配置文件
install_config() {
    echo "⚙️  安装配置文件..."
    
    mkdir -p "$CONFIG_DIR"
    
    # 如果当前目录有openwrt-config.yaml，使用它
    if [[ -f "openwrt-config.yaml" ]]; then
        cp openwrt-config.yaml "$CONFIG_DIR/config.yaml"
        echo "✅ 使用当前目录的openwrt-config.yaml"
    else
        # 创建默认配置
        cat > "$CONFIG_DIR/config.yaml" << 'EOF'
# Smart Forward OpenWrt 测试配置
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"

buffer_size: 8192

rules:
  - name: "TEST_HTTP"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "httpbin.org:80"
      
  - name: "TEST_DNS"
    listen_port: 8053
    protocol: "tcp"
    targets:
      - "1.1.1.1:53"
EOF
        echo "✅ 创建默认测试配置"
    fi
}

# 安装服务脚本
install_service() {
    echo "🔧 安装OpenWrt服务脚本..."
    
    cat > "$SERVICE_FILE" << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10
USE_PROCD=1

PROG="/usr/local/bin/smart-forward"
CONF="/etc/smart-forward/config.yaml"

start_service() {
    echo "启动Smart Forward服务..."
    
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
}

status() {
    local pids=$(pidof smart-forward)
    if [ -n "$pids" ]; then
        echo "✅ Smart Forward正在运行, PID: $pids"
        
        # 检查内核规则
        echo "🔍 检查内核转发规则:"
        if command -v nft >/dev/null 2>&1; then
            echo "nftables规则:"
            nft list table inet smart_forward 2>/dev/null || echo "  未找到nftables规则"
        fi
        
        if command -v iptables >/dev/null 2>&1; then
            echo "iptables规则:"
            iptables -t nat -L SMART_FORWARD_PREROUTING 2>/dev/null || echo "  未找到iptables规则"
        fi
    else
        echo "❌ Smart Forward未运行"
        return 1
    fi
}

# 强制内核态模式
enable_kernel_mode() {
    echo "🚀 启用强制内核态转发模式..."
    stop
    
    procd_open_instance
    procd_set_param command "$PROG" -c "$CONF" --kernel-mode --firewall-backend auto
    procd_set_param respawn 3600 5 5
    procd_set_param file "$CONF"
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
    
    echo "✅ 内核态模式已启用"
}

# 强制用户态模式
enable_user_mode() {
    echo "📡 启用强制用户态转发模式..."
    stop
    
    procd_open_instance
    procd_set_param command "$PROG" -c "$CONF" --user-mode
    procd_set_param respawn 3600 5 5
    procd_set_param file "$CONF"
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
    
    echo "✅ 用户态模式已启用"
}
EOF

    chmod +x "$SERVICE_FILE"
    echo "✅ 服务脚本安装完成"
}

# 测试功能
test_functionality() {
    echo "🧪 测试Smart Forward功能..."
    
    # 验证配置
    echo "1️⃣ 验证配置文件..."
    "$SMART_FORWARD_DIR/smart-forward" -c "$CONFIG_DIR/config.yaml" --validate-config
    
    echo ""
    echo "2️⃣ 测试内核态转发支持..."
    if "$SMART_FORWARD_DIR/smart-forward" -c "$CONFIG_DIR/config.yaml" --kernel-mode --validate-config 2>/dev/null; then
        echo "✅ 内核态转发支持正常"
    else
        echo "⚠️  内核态转发可能不支持，将使用用户态模式"
    fi
}

# 显示使用说明
show_usage() {
    echo ""
    echo "🎉 Smart Forward v1.5.0 安装完成！"
    echo "=================================="
    echo ""
    echo "📋 服务管理命令:"
    echo "  启动服务:     /etc/init.d/smart-forward start"
    echo "  停止服务:     /etc/init.d/smart-forward stop"
    echo "  重启服务:     /etc/init.d/smart-forward restart"
    echo "  查看状态:     /etc/init.d/smart-forward status"
    echo "  开机启动:     /etc/init.d/smart-forward enable"
    echo ""
    echo "🚀 转发模式:"
    echo "  自动模式:     /etc/init.d/smart-forward start"
    echo "  强制内核态:   /etc/init.d/smart-forward enable_kernel_mode"
    echo "  强制用户态:   /etc/init.d/smart-forward enable_user_mode"
    echo ""
    echo "⚙️  配置文件:   $CONFIG_DIR/config.yaml"
    echo "📝 日志查看:    logread | grep smart-forward"
    echo ""
    echo "🔧 手动测试:"
    echo "  验证配置:     $SMART_FORWARD_DIR/smart-forward -c $CONFIG_DIR/config.yaml --validate-config"
    echo "  内核态测试:   sudo $SMART_FORWARD_DIR/smart-forward -c $CONFIG_DIR/config.yaml --kernel-mode"
    echo "  用户态测试:   $SMART_FORWARD_DIR/smart-forward -c $CONFIG_DIR/config.yaml --user-mode"
}

# 主函数
main() {
    check_permissions
    check_openwrt
    check_firewall
    
    echo ""
    read -p "是否继续安装Smart Forward v1.5.0? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "安装已取消"
        exit 0
    fi
    
    download_binary
    install_config
    install_service
    test_functionality
    show_usage
    
    echo ""
    echo "🎯 建议下一步:"
    echo "1. 编辑配置文件: vi $CONFIG_DIR/config.yaml"
    echo "2. 启动服务: /etc/init.d/smart-forward start"
    echo "3. 查看状态: /etc/init.d/smart-forward status"
    echo "4. 测试转发: curl -v http://your-openwrt-ip:8080"
}

# 执行主函数
main "$@"
