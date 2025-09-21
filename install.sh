#!/bin/bash
# Smart Forward 交互式安装脚本
# 支持 Linux、OpenWrt、Docker 环境自动检测和安装
# 支持智能升级：保留现有配置，清理旧程序
# 新特性：交互式配置监听地址和转发规则

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 版本信息
VERSION="v1.5.6"
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
    elif [ -f /etc/os-release ]; then
        echo "linux"
    else
        echo "unknown"
    fi
}

# 检测是否在Docker容器中
is_docker_container() {
    if [ -f /.dockerenv ] || [ -d /.dockerinit ] || grep -q "docker" /proc/1/cgroup 2>/dev/null; then
        return 0  # 真，是Docker容器
    else
        return 1  # 假，不是Docker容器
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
        arm*)
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
            if [ "$arch" = "armv7" ]; then
                url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/smart-forward-linux-armv7-musl.tar.gz"
            else
                url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/smart-forward-linux-${arch}-musl.tar.gz"
            fi
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

# 备份现有配置
backup_existing_config() {
    local config_path="$1"
    if [ -f "$config_path" ]; then
        local backup_path="${config_path}.backup.$(date +%Y%m%d_%H%M%S)"
        print_info "备份现有配置: $config_path -> $backup_path"
        cp "$config_path" "$backup_path"
        return 0
    fi
    return 1
}

# 停止现有服务
stop_existing_service() {
    local system="$1"
    
    print_info "检查并停止现有服务..."
    
    case $system in
        openwrt)
            if [ -f /etc/init.d/smart-forward ]; then
                print_info "停止现有 OpenWrt 服务..."
                /etc/init.d/smart-forward stop 2>/dev/null || true
                /etc/init.d/smart-forward disable 2>/dev/null || true
            fi
            ;;
        linux)
            if systemctl is-active smart-forward >/dev/null 2>&1; then
                print_info "停止现有 systemd 服务..."
                sudo systemctl stop smart-forward 2>/dev/null || true
                sudo systemctl disable smart-forward 2>/dev/null || true
            fi
            ;;
    esac
    
    # 强制停止所有 smart-forward 进程
    if command -v pkill >/dev/null 2>&1; then
        pkill -f smart-forward 2>/dev/null || true
    else
        # 如果没有 pkill，使用 kill
        local pids=$(pgrep -f smart-forward 2>/dev/null || true)
        if [ -n "$pids" ]; then
            kill $pids 2>/dev/null || true
            sleep 2
            # 如果还在运行，强制杀死
            pids=$(pgrep -f smart-forward 2>/dev/null || true)
            if [ -n "$pids" ]; then
                kill -9 $pids 2>/dev/null || true
            fi
        fi
    fi
    
    print_success "现有服务已停止"
}

# 清理旧文件
cleanup_old_files() {
    local system="$1"
    
    print_info "清理旧的程序文件..."
    
    case $system in
        openwrt)
            # 删除旧的二进制文件
            [ -f /usr/bin/smart-forward ] && rm -f /usr/bin/smart-forward
            [ -f /usr/local/bin/smart-forward ] && rm -f /usr/local/bin/smart-forward
            # 删除旧的服务脚本
            [ -f /etc/init.d/smart-forward ] && rm -f /etc/init.d/smart-forward
            ;;
        linux)
            # 删除旧的二进制文件
            [ -f /usr/local/bin/smart-forward ] && sudo rm -f /usr/local/bin/smart-forward
            [ -f /usr/bin/smart-forward ] && sudo rm -f /usr/bin/smart-forward
            # 删除旧的服务文件
            [ -f /etc/systemd/system/smart-forward.service ] && sudo rm -f /etc/systemd/system/smart-forward.service
            ;;
    esac
    
    print_success "旧文件清理完成"
}

# 安装到 OpenWrt
install_openwrt() {
    local config_file="$1"
    print_info "在 OpenWrt 上安装 Smart Forward..."
    
    # 停止现有服务并清理
    stop_existing_service "openwrt"
    cleanup_old_files "openwrt"
    
    # 解压二进制文件
    cd /tmp
    tar -xzf smart-forward.tar.gz
    
    # 安装二进制文件
    cp smart-forward /usr/bin/
    chmod +x /usr/bin/smart-forward
    
    # 创建配置目录
    mkdir -p /etc/smart-forward
    
    # 备份现有配置并创建新配置
    local config_exists=false
    if backup_existing_config "/etc/smart-forward/config.yaml"; then
        config_exists=true
        print_success "已保留现有配置文件"
    fi
    
    # 只有在没有现有配置时才创建默认配置
    if [ "$config_exists" = false ]; then
        print_info "创建配置文件..."
        # 使用传入的配置文件
        if [ -f "$config_file" ]; then
            cp "$config_file" /etc/smart-forward/config.yaml
            print_success "使用交互式配置"
        else
            print_warning "配置文件不存在，使用默认配置"
            cat > /etc/smart-forward/config.yaml << 'EOF'
# Smart Forward 配置文件
# 详细配置说明: https://github.com/cls3389/smart-forward

logging:
  level: "info"
  format: "text"  # OpenWrt推荐使用text格式

network:
  listen_addrs:
    - "0.0.0.0"

buffer_size: 8192

dynamic_update:
  check_interval: 5
  connection_timeout: 2
  auto_reconnect: true

rules:
  - name: "HTTP_TEST"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "httpbin.org:80"
EOF
        fi
    fi
    
    
    
    
    # 创建 OpenWrt 服务脚本
    cat > /etc/init.d/smart-forward << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10

USE_PROCD=1
PROG="/usr/bin/smart-forward"
CONF="/etc/smart-forward/config.yaml"

start_service() {
    echo "启动Smart Forward服务..."
    
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
    local config_file="$1"
    print_info "在 Linux 上安装 Smart Forward..."
    
    # 停止现有服务并清理
    stop_existing_service "linux"
    cleanup_old_files "linux"
    
    # 解压二进制文件
    cd /tmp
    tar -xzf smart-forward.tar.gz
    
    # 安装二进制文件
    if command -v sudo >/dev/null 2>&1; then
        sudo cp smart-forward /usr/bin/
        sudo chmod +x /usr/bin/smart-forward
    else
        cp smart-forward /usr/bin/
        chmod +x /usr/bin/smart-forward
    fi
    
    # 创建配置目录
    if command -v sudo >/dev/null 2>&1; then
        sudo mkdir -p /etc/smart-forward
    else
        mkdir -p /etc/smart-forward
    fi

    # 备份现有配置并创建新配置
    local config_exists=false
    if backup_existing_config "/etc/smart-forward/config.yaml"; then
        config_exists=true
        print_success "已保留现有配置文件"
    fi

    # 只有在没有现有配置时才创建默认配置
    if [ "$config_exists" = false ]; then
        print_info "创建默认配置文件..."
        if command -v sudo >/dev/null 2>&1; then
            sudo tee /etc/smart-forward/config.yaml > /dev/null << 'EOF'
        else
            tee /etc/smart-forward/config.yaml > /dev/null << 'EOF'
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
  - name: "HTTP_TEST"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "httpbin.org:80"
EOF
        fi
    fi
    
    
    
    
    # 创建 systemd 服务文件
    if command -v sudo >/dev/null 2>&1; then
        sudo tee /etc/systemd/system/smart-forward.service > /dev/null << 'EOF'
    else
        tee /etc/systemd/system/smart-forward.service > /dev/null << 'EOF'
    fi
[Unit]
Description=Smart Forward - 智能网络转发器
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/bin/smart-forward -c /etc/smart-forward/config.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
User=root
Group=root

# 安全设置
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/etc/smart-forward
PrivateTmp=true

# 网络权限
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_RAW

[Install]
WantedBy=multi-user.target
EOF

    # 重新加载 systemd
    if command -v sudo >/dev/null 2>&1; then
        sudo systemctl daemon-reload
        sudo systemctl enable smart-forward
    else
        systemctl daemon-reload
        systemctl enable smart-forward
    fi
    
    print_success "Linux 安装完成"
    print_info "使用方法:"
    if command -v sudo >/dev/null 2>&1; then
        print_info "  启动服务: sudo systemctl start smart-forward"
        print_info "  查看状态: sudo systemctl status smart-forward"
        print_info "  查看日志: sudo journalctl -u smart-forward -f"
        print_info "  编辑配置: sudo vi /etc/smart-forward/config.yaml"
    else
        print_info "  启动服务: systemctl start smart-forward"
        print_info "  查看状态: systemctl status smart-forward"
        print_info "  查看日志: journalctl -u smart-forward -f"
        print_info "  编辑配置: vi /etc/smart-forward/config.yaml"
    fi
}

# 获取网络接口信息
get_network_interfaces() {
    local interfaces=""

    # 获取所有非lo回环接口
    if command -v ip >/dev/null 2>&1; then
        interfaces=$(ip addr show | grep -E "^[0-9]+:" | grep -v " lo:" | cut -d: -f2 | tr -d ' ' | head -5)
    elif command -v ifconfig >/dev/null 2>&1; then
        interfaces=$(ifconfig | grep -E "^[a-zA-Z]" | cut -d: -f1 | grep -v "^lo" | head -5)
    fi

    if [ -z "$interfaces" ]; then
        echo "0.0.0.0"
    else
        echo "$interfaces"
    fi
}

# 获取网络接口IP地址
get_interface_ip() {
    local interface="$1"

    if command -v ip >/dev/null 2>&1; then
        ip addr show "$interface" 2>/dev/null | grep -E "inet [0-9]" | head -1 | awk '{print $2}' | cut -d/ -f1
    elif command -v ifconfig >/dev/null 2>&1; then
        ifconfig "$interface" 2>/dev/null | grep -E "inet [0-9]" | head -1 | awk '{print $2}'
    else
        echo ""
    fi
}

# 交互式配置监听地址
configure_listen_addresses() {
    print_info "=== 网络配置 ==="
    print_info "检测到的网络接口:"

    local interfaces=$(get_network_interfaces)
    local index=1
    local ip_list=""

    for interface in $interfaces; do
        local ip=$(get_interface_ip "$interface")
        if [ -n "$ip" ] && [ "$ip" != "127.0.0.1" ] && [ "$ip" != "::1" ]; then
            print_info "  $index) $interface - $ip"
            ip_list="$ip_list $ip"
            index=$((index + 1))
        fi
    done

    if [ -z "$ip_list" ]; then
        print_warning "未检测到有效的网络接口IP地址"
        print_info "将使用默认配置: 0.0.0.0 (监听所有接口)"
        echo "0.0.0.0"
        return
    fi

    echo ""
    print_info "请选择监听地址 (可多选，用空格分隔):"
    print_info "  1-$(($index - 1))) 上述检测到的IP地址"
    print_info "  a) 所有地址 (0.0.0.0)"
    print_info "  c) 自定义IP地址"
    print_info "  d) 使用默认配置 (0.0.0.0)"
    echo ""

    local choice=""
    read -p "请输入选择 (默认: d): " choice

    case "$choice" in
        ""|"d"|"D")
            print_info "使用默认配置: 0.0.0.0"
            echo "0.0.0.0"
            ;;
        "a"|"A")
            print_info "使用所有地址: 0.0.0.0"
            echo "0.0.0.0"
            ;;
        "c"|"C")
            local custom_ip=""
            read -p "请输入自定义IP地址: " custom_ip
            if [ -n "$custom_ip" ]; then
                print_info "使用自定义地址: $custom_ip"
                echo "$custom_ip"
            else
                print_warning "未输入有效IP地址，使用默认配置"
                echo "0.0.0.0"
            fi
            ;;
        *)
            local selected_ips=""
            local valid_choice=false
            for i in $choice; do
                local num_ips=$(echo "$ip_list" | wc -w)
                if [ "$i" -ge 1 ] && [ "$i" -le "$num_ips" ] 2>/dev/null; then
                    local selected_ip=$(echo "$ip_list" | awk "{print \$$i}")
                    if [ -n "$selected_ip" ]; then
                        selected_ips="$selected_ips $selected_ip"
                        valid_choice=true
                    fi
                fi
            done

            if [ "$valid_choice" = true ]; then
                echo "$selected_ips" | sed 's/^ *//'
            else
                print_warning "无效选择，使用默认配置"
                echo "0.0.0.0"
            fi
            ;;
    esac
}

# 交互式配置转发规则
configure_forwarding_rules() {
    print_info "=== 转发规则配置 ==="
    print_info "是否配置转发规则? (y/N)"
    local configure_rules=""

    read -p "请输入选择 (默认: N): " configure_rules

    if [ "$configure_rules" != "y" ] && [ "$configure_rules" != "Y" ]; then
        print_info "跳过规则配置，使用默认配置"
        return
    fi

    print_info "添加转发规则:"
    print_info "示例格式:"
    print_info "  - 规则名称: HTTP转发"
    print_info "  - 监听端口: 8080"
    print_info "  - 目标地址: httpbin.org:80"
    echo ""

    local rules=""

    while true; do
        print_info "添加新规则? (y/N)"
        local add_rule=""
        read -p "请输入选择 (默认: N): " add_rule

        if [ "$add_rule" != "y" ] && [ "$add_rule" != "Y" ]; then
            break
        fi

        local rule_name=""
        local listen_port=""
        local target_addr=""

        read -p "规则名称: " rule_name
        read -p "监听端口: " listen_port
        read -p "目标地址 (host:port): " target_addr

        if [ -n "$rule_name" ] && [ -n "$listen_port" ] && [ -n "$target_addr" ]; then
            rules="$rules
  - name: \"$rule_name\"
    listen_port: $listen_port
    protocol: \"tcp\"
    targets:
      - \"$target_addr\""
            print_success "规则添加成功: $rule_name -> $target_addr"
        else
            print_warning "规则信息不完整，跳过此规则"
        fi

        echo ""
    done

    if [ -n "$rules" ]; then
        echo "rules:"
        echo "$rules" | sed 's/^ *//'
    fi
}

# 生成配置文件
generate_config() {
    local listen_addrs="$1"
    local rules="$2"

    cat << EOF
# Smart Forward 配置文件
# 由交互式安装脚本生成

logging:
  level: "info"
  format: "text"

network:
  listen_addrs:
    - "$listen_addrs"

buffer_size: 8192

dynamic_update:
  check_interval: 5
  connection_timeout: 2
  auto_reconnect: true

$rules
EOF
}

# 交互式安装
interactive_install() {
    echo "========================================"
    echo "  Smart Forward ${VERSION} 交互式安装"
    echo "  智能配置网络和转发规则"
    echo "========================================"
    echo ""

    # 配置监听地址
    local listen_addrs=$(configure_listen_addresses)

    # 配置转发规则
    local rules=$(configure_forwarding_rules)

    # 生成配置文件
    local config_content=$(generate_config "$listen_addrs" "$rules")

    # 询问是否继续安装
    echo ""
    print_info "=== 配置预览 ==="
    echo "$config_content" | head -20
    if [ "$(echo "$config_content" | wc -l)" -gt 20 ]; then
        echo "..."
    fi
    echo ""

    print_info "是否继续安装? (y/N)"
    local continue_install=""
    read -p "请输入选择 (默认: Y): " continue_install

    if [ "$continue_install" = "n" ] || [ "$continue_install" = "N" ]; then
        print_info "安装已取消"
        exit 0
    fi

    return 0
}

# 主函数
main() {
    echo "========================================"
    echo "  Smart Forward ${VERSION} 原生安装脚本"
    echo "  支持 Linux、OpenWrt 原生系统安装"
    echo "  智能升级：保留配置，清理旧程序"
    echo "========================================"
    echo ""

    # 检查是否在Docker容器中
    if is_docker_container; then
        print_warning "检测到Docker容器环境"
        print_info "Docker部署不需要此安装脚本"
        print_info "请使用以下Docker命令:"
        echo ""
        echo "  # 用户态转发 (推荐)"
        echo "  docker run -d --name smart-forward --network host \\"
        echo "    -v /path/to/config.yaml:/app/config.yaml \\"
        echo "    ghcr.io/cls3389/smart-forward:latest"
        echo ""
        echo "  # 内核态转发 (需要特权模式)"
        echo "  docker run -d --name smart-forward --privileged --network host \\"
        echo "    -v /path/to/config.yaml:/app/config.yaml \\"
        echo "    ghcr.io/cls3389/smart-forward:latest \\"
        echo "    --kernel-mode"
        echo ""
        print_info "更多Docker配置请参考: https://github.com/cls3389/smart-forward"
        exit 0
    fi

    local system=$(detect_system)
    local arch=$(detect_arch)

    print_info "检测到系统: $system"
    print_info "检测到架构: $arch"
    echo ""

    # 询问是否使用交互式配置
    print_info "是否使用交互式配置? (y/N)"
    local use_interactive=""
    read -p "请输入选择 (默认: N): " use_interactive

    local config_file="/tmp/smart-forward-config.yaml"

    if [ "$use_interactive" = "y" ] || [ "$use_interactive" = "Y" ]; then
        interactive_install > "$config_file"
        print_info "使用交互式配置..."
    else
        print_info "使用默认配置安装..."
        # 创建默认配置文件
        cat > "$config_file" << 'EOF'
# Smart Forward 配置文件
# 默认配置

logging:
  level: "info"
  format: "text"

network:
  listen_addrs:
    - "0.0.0.0"

buffer_size: 8192

dynamic_update:
  check_interval: 5
  connection_timeout: 2
  auto_reconnect: true

rules:
  - name: "HTTP_TEST"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "httpbin.org:80"
EOF
        fi
    fi

    case $system in
        openwrt)
            download_binary "$system" "$arch"
            install_openwrt "$config_file"
            ;;
        linux)
            download_binary "$system" "$arch"
            install_linux "$config_file"
            ;;
        *)
            print_error "不支持的系统: $system"
            print_info "支持的系统: Linux, OpenWrt"
            exit 1
            ;;
    esac
    
    echo ""
    print_success "安装完成！"
    echo ""
    print_info "升级特性:"
    print_info "  🔄 自动备份现有配置文件"
    print_info "  🗑️  清理旧的程序文件和服务"
    print_info "  ⚙️  保留用户自定义配置"
    echo ""
    print_info "功能特性:"
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