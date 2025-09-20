#!/bin/bash

# OpenWrt Firewall4 (nftables) 内核态转发测试脚本

echo "🚀 Smart Forward - Firewall4 内核态转发测试"
echo "============================================="

# 检查运行环境
echo "📋 检查运行环境..."

# 检查是否为Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ 此功能仅支持Linux系统（特别是OpenWrt）"
    exit 1
fi

# 检查是否有root权限
if [[ $EUID -ne 0 ]]; then
    echo "❌ 需要root权限来管理防火墙规则"
    echo "请使用: sudo $0"
    exit 1
fi

# 检查nftables支持
echo "🔍 检查防火墙后端支持..."
HAS_NFTABLES=false
HAS_IPTABLES=false

if command -v nft &> /dev/null; then
    echo "✅ 检测到nftables支持"
    HAS_NFTABLES=true
fi

if command -v iptables &> /dev/null; then
    echo "✅ 检测到iptables支持"  
    HAS_IPTABLES=true
fi

if [[ "$HAS_NFTABLES" == false && "$HAS_IPTABLES" == false ]]; then
    echo "❌ 未检测到支持的防火墙后端"
    exit 1
fi

# 推荐使用nftables（Firewall4），但兼容iptables
if [[ "$HAS_NFTABLES" == true ]]; then
    RECOMMENDED_BACKEND="nftables"
    echo "🎯 推荐使用: nftables (Firewall4 - 新版OpenWrt)"
elif [[ "$HAS_IPTABLES" == true ]]; then
    RECOMMENDED_BACKEND="iptables"
    echo "🎯 推荐使用: iptables (传统OpenWrt)"
else
    echo "❌ 未检测到支持的防火墙后端"
    exit 1
fi

# 创建测试配置
echo "📝 创建测试配置..."
cat > test-kernel-config.yaml << 'EOF'
# Firewall4 (nftables) 内核态转发测试配置
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"

rules:
  - name: "Web-Kernel"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "192.168.1.100:80"
      - "backup.example.com:80"
      
  - name: "SSH-Kernel"  
    listen_port: 2222
    protocol: "tcp"
    targets:
      - "192.168.1.200:22"
EOF

echo "✅ 测试配置已创建: test-kernel-config.yaml"

# 显示使用说明
echo ""
echo "🎯 内核态转发测试命令："
echo "============================================="
echo ""
echo "1️⃣ 验证配置（推荐先执行）："
echo "   ./smart-forward -c test-kernel-config.yaml --validate-config --kernel-mode --firewall-backend $RECOMMENDED_BACKEND"
echo ""
echo "2️⃣ 启动内核态转发："
echo "   sudo ./smart-forward -c test-kernel-config.yaml --kernel-mode --firewall-backend $RECOMMENDED_BACKEND"
echo ""
echo "3️⃣ 自动检测防火墙后端："
echo "   sudo ./smart-forward -c test-kernel-config.yaml --kernel-mode --firewall-backend auto"
echo ""
echo "4️⃣ 测试转发效果："
echo "   curl http://localhost:8080  # 应该转发到192.168.1.100:80"
echo "   ssh -p 2222 localhost       # 应该转发到192.168.1.200:22"
echo ""

# 显示Firewall4优先级说明
echo "🔥 Firewall4 优先级优化说明："
echo "============================================="
echo "✅ smart-forward使用优先级-150的prerouting链"
echo "✅ 高于Firewall4默认DNAT规则（优先级-100）"
echo "✅ 确保转发到外网地址不被覆盖"
echo "✅ 专用table避免与现有规则冲突"
echo ""

# 显示防火墙规则查看命令
echo "🔍 查看防火墙规则："
if [[ "$HAS_NFTABLES" == true ]]; then
    echo "   # nftables规则："
    echo "   nft list table inet smart_forward"
    echo "   nft list chain inet smart_forward prerouting"
    echo "   nft list chain inet smart_forward postrouting"
fi
if [[ "$HAS_IPTABLES" == true ]]; then
    echo "   # iptables规则："
    echo "   iptables -t nat -L SMART_FORWARD_PREROUTING -v"
    echo "   iptables -t nat -L SMART_FORWARD_POSTROUTING -v"
    echo "   iptables -t nat -L PREROUTING --line-numbers"
fi
echo ""

echo "🎉 测试环境准备完成！"
echo "请按照上述命令进行测试。"
