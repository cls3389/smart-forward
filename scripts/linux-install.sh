#!/bin/bash
# Linux 通用安装脚本
# 用于在各种Linux发行版上安装和运行 smart-forward
# 
# 使用方法:
#   ./linux-install.sh                       # 默认安装musl版本 (推荐)
#   BINARY_TYPE=gnu ./linux-install.sh       # 安装GNU版本  
#   BINARY_TYPE=musl ./linux-install.sh      # 明确指定musl版本
#
# 二进制类型说明:
#   musl: 静态链接，零依赖，兼容所有Linux发行版 (推荐)
#   gnu:  动态链接，性能稍好，需要glibc 2.17+

set -e

# 配置变量
APP_NAME="smart-forward"
APP_VERSION="latest"
APP_URL="https://github.com/cls3389/smart-forward/releases/latest/download"
CONFIG_DIR="/etc/smart-forward"
LOG_DIR="/var/log/smart-forward"
BIN_DIR="/usr/local/bin"
SERVICE_DIR="/etc/systemd/system"

# 二进制类型选择 (可通过环境变量修改)
# musl: 静态链接，更好兼容性，推荐用于生产环境 (默认)
# gnu:  动态链接，需要glibc，性能稍好
BINARY_TYPE="${BINARY_TYPE:-musl}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# 检查是否为root用户
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_info "以root用户运行"
        USE_SUDO=""
    else
        print_info "以普通用户运行，将使用sudo"
        USE_SUDO="sudo"
        
        # 检查sudo权限
        if ! command -v sudo >/dev/null 2>&1; then
            print_error "需要sudo权限来安装系统服务"
            print_info "请以root用户运行或安装sudo"
            exit 1
        fi
    fi
}

# 检测系统信息
detect_system() {
    print_step "检测系统信息..."
    
    # 检测操作系统
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS_NAME=$NAME
        OS_VERSION=$VERSION_ID
    else
        OS_NAME="Unknown"
        OS_VERSION="Unknown"
    fi
    
    print_info "操作系统: $OS_NAME $OS_VERSION"
    
    # 检测架构
    local arch=$(uname -m)
    case $arch in
        x86_64|amd64)
            ARCH="linux-x86_64"
            print_info "检测到架构: $arch -> $ARCH (x86 64位)"
            ;;
        aarch64|arm64)
            ARCH="linux-aarch64"
            print_info "检测到架构: $arch -> $ARCH (ARM 64位)"
            ;;
        *)
            print_error "不支持的架构: $arch"
            print_info "支持的架构: x86_64, aarch64"
            exit 1
            ;;
    esac
    
    # 显示二进制类型信息
    if [ "$BINARY_TYPE" = "musl" ]; then
        print_info "二进制类型: musl (静态链接，推荐用于生产环境)"
    elif [ "$BINARY_TYPE" = "gnu" ]; then
        print_info "二进制类型: GNU (动态链接，需要glibc 2.17+)"
        # 检查glibc版本
        local glibc_version=$(ldd --version | head -n1 | grep -o '[0-9]\+\.[0-9]\+' | head -1)
        if [ -n "$glibc_version" ]; then
            print_info "系统glibc版本: $glibc_version"
        else
            print_warn "无法检测glibc版本，GNU版本可能不兼容"
        fi
    else
        print_error "不支持的二进制类型: $BINARY_TYPE"
        print_info "支持的类型: musl, gnu"
        exit 1
    fi
}

# 检查依赖
check_dependencies() {
    print_step "检查依赖..."
    
    local missing_deps=()
    
    # 检查 wget 或 curl
    if command -v wget >/dev/null 2>&1; then
        DOWNLOAD_CMD="wget -O"
        print_info "下载工具: wget"
    elif command -v curl >/dev/null 2>&1; then
        DOWNLOAD_CMD="curl -L -o"
        print_info "下载工具: curl"
    else
        missing_deps+=("wget 或 curl")
    fi
    
    # 检查 tar
    if ! command -v tar >/dev/null 2>&1; then
        missing_deps+=("tar")
    fi
    
    # 检查 systemctl (systemd)
    if ! command -v systemctl >/dev/null 2>&1; then
        print_warn "未检测到systemd，将跳过系统服务安装"
        INSTALL_SERVICE=false
    else
        print_info "检测到systemd支持"
        INSTALL_SERVICE=true
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "缺少依赖: ${missing_deps[*]}"
        print_info "请安装依赖后重试"
        
        # 根据发行版提供安装建议
        if command -v apt >/dev/null 2>&1; then
            print_info "Ubuntu/Debian: sudo apt update && sudo apt install -y wget tar"
        elif command -v yum >/dev/null 2>&1; then
            print_info "CentOS/RHEL: sudo yum install -y wget tar"
        elif command -v dnf >/dev/null 2>&1; then
            print_info "Fedora: sudo dnf install -y wget tar"
        elif command -v pacman >/dev/null 2>&1; then
            print_info "Arch Linux: sudo pacman -S wget tar"
        fi
        exit 1
    fi
    
    print_info "依赖检查通过"
}

# 下载二进制文件
download_binary() {
    print_step "下载 $APP_NAME 二进制文件..."
    
    # 构建文件名
    local file_suffix="${ARCH}-${BINARY_TYPE}.tar.gz"
    local download_url="$APP_URL/smart-forward-$file_suffix"
    local temp_file="/tmp/smart-forward-$file_suffix"
    
    print_info "下载地址: $download_url"
    
    # 下载文件
    if $DOWNLOAD_CMD "$temp_file" "$download_url"; then
        print_info "下载成功"
    else
        print_error "下载失败，请检查网络连接或GitHub Release是否存在"
        print_info "手动下载: https://github.com/cls3389/smart-forward/releases/latest"
        exit 1
    fi
    
    # 解压文件
    print_info "解压文件..."
    cd /tmp
    if tar -xzf "$temp_file"; then
        print_info "解压成功"
    else
        print_error "解压失败，文件可能损坏"
        exit 1
    fi
    
    # 查找二进制文件
    local binary_file="/tmp/smart-forward"
    if [ ! -f "$binary_file" ]; then
        print_error "找不到二进制文件"
        print_info "解压内容:"
        ls -la /tmp/ | grep -E "(smart|forward)"
        exit 1
    fi
    
    # 安装二进制文件
    print_info "安装二进制文件到 $BIN_DIR..."
    $USE_SUDO mkdir -p "$BIN_DIR"
    $USE_SUDO cp "$binary_file" "$BIN_DIR/smart-forward"
    $USE_SUDO chmod +x "$BIN_DIR/smart-forward"
    
    # 验证安装
    if "$BIN_DIR/smart-forward" --version >/dev/null 2>&1; then
        local version=$("$BIN_DIR/smart-forward" --version 2>/dev/null || echo "unknown")
        print_info "二进制文件安装完成: $version"
    else
        print_warn "二进制文件已安装，但版本验证失败（可能是架构不兼容）"
    fi
    
    # 清理临时文件
    rm -f "$temp_file" "$binary_file"
    print_info "临时文件已清理"
}

# 创建配置目录和文件
create_config() {
    print_step "创建配置目录和文件..."
    
    # 创建目录
    $USE_SUDO mkdir -p "$CONFIG_DIR"
    $USE_SUDO mkdir -p "$LOG_DIR"
    
    # 创建配置文件
    $USE_SUDO tee "$CONFIG_DIR/config.yaml" > /dev/null << 'EOF'
# ================================
# 智能网络转发器配置文件
# ================================

# 日志配置
logging:
  level: "info"
  format: "text"

# 网络配置  
network:
  listen_addr: "0.0.0.0"

# 缓冲区大小
buffer_size: 8192

# 转发规则
rules:
  - name: "HTTPS转发示例"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "example.com:443"
      
  - name: "HTTP转发示例"
    listen_port: 80
    protocol: "tcp"
    targets:
      - "example.com:80"
      
  # 添加更多规则...
EOF
    
    print_info "配置文件创建完成: $CONFIG_DIR/config.yaml"
    print_warn "请编辑配置文件设置您的转发规则"
}

# 创建systemd服务文件
create_systemd_service() {
    if [ "$INSTALL_SERVICE" != "true" ]; then
        print_warn "跳过systemd服务安装"
        return
    fi
    
    print_step "创建systemd服务..."
    
    $USE_SUDO tee "$SERVICE_DIR/smart-forward.service" > /dev/null << EOF
[Unit]
Description=Smart Forward - 智能网络转发器
After=network.target
Wants=network.target

[Service]
Type=simple
User=nobody
Group=nogroup
ExecStart=$BIN_DIR/smart-forward -c $CONFIG_DIR/config.yaml
ExecReload=/bin/kill -HUP \$MAINPID
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

# 安全选项
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$LOG_DIR

[Install]
WantedBy=multi-user.target
EOF
    
    # 重新加载systemd
    $USE_SUDO systemctl daemon-reload
    
    print_info "systemd服务创建完成"
}

# 创建管理脚本
create_management_script() {
    print_step "创建管理脚本..."
    
    $USE_SUDO tee "/usr/local/bin/smart-forward-ctl" > /dev/null << 'EOF'
#!/bin/bash
# Smart Forward 管理脚本

usage() {
    echo "Smart Forward 管理工具"
    echo ""
    echo "用法: $0 <命令>"
    echo ""
    echo "命令:"
    echo "  start     启动服务"
    echo "  stop      停止服务"  
    echo "  restart   重启服务"
    echo "  status    查看状态"
    echo "  logs      查看日志"
    echo "  config    编辑配置"
    echo "  version   查看版本"
    echo ""
}

case "$1" in
    start)
        if command -v systemctl >/dev/null 2>&1; then
            sudo systemctl start smart-forward
        else
            echo "systemd不可用，请手动启动"
        fi
        ;;
    stop)
        if command -v systemctl >/dev/null 2>&1; then
            sudo systemctl stop smart-forward
        else
            pkill -f smart-forward
        fi
        ;;
    restart)
        if command -v systemctl >/dev/null 2>&1; then
            sudo systemctl restart smart-forward
        else
            pkill -f smart-forward
            sleep 1
            echo "请手动重新启动服务"
        fi
        ;;
    status)
        if command -v systemctl >/dev/null 2>&1; then
            systemctl status smart-forward
        else
            if pgrep -f smart-forward >/dev/null; then
                echo "Smart Forward 正在运行"
            else
                echo "Smart Forward 未运行"
            fi
        fi
        ;;
    logs)
        if command -v journalctl >/dev/null 2>&1; then
            journalctl -u smart-forward -f
        elif [ -f "/var/log/smart-forward/smart-forward.log" ]; then
            tail -f /var/log/smart-forward/smart-forward.log
        else
            echo "找不到日志文件"
        fi
        ;;
    config)
        ${EDITOR:-vi} /etc/smart-forward/config.yaml
        ;;
    version)
        /usr/local/bin/smart-forward --version
        ;;
    *)
        usage
        exit 1
        ;;
esac
EOF
    
    $USE_SUDO chmod +x "/usr/local/bin/smart-forward-ctl"
    print_info "管理脚本创建完成: smart-forward-ctl"
}

# 启动服务
start_service() {
    if [ "$INSTALL_SERVICE" != "true" ]; then
        print_warn "systemd不可用，请手动启动服务:"
        print_info "$BIN_DIR/smart-forward -c $CONFIG_DIR/config.yaml"
        return
    fi
    
    print_step "配置并启动服务..."
    
    # 启用服务
    $USE_SUDO systemctl enable smart-forward
    print_info "服务已设置为开机自启"
    
    # 提示用户编辑配置
    print_warn "在启动服务前，请先编辑配置文件:"
    print_info "smart-forward-ctl config"
    print_info ""
    print_info "配置完成后启动服务:"
    print_info "smart-forward-ctl start"
}

# 显示使用说明
show_usage() {
    print_info "安装完成！"
    echo ""
    echo "📋 管理命令："
    echo "  启动服务: smart-forward-ctl start"
    echo "  停止服务: smart-forward-ctl stop"
    echo "  重启服务: smart-forward-ctl restart"
    echo "  查看状态: smart-forward-ctl status"
    echo "  查看日志: smart-forward-ctl logs"
    echo "  编辑配置: smart-forward-ctl config"
    echo "  查看版本: smart-forward-ctl version"
    echo ""
    echo "📁 重要文件："
    echo "  配置文件: $CONFIG_DIR/config.yaml"
    echo "  日志目录: $LOG_DIR/"
    echo "  二进制文件: $BIN_DIR/smart-forward"
    echo ""
    echo "⚡ 下一步："
    echo "  1. 编辑配置文件: smart-forward-ctl config"
    echo "  2. 启动服务: smart-forward-ctl start"
    echo "  3. 查看状态: smart-forward-ctl status"
    echo ""
    
    if [ "$BINARY_TYPE" = "musl" ]; then
        print_info "✅ 使用musl版本，完美兼容所有Linux发行版"
    else
        print_info "✅ 使用GNU版本，性能优化，需要glibc 2.17+"
    fi
}

# 主函数
main() {
    echo "================================"
    echo "   Smart Forward Linux 安装器"
    echo "================================"
    echo ""
    
    check_root
    detect_system
    check_dependencies
    download_binary
    create_config
    create_systemd_service
    create_management_script
    start_service
    show_usage
    
    echo ""
    print_info "🎉 安装完成！"
}

# 运行主函数
main "$@"
