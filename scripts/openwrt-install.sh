#!/bin/sh
# OpenWrt 安装脚本
# 用于在 MT7981 等 OpenWrt 设备上安装和运行 smart-forward
# 
# 使用方法:
#   ./openwrt-install.sh                     # 默认安装musl版本 (推荐)
#   BINARY_TYPE=gnu ./openwrt-install.sh     # 安装GNU版本
#   BINARY_TYPE=musl ./openwrt-install.sh    # 明确指定musl版本
#
# 二进制类型说明:
#   musl: 静态链接，零依赖，兼容所有OpenWrt设备 (推荐)
#   gnu:  动态链接，性能稍好，需要glibc 2.17+

set -e

# 配置变量
APP_NAME="smart-forward"
APP_VERSION="latest"
APP_URL="https://github.com/cls3389/smart-forward/releases/latest/download"
CONFIG_DIR="/etc/smart-forward"
LOG_DIR="/var/log/smart-forward"
BIN_DIR="/usr/local/bin"

# 二进制类型选择 (可通过环境变量修改)
# musl: 静态链接，更好兼容性，推荐用于OpenWrt (默认)
# gnu:  动态链接，需要glibc，性能稍好
BINARY_TYPE="${BINARY_TYPE:-musl}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# 检查架构
check_architecture() {
    local arch=$(uname -m)
    case $arch in
        aarch64|arm64)
            ARCH="linux-aarch64"
            print_info "检测到架构: $arch -> $ARCH (ARM 64位)"
            ;;
        x86_64|amd64)
            ARCH="linux-x86_64"
            print_info "检测到架构: $arch -> $ARCH (x86 64位)"
            ;;
        armv7l|armv7)
            ARCH="linux-armv7"
            print_warn "检测到架构: $arch -> $ARCH (ARM 32位，性能较低)"
            print_warn "建议使用 aarch64 设备以获得更好性能"
            ;;
        mips|mipsel)
            ARCH="linux-mips"
            print_warn "检测到架构: $arch -> $ARCH (MIPS 架构)"
            print_warn "MIPS 架构可能性能较低"
            ;;
        *)
            print_error "不支持的架构: $arch"
            print_info "支持的架构: aarch64, x86_64, armv7, mips"
            exit 1
            ;;
    esac
    
    # 显示二进制类型信息
    if [ "$BINARY_TYPE" = "musl" ]; then
        print_info "二进制类型: musl (静态链接，推荐用于OpenWrt)"
    elif [ "$BINARY_TYPE" = "gnu" ]; then
        print_info "二进制类型: GNU (动态链接，需要glibc 2.17+)"
    else
        print_error "不支持的二进制类型: $BINARY_TYPE"
        print_info "支持的类型: musl, gnu"
        exit 1
    fi
}

# 检查依赖
check_dependencies() {
    print_info "检查依赖..."
    
    # 检查 wget 或 curl
    if command -v wget >/dev/null 2>&1; then
        DOWNLOAD_CMD="wget -O"
    elif command -v curl >/dev/null 2>&1; then
        DOWNLOAD_CMD="curl -L -o"
    else
        print_error "需要 wget 或 curl 来下载文件"
        print_info "安装命令: opkg update && opkg install wget"
        exit 1
    fi
    
    # 检查 tar
    if ! command -v tar >/dev/null 2>&1; then
        print_error "需要 tar 来解压文件"
        print_info "安装命令: opkg update && opkg install tar"
        exit 1
    fi
    
    print_info "依赖检查通过"
}

# 下载二进制文件
download_binary() {
    print_info "下载 $APP_NAME 二进制文件..."
    
    # 根据架构和二进制类型选择正确的文件名 (匹配GitHub Release命名)
    local file_suffix
    case "$ARCH" in
        "linux-aarch64")
            file_suffix="linux-aarch64-$BINARY_TYPE.tar.gz"
            ;;
        "linux-x86_64")
            file_suffix="linux-x86_64-$BINARY_TYPE.tar.gz"
            ;;
        "linux-armv7")
            # ARM32位暂时不支持，使用ARMv7的musl版本（如果有）
            print_warn "ARM32位架构暂不提供预编译版本"
            print_warn "将尝试使用aarch64版本，可能不兼容"
            file_suffix="linux-aarch64-$BINARY_TYPE.tar.gz"
            ;;
        "linux-mips")
            # MIPS架构不提供预编译版本
            print_warn "MIPS架构暂不提供预编译版本"
            print_warn "将尝试使用x86_64版本，可能不兼容"
            file_suffix="linux-x86_64-$BINARY_TYPE.tar.gz"
            ;;
        *)
            print_error "不支持的架构: $ARCH"
            exit 1
            ;;
    esac
    
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
    
    # 解压文件 (GitHub Release中的tar.gz已经是正确格式)
    print_info "解压文件..."
    cd /tmp
    if tar -xzf "$temp_file"; then
        print_info "解压成功"
    else
        print_error "解压失败，文件可能损坏"
        exit 1
    fi
    
    # 查找二进制文件 (解压后可能在当前目录或子目录)
    local binary_file="/tmp/smart-forward"
    if [ ! -f "$binary_file" ]; then
        # 尝试在解压目录中查找
        binary_file=$(find /tmp -name "smart-forward" -type f 2>/dev/null | head -1)
        if [ -z "$binary_file" ]; then
            print_error "找不到二进制文件"
            print_info "解压内容:"
            ls -la /tmp/ | grep -E "(smart|forward)"
            exit 1
        fi
    fi
    
    # 安装二进制文件
    print_info "安装二进制文件到 $BIN_DIR..."
    mkdir -p "$BIN_DIR"
    cp "$binary_file" "$BIN_DIR/smart-forward"
    chmod +x "$BIN_DIR/smart-forward"
    
    # 验证安装
    if "$BIN_DIR/smart-forward" --version >/dev/null 2>&1; then
        print_info "二进制文件安装完成并验证成功"
    else
        print_warn "二进制文件已安装，但版本验证失败（可能是架构不兼容）"
    fi
    
    # 清理临时文件
    rm -f "$temp_file"
    [ -f "$binary_file" ] && rm -f "$binary_file"
    
    print_info "二进制文件安装完成"
}

# 创建配置目录和文件
create_config() {
    print_info "创建配置目录和文件..."
    
    # 创建目录
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$LOG_DIR"
    
    # 创建配置文件
    cat > "$CONFIG_DIR/config.yaml" << 'EOF'
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
}

# 创建启动脚本
create_startup_script() {
    print_info "创建启动脚本..."
    
    cat > "/etc/init.d/smart-forward" << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10

USE_PROCD=1
PROG="/usr/local/bin/smart-forward"
CONFIG="/etc/smart-forward/config.yaml"

start_service() {
    procd_open_instance
    procd_set_param command "$PROG" -c "$CONFIG"
    procd_set_param respawn
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
}
EOF
    
    chmod +x "/etc/init.d/smart-forward"
    
    print_info "启动脚本创建完成"
}

# 创建管理脚本
create_management_script() {
    print_info "创建管理脚本..."
    
    cat > "/usr/local/bin/smart-forward-ctl" << 'EOF'
#!/bin/sh
# Smart Forward 管理脚本

case "$1" in
    start)
        /etc/init.d/smart-forward start
        ;;
    stop)
        /etc/init.d/smart-forward stop
        ;;
    restart)
        /etc/init.d/smart-forward restart
        ;;
    status)
        /etc/init.d/smart-forward status
        ;;
    logs)
        tail -f /var/log/smart-forward/smart-forward.log
        ;;
    config)
        vi /etc/smart-forward/config.yaml
        ;;
    *)
        echo "用法: $0 {start|stop|restart|status|logs|config}"
        exit 1
        ;;
esac
EOF
    
    chmod +x "/usr/local/bin/smart-forward-ctl"
    
    print_info "管理脚本创建完成"
}

# 启动服务
start_service() {
    print_info "启动服务..."
    
    # 启用服务
    /etc/init.d/smart-forward enable
    
    # 启动服务
    /etc/init.d/smart-forward start
    
    # 等待服务启动
    sleep 2
    
    # 检查服务状态
    if /etc/init.d/smart-forward status >/dev/null 2>&1; then
        print_info "服务启动成功"
    else
        print_error "服务启动失败"
        print_info "查看日志: smart-forward-ctl logs"
        exit 1
    fi
}

# 显示使用说明
show_usage() {
    print_info "安装完成！"
    echo ""
    echo "管理命令："
    echo "  启动服务: smart-forward-ctl start"
    echo "  停止服务: smart-forward-ctl stop"
    echo "  重启服务: smart-forward-ctl restart"
    echo "  查看状态: smart-forward-ctl status"
    echo "  查看日志: smart-forward-ctl logs"
    echo "  编辑配置: smart-forward-ctl config"
    echo ""
    echo "配置文件: $CONFIG_DIR/config.yaml"
    echo "日志文件: $LOG_DIR/smart-forward.log"
    echo ""
    echo "请编辑配置文件后重启服务："
    echo "  smart-forward-ctl config"
    echo "  smart-forward-ctl restart"
}

# 主函数
main() {
    print_info "开始安装 $APP_NAME..."
    
    check_architecture
    check_dependencies
    download_binary
    create_config
    create_startup_script
    create_management_script
    start_service
    show_usage
    
    print_info "安装完成！"
}

# 运行主函数
main "$@"
