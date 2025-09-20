#!/bin/sh /etc/rc.common

# Copyright (C) 2025 Smart Forward Project
# This is free software, licensed under the MIT License.

NAME=smart-forward
USE_PROCD=1
START=99
STOP=10

# Description:
#	Smart Forward - 智能端口转发服务
#	支持 TCP/UDP 多规则转发与健康检查
#	支持内核态转发 (nftables/iptables)
#	项目地址: https://github.com/cls3389/smart-forward

start_service() {
    # 必须先定义变量！
    local BIN="/usr/local/bin/smart-forward"
    local CONF="/etc/smart-forward/config.yaml"
    local LOG_FILE="/tmp/smart-forward-start.log"

    # 把所有输出打到日志
    exec >> "$LOG_FILE" 2>&1
    echo "=== 启动 smart-forward 开始 ==="
    date

    [ ! -f "$CONF" ] && {
        echo "❌ 错误: 配置文件不存在: $CONF"
        echo "请运行: opkg install smart-forward 或手动创建配置"
        exit 1
    }
    [ ! -x "$BIN" ] && {
        echo "❌ 错误: 无执行权限: $BIN"
        ls -l "$BIN"
        exit 1
    }

    echo "✅ 配置和二进制检查通过"

    # 检测防火墙后端
    local FIREWALL_BACKEND="auto"
    if command -v nft >/dev/null 2>&1; then
        echo "✅ 检测到nftables支持 (Firewall4)"
        FIREWALL_BACKEND="nftables"
    elif command -v iptables >/dev/null 2>&1; then
        echo "✅ 检测到iptables支持"
        FIREWALL_BACKEND="iptables"
    else
        echo "⚠️  未检测到防火墙后端，使用用户态转发"
    fi

    # 智能选择转发模式：默认自动尝试内核态
    local KERNEL_MODE=""
    if [ -f "/etc/smart-forward/user-mode-only" ]; then
        echo "📡 强制使用用户态转发模式"
        KERNEL_MODE="--user-mode"
    elif [ -f "/etc/smart-forward/kernel-mode-force" ]; then
        echo "🚀 强制启用内核态转发模式"
        KERNEL_MODE="--kernel-mode --firewall-backend $FIREWALL_BACKEND"
    else
        echo "🚀 自动优先内核态转发（失败自动回退用户态）"
        KERNEL_MODE="--firewall-backend $FIREWALL_BACKEND"
    fi

    procd_open_instance
    procd_set_param command "$BIN" -c "$CONF" $KERNEL_MODE
    procd_set_param cwd /etc/smart-forward
    procd_set_param respawn 3600 5 5
    procd_set_param file "$CONF"
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance

    echo "=== procd 实例已打开 ==="
    echo "启动命令: $BIN -c $CONF $KERNEL_MODE"
}

# 自定义 status
status() {
    echo "=== $NAME 状态 ==="
    local pids=$(pidof smart-forward)
    if [ -n "$pids" ]; then
        echo "✅ 正在运行, PID: $pids"
        
        # 显示内核态转发状态
        if command -v nft >/dev/null 2>&1; then
            if nft list table inet smart_forward >/dev/null 2>&1; then
                echo "🚀 内核态转发: 已启用 (nftables)"
            else
                echo "📡 内核态转发: 未启用"
            fi
        elif command -v iptables >/dev/null 2>&1; then
            if iptables -t nat -L SMART_FORWARD_PREROUTING >/dev/null 2>&1; then
                echo "🚀 内核态转发: 已启用 (iptables)"
            else
                echo "📡 内核态转发: 未启用"
            fi
        fi
    else
        echo "❌ 未运行"
    fi
    echo "日志查看: logread | grep smart-forward"
    echo "启动日志: cat /tmp/smart-forward-start.log"
}

# 强制启用内核态转发
force_kernel_mode() {
    echo "🚀 强制启用内核态转发模式..."
    rm -f /etc/smart-forward/user-mode-only
    touch /etc/smart-forward/kernel-mode-force
    echo "✅ 强制内核态转发已启用，重启服务生效"
    echo "重启命令: /etc/init.d/smart-forward restart"
}

# 强制使用用户态转发
force_user_mode() {
    echo "📡 强制使用用户态转发模式..."
    rm -f /etc/smart-forward/kernel-mode-force
    touch /etc/smart-forward/user-mode-only
    echo "✅ 强制用户态转发已启用，重启服务生效"
    echo "重启命令: /etc/init.d/smart-forward restart"
}

# 恢复自动模式（默认）
auto_mode() {
    echo "🚀 恢复自动模式（优先内核态，失败回退用户态）..."
    rm -f /etc/smart-forward/kernel-mode-force
    rm -f /etc/smart-forward/user-mode-only
    echo "✅ 自动模式已启用，重启服务生效"
    echo "重启命令: /etc/init.d/smart-forward restart"
}

# 显示帮助
help() {
    echo "Smart Forward OpenWrt 服务管理"
    echo ""
    echo "基本命令:"
    echo "  /etc/init.d/smart-forward start     - 启动服务"
    echo "  /etc/init.d/smart-forward stop      - 停止服务"
    echo "  /etc/init.d/smart-forward restart   - 重启服务"
    echo "  /etc/init.d/smart-forward status    - 查看状态"
    echo "  /etc/init.d/smart-forward enable    - 开机自启"
    echo "  /etc/init.d/smart-forward disable   - 禁用自启"
    echo ""
    echo "转发模式管理:"
    echo "  /etc/init.d/smart-forward auto_mode         - 自动模式（默认，推荐）"
    echo "  /etc/init.d/smart-forward force_kernel_mode - 强制内核态"
    echo "  /etc/init.d/smart-forward force_user_mode   - 强制用户态"
    echo ""
    echo "配置文件: /etc/smart-forward/config.yaml"
    echo "项目地址: https://github.com/cls3389/smart-forward"
}
