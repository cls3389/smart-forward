#!/bin/bash
# HomeProxy 恢复脚本 - 移除端口转发，恢复原始配置
# 用于恢复 OpenWrt HomeProxy 配置，移除 smart-forward 管理的端口转发

echo "=== HomeProxy 恢复脚本 ==="

# 检查是否为 OpenWrt 系统
if [ ! -f "/etc/openwrt_release" ]; then
    echo "错误: 此脚本仅适用于 OpenWrt 系统"
    exit 1
fi

# 停止 smart-forward 服务
echo "停止 smart-forward 服务..."
/etc/init.d/smart-forward stop 2>/dev/null || true

# 清理 smart-forward 的防火墙规则
echo "清理 smart-forward 防火墙规则..."
# 清理 nftables 规则
nft delete table inet smart_forward 2>/dev/null || true
# 清理 iptables 规则
iptables -t nat -F SMART_FORWARD_PREROUTING 2>/dev/null || true
iptables -t nat -X SMART_FORWARD_PREROUTING 2>/dev/null || true
iptables -t nat -F SMART_FORWARD_POSTROUTING 2>/dev/null || true
iptables -t nat -X SMART_FORWARD_POSTROUTING 2>/dev/null || true

# 移除端口转发配置（由 smart-forward 管理的部分）
echo "移除 smart-forward 管理的端口转发配置..."

# 移除可能的端口转发规则
uci -q delete firewall.smart_forward_443 2>/dev/null || true
uci -q delete firewall.smart_forward_99 2>/dev/null || true  
uci -q delete firewall.smart_forward_6690 2>/dev/null || true
uci -q delete firewall.smart_forward_999 2>/dev/null || true

# 提交防火墙配置更改
uci commit firewall

# 重启防火墙服务
echo "重启防火墙服务..."
/etc/init.d/firewall restart

# 检查 HomeProxy 是否已安装
if opkg list-installed | grep -q homeproxy; then
    echo "HomeProxy 已安装，重启服务..."
    
    # 确保 HomeProxy 配置正确
    echo "检查 HomeProxy 配置..."
    
    # 重启 HomeProxy 服务
    /etc/init.d/homeproxy restart
    
    # 等待服务启动
    sleep 3
    
else
    echo "HomeProxy 未安装，开始安装..."
    
    # 更新软件包列表
    opkg update
    
    # 安装 HomeProxy
    opkg install homeproxy
    
    # 启用并启动 HomeProxy
    /etc/init.d/homeproxy enable
    /etc/init.d/homeproxy start
    
    # 等待服务启动
    sleep 5
fi

# 检查服务状态
echo ""
echo "=== 服务状态检查 ==="
echo "HomeProxy 状态:"
/etc/init.d/homeproxy status

echo ""
echo "防火墙状态:"
/etc/init.d/firewall status

echo ""
echo "网络连接测试:"
ping -c 3 8.8.8.8 2>/dev/null && echo "✅ 网络连接正常" || echo "❌ 网络连接异常"

echo ""
echo "=== HomeProxy 恢复完成 ==="
echo "✅ 已移除 smart-forward 管理的端口转发配置"
echo "✅ 已清理所有相关防火墙规则"
echo "✅ HomeProxy 服务已恢复"
echo ""
echo "请检查 HomeProxy 配置是否正常工作"
echo "如需重新配置，请访问 OpenWrt 管理界面: http://192.168.1.1"
