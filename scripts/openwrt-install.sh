#!/bin/bash

# Smart Forward OpenWrt 一键安装脚本
# 支持自动检测架构和内核态转发

set -e

REPO_URL="https://github.com/cls3389/smart-forward"
RELEASE_API="https://api.github.com/repos/cls3389/smart-forward/releases/latest"

echo "🚀 Smart Forward OpenWrt 一键安装脚本"
echo "============================================="

# 检查是否为OpenWrt
if [ ! -f "/etc/openwrt_release" ]; then
    echo "❌ 此脚本仅适用于OpenWrt系统"
    exit 1
fi

# 显示OpenWrt信息
echo "📋 OpenWrt 系统信息:"
cat /etc/openwrt_release
echo ""

# 检查网络连接
echo "🌐 检查网络连接..."
if ! ping -c 1 github.com >/dev/null 2>&1; then
    echo "❌ 网络连接失败，请检查网络设置"
    exit 1
fi
echo "✅ 网络连接正常"

# 检测架构
echo "🔍 检测系统架构..."
ARCH=$(uname -m)
case "$ARCH" in
    "x86_64")
        TARGET="x86_64-unknown-linux-musl"
        ;;
    "aarch64")
        TARGET="aarch64-unknown-linux-musl"
        ;;
    "armv7l"|"armv6l")
        TARGET="arm-unknown-linux-musleabihf"
        ;;
    "mips")
        TARGET="mips-unknown-linux-musl"
        ;;
    "mipsel")
        TARGET="mipsel-unknown-linux-musl"
        ;;
    *)
        echo "❌ 不支持的架构: $ARCH"
        echo "支持的架构: x86_64, aarch64, armv7l, mips, mipsel"
        exit 1
        ;;
esac

echo "✅ 检测到架构: $ARCH -> $TARGET"

# 检测防火墙后端
echo "🔍 检测防火墙后端..."
HAS_NFTABLES=false
HAS_IPTABLES=false

if command -v nft >/dev/null 2>&1; then
    echo "✅ 检测到nftables支持 (Firewall4)"
    HAS_NFTABLES=true
fi

if command -v iptables >/dev/null 2>&1; then
    echo "✅ 检测到iptables支持"
    HAS_IPTABLES=true
fi

if [ "$HAS_NFTABLES" = true ]; then
    FIREWALL_TYPE="nftables (Firewall4 - 推荐)"
elif [ "$HAS_IPTABLES" = true ]; then
    FIREWALL_TYPE="iptables (传统防火墙)"
else
    FIREWALL_TYPE="无防火墙后端 (仅用户态转发)"
fi

echo "🎯 防火墙后端: $FIREWALL_TYPE"

# 获取最新版本
echo "📥 获取最新版本信息..."
LATEST_VERSION=$(curl -s "$RELEASE_API" | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$LATEST_VERSION" ]; then
    echo "❌ 获取版本信息失败"
    exit 1
fi
echo "✅ 最新版本: $LATEST_VERSION"

# 构建下载URL
BINARY_NAME="smart-forward"
DOWNLOAD_URL="$REPO_URL/releases/download/$LATEST_VERSION/smart-forward-$TARGET"

echo "📥 下载二进制文件..."
echo "URL: $DOWNLOAD_URL"

# 下载到临时目录
TMP_DIR="/tmp/smart-forward-install"
mkdir -p "$TMP_DIR"
cd "$TMP_DIR"

if ! curl -L -o "$BINARY_NAME" "$DOWNLOAD_URL"; then
    echo "❌ 下载失败，请检查网络或版本是否支持当前架构"
    exit 1
fi

# 验证下载
if [ ! -f "$BINARY_NAME" ] || [ ! -s "$BINARY_NAME" ]; then
    echo "❌ 下载的文件无效"
    exit 1
fi

echo "✅ 下载完成"

# 安装二进制文件
echo "📦 安装二进制文件..."
chmod +x "$BINARY_NAME"
mv "$BINARY_NAME" /usr/local/bin/smart-forward

# 创建配置目录
echo "📁 创建配置目录..."
mkdir -p /etc/smart-forward

# 创建默认配置
if [ ! -f "/etc/smart-forward/config.yaml" ]; then
    echo "📝 创建默认配置..."
    cat > /etc/smart-forward/config.yaml << 'EOF'
# Smart Forward OpenWrt 配置
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"

# 转发规则示例
rules:
  - name: "Web"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "192.168.1.100:80"
      - "backup.example.com:80"
      
  - name: "SSH"
    listen_port: 2222
    protocol: "tcp"
    targets:
      - "192.168.1.200:22"
EOF
    echo "✅ 默认配置已创建: /etc/smart-forward/config.yaml"
else
    echo "⚠️  配置文件已存在，跳过创建"
fi

# 下载并安装服务脚本
echo "🔧 安装服务脚本..."
SERVICE_SCRIPT_URL="$REPO_URL/raw/main/scripts/openwrt-service.sh"
if curl -s -L -o /etc/init.d/smart-forward "$SERVICE_SCRIPT_URL"; then
    chmod +x /etc/init.d/smart-forward
    echo "✅ 服务脚本安装完成"
else
    echo "⚠️  服务脚本下载失败，手动创建基础版本"
    # 创建基础服务脚本
    cat > /etc/init.d/smart-forward << 'EOF'
#!/bin/sh /etc/rc.common

NAME=smart-forward
USE_PROCD=1
START=99
STOP=10

start_service() {
    local BIN="/usr/local/bin/smart-forward"
    local CONF="/etc/smart-forward/config.yaml"
    
    procd_open_instance
    procd_set_param command "$BIN" -c "$CONF"
    procd_set_param cwd /etc/smart-forward
    procd_set_param respawn 3600 5 5
    procd_set_param file "$CONF"
    procd_close_instance
}
EOF
    chmod +x /etc/init.d/smart-forward
fi

# 询问是否启用内核态转发
echo ""
echo "🚀 内核态转发配置"
echo "============================================="
if [ "$HAS_NFTABLES" = true ] || [ "$HAS_IPTABLES" = true ]; then
    echo "检测到防火墙支持，可以启用内核态转发获得更好性能"
    echo "内核态转发优势："
    echo "  ✅ 更低延迟"
    echo "  ✅ 更高吞吐量" 
    echo "  ✅ 更少CPU占用"
    echo ""
    read -p "是否启用内核态转发? [Y/n]: " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        touch /etc/smart-forward/kernel-mode
        echo "✅ 内核态转发已启用"
    else
        echo "📡 将使用用户态转发"
    fi
else
    echo "⚠️  未检测到防火墙后端，将使用用户态转发"
fi

# 询问是否开机自启
echo ""
read -p "是否设置开机自启? [Y/n]: " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
    /etc/init.d/smart-forward enable
    echo "✅ 开机自启已启用"
else
    echo "⚠️  开机自启未启用"
fi

# 询问是否立即启动
echo ""
read -p "是否立即启动服务? [Y/n]: " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
    /etc/init.d/smart-forward start
    echo "🎉 服务启动完成！"
else
    echo "⚠️  服务未启动，可手动启动: /etc/init.d/smart-forward start"
fi

# 清理临时文件
cd /
rm -rf "$TMP_DIR"

echo ""
echo "🎉 Smart Forward 安装完成！"
echo "============================================="
echo "📁 配置文件: /etc/smart-forward/config.yaml"
echo "🔧 服务管理: /etc/init.d/smart-forward {start|stop|restart|status}"
echo "📊 查看状态: /etc/init.d/smart-forward status"
echo "📝 查看日志: logread | grep smart-forward"
echo ""
echo "🚀 内核态转发管理:"
echo "  启用: /etc/init.d/smart-forward enable_kernel_mode"
echo "  禁用: /etc/init.d/smart-forward disable_kernel_mode"
echo ""
echo "📖 项目地址: $REPO_URL"
echo "🎯 请编辑配置文件后重启服务以生效"