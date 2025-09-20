#!/bin/bash

# 清理443端口转发规则脚本
# 适用于OpenWrt环境，清理iptables和nftables中的443端口规则

echo "🧹 清理443端口转发规则"
echo "========================"

# 检查权限
if [[ $EUID -ne 0 ]]; then
    echo "❌ 需要root权限来清理防火墙规则"
    echo "请使用: sudo $0"
    exit 1
fi

# 清理iptables规则
cleanup_iptables() {
    echo "🔧 清理iptables中的443端口规则..."
    
    if ! command -v iptables >/dev/null 2>&1; then
        echo "⚠️  iptables未安装，跳过"
        return
    fi
    
    echo "当前iptables NAT规则:"
    iptables -t nat -L -n --line-numbers | grep -E "(443|HTTPS)" || echo "未找到443相关规则"
    
    # 清理PREROUTING规则 (DNAT)
    echo "清理PREROUTING DNAT规则..."
    iptables -t nat -L PREROUTING --line-numbers -n | grep ":443 " | tac | while read line; do
        line_num=$(echo $line | awk '{print $1}')
        if [[ "$line_num" =~ ^[0-9]+$ ]]; then
            echo "删除PREROUTING规则 #$line_num"
            iptables -t nat -D PREROUTING $line_num 2>/dev/null || true
        fi
    done
    
    # 清理POSTROUTING规则 (SNAT/MASQUERADE)
    echo "清理POSTROUTING SNAT/MASQUERADE规则..."
    iptables -t nat -L POSTROUTING --line-numbers -n | grep -E "(MASQUERADE|SNAT)" | tac | while read line; do
        if echo "$line" | grep -q "443"; then
            line_num=$(echo $line | awk '{print $1}')
            if [[ "$line_num" =~ ^[0-9]+$ ]]; then
                echo "删除POSTROUTING规则 #$line_num"
                iptables -t nat -D POSTROUTING $line_num 2>/dev/null || true
            fi
        fi
    done
    
    # 清理FORWARD规则
    echo "清理FORWARD规则..."
    iptables -L FORWARD --line-numbers -n | grep ":443 " | tac | while read line; do
        line_num=$(echo $line | awk '{print $1}')
        if [[ "$line_num" =~ ^[0-9]+$ ]]; then
            echo "删除FORWARD规则 #$line_num"
            iptables -D FORWARD $line_num 2>/dev/null || true
        fi
    done
    
    echo "✅ iptables清理完成"
}

# 清理nftables规则
cleanup_nftables() {
    echo "🔧 清理nftables中的443端口规则..."
    
    if ! command -v nft >/dev/null 2>&1; then
        echo "⚠️  nftables未安装，跳过"
        return
    fi
    
    echo "当前nftables规则集:"
    nft list ruleset | grep -C 2 "443" || echo "未找到443相关规则"
    
    # 清理可能存在的smart_forward表
    if nft list table inet smart_forward >/dev/null 2>&1; then
        echo "删除smart_forward表..."
        nft delete table inet smart_forward
        echo "✅ smart_forward表已删除"
    fi
    
    # 清理其他表中的443规则
    nft list tables | while read table_line; do
        if echo "$table_line" | grep -q "table"; then
            family=$(echo "$table_line" | awk '{print $2}')
            table=$(echo "$table_line" | awk '{print $3}')
            
            # 跳过smart_forward表（已删除）
            if [[ "$table" == "smart_forward" ]]; then
                continue
            fi
            
            echo "检查表: $family $table"
            
            # 获取包含443的规则句柄
            nft -a list table $family $table 2>/dev/null | grep "443" | grep "handle" | while read rule_line; do
                handle=$(echo "$rule_line" | grep -o "handle [0-9]*" | awk '{print $2}')
                chain=$(echo "$rule_line" | grep -o "chain [a-zA-Z_]*" | awk '{print $2}')
                
                if [[ -n "$handle" && -n "$chain" ]]; then
                    echo "删除规则: $family $table $chain handle $handle"
                    nft delete rule $family $table $chain handle $handle 2>/dev/null || true
                fi
            done
        fi
    done
    
    echo "✅ nftables清理完成"
}

# 清理OpenWrt防火墙配置
cleanup_openwrt_firewall() {
    echo "🔧 检查OpenWrt防火墙配置..."
    
    if [[ ! -f /etc/config/firewall ]]; then
        echo "⚠️  OpenWrt防火墙配置文件不存在，跳过"
        return
    fi
    
    echo "当前防火墙配置中的443端口规则:"
    grep -n "443" /etc/config/firewall || echo "未找到443端口规则"
    
    # 备份原配置
    cp /etc/config/firewall /etc/config/firewall.backup.$(date +%Y%m%d_%H%M%S)
    echo "✅ 防火墙配置已备份"
    
    # 这里不自动删除配置文件中的规则，因为可能影响其他服务
    echo "⚠️  请手动检查 /etc/config/firewall 中的443端口规则"
    echo "   如需删除，请编辑该文件并运行: /etc/init.d/firewall restart"
}

# 检查端口占用
check_port_usage() {
    echo "🔍 检查443端口占用情况..."
    
    echo "netstat检查:"
    netstat -tulpn 2>/dev/null | grep ":443 " || echo "443端口未被占用"
    
    echo "ss检查:"
    ss -tulpn 2>/dev/null | grep ":443 " || echo "443端口未被占用"
    
    echo "进程检查:"
    lsof -i :443 2>/dev/null || echo "未找到占用443端口的进程"
}

# 重启相关服务
restart_services() {
    echo "🔄 重启相关服务..."
    
    # 重启防火墙服务
    if [[ -f /etc/init.d/firewall ]]; then
        echo "重启OpenWrt防火墙..."
        /etc/init.d/firewall restart
    fi
    
    # 如果有smart-forward服务，停止它
    if [[ -f /etc/init.d/smart-forward ]]; then
        echo "停止smart-forward服务..."
        /etc/init.d/smart-forward stop 2>/dev/null || true
    fi
    
    echo "✅ 服务重启完成"
}

# 显示清理结果
show_cleanup_result() {
    echo ""
    echo "🎯 清理结果检查:"
    echo "=================="
    
    echo "1. 端口占用检查:"
    netstat -tulpn 2>/dev/null | grep ":443 " || echo "   ✅ 443端口未被占用"
    
    echo "2. iptables NAT规则:"
    iptables -t nat -L -n | grep "443" || echo "   ✅ 未找到443相关iptables规则"
    
    echo "3. nftables规则:"
    nft list ruleset | grep "443" || echo "   ✅ 未找到443相关nftables规则"
    
    echo ""
    echo "✅ 443端口转发规则清理完成！"
    echo "现在可以安全地部署Smart Forward了。"
}

# 主函数
main() {
    echo "开始清理443端口转发规则..."
    echo "目标: 清理所有与443端口相关的转发规则"
    echo ""
    
    cleanup_iptables
    echo ""
    cleanup_nftables
    echo ""
    cleanup_openwrt_firewall
    echo ""
    check_port_usage
    echo ""
    restart_services
    echo ""
    show_cleanup_result
}

# 执行主函数
main "$@"
