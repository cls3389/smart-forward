#!/bin/bash

# SSH部署到OpenWrt (Cudy) 脚本
# 自动连接、清理现有规则、部署Smart Forward

set -e

# 配置变量
OPENWRT_HOST="cudy"  # 您的OpenWrt主机名或IP
SSH_KEY_PATH="$HOME/.ssh/id_rsa"  # 默认SSH密钥路径
REMOTE_USER="root"   # OpenWrt默认用户

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Smart Forward OpenWrt SSH部署脚本${NC}"
echo "============================================="

# 检查SSH密钥
check_ssh_key() {
    echo -e "${YELLOW}🔑 检查SSH密钥...${NC}"
    
    if [[ ! -f "$SSH_KEY_PATH" ]]; then
        echo -e "${RED}❌ SSH密钥不存在: $SSH_KEY_PATH${NC}"
        echo "请确保SSH密钥存在，或修改脚本中的SSH_KEY_PATH变量"
        exit 1
    fi
    
    echo -e "${GREEN}✅ SSH密钥找到: $SSH_KEY_PATH${NC}"
}

# 测试SSH连接
test_ssh_connection() {
    echo -e "${YELLOW}🔗 测试SSH连接到 $OPENWRT_HOST...${NC}"
    
    if ssh -i "$SSH_KEY_PATH" -o ConnectTimeout=10 -o BatchMode=yes "$REMOTE_USER@$OPENWRT_HOST" "echo 'SSH连接成功'" 2>/dev/null; then
        echo -e "${GREEN}✅ SSH连接正常${NC}"
    else
        echo -e "${RED}❌ SSH连接失败${NC}"
        echo "请检查："
        echo "1. 主机名/IP是否正确: $OPENWRT_HOST"
        echo "2. SSH密钥是否正确: $SSH_KEY_PATH"
        echo "3. OpenWrt是否允许SSH连接"
        exit 1
    fi
}

# 清理现有的443端口转发规则
cleanup_existing_rules() {
    echo -e "${YELLOW}🧹 清理现有的443端口转发规则...${NC}"
    
    ssh -i "$SSH_KEY_PATH" "$REMOTE_USER@$OPENWRT_HOST" << 'EOF'
echo "检查现有的443端口转发规则..."

# 检查并清理iptables规则
if command -v iptables >/dev/null 2>&1; then
    echo "清理iptables中的443端口规则..."
    
    # 清理PREROUTING规则 (DNAT)
    iptables -t nat -L PREROUTING --line-numbers -n | grep ":443 " | while read line; do
        line_num=$(echo $line | awk '{print $1}')
        if [[ "$line_num" =~ ^[0-9]+$ ]]; then
            echo "删除PREROUTING规则 #$line_num: $line"
            iptables -t nat -D PREROUTING $line_num 2>/dev/null || true
        fi
    done
    
    # 清理POSTROUTING规则 (SNAT/MASQUERADE)
    iptables -t nat -L POSTROUTING --line-numbers -n | grep -E "(MASQUERADE|SNAT)" | while read line; do
        if echo "$line" | grep -q "443"; then
            line_num=$(echo $line | awk '{print $1}')
            if [[ "$line_num" =~ ^[0-9]+$ ]]; then
                echo "删除POSTROUTING规则 #$line_num: $line"
                iptables -t nat -D POSTROUTING $line_num 2>/dev/null || true
            fi
        fi
    done
    
    # 清理FORWARD规则
    iptables -L FORWARD --line-numbers -n | grep ":443 " | while read line; do
        line_num=$(echo $line | awk '{print $1}')
        if [[ "$line_num" =~ ^[0-9]+$ ]]; then
            echo "删除FORWARD规则 #$line_num: $line"
            iptables -D FORWARD $line_num 2>/dev/null || true
        fi
    done
fi

# 检查并清理nftables规则
if command -v nft >/dev/null 2>&1; then
    echo "清理nftables中的443端口规则..."
    
    # 列出所有包含443的规则
    nft list ruleset | grep -n "443" || echo "未找到nftables 443端口规则"
    
    # 清理可能的443端口规则 (需要根据实际规则调整)
    # 这里提供一个通用的清理方法
    nft list tables | while read table_info; do
        if echo "$table_info" | grep -q "table"; then
            family=$(echo "$table_info" | awk '{print $2}')
            table=$(echo "$table_info" | awk '{print $3}')
            
            echo "检查表: $family $table"
            nft list table $family $table 2>/dev/null | grep -n "443" || true
        fi
    done
fi

# 检查进程占用443端口
echo "检查443端口占用情况..."
netstat -tulpn 2>/dev/null | grep ":443 " || echo "443端口未被占用"
ss -tulpn 2>/dev/null | grep ":443 " || echo "443端口未被占用 (ss检查)"

# 检查OpenWrt防火墙配置
if [[ -f /etc/config/firewall ]]; then
    echo "检查OpenWrt防火墙配置中的443端口规则..."
    grep -n "443" /etc/config/firewall || echo "防火墙配置中未找到443端口规则"
fi

echo "现有规则清理完成"
EOF

    echo -e "${GREEN}✅ 现有规则清理完成${NC}"
}

# 上传配置文件
upload_config() {
    echo -e "${YELLOW}📤 上传配置文件...${NC}"
    
    # 检查本地配置文件
    if [[ ! -f "openwrt-config.yaml" ]]; then
        echo -e "${RED}❌ 本地配置文件不存在: openwrt-config.yaml${NC}"
        echo "请确保当前目录有openwrt-config.yaml文件"
        exit 1
    fi
    
    # 上传配置文件
    scp -i "$SSH_KEY_PATH" openwrt-config.yaml "$REMOTE_USER@$OPENWRT_HOST:/tmp/smart-forward-config.yaml"
    
    echo -e "${GREEN}✅ 配置文件上传完成${NC}"
}

# 上传并执行部署脚本
deploy_smart_forward() {
    echo -e "${YELLOW}🚀 部署Smart Forward...${NC}"
    
    # 检查本地部署脚本
    if [[ ! -f "openwrt-test.sh" ]]; then
        echo -e "${RED}❌ 本地部署脚本不存在: openwrt-test.sh${NC}"
        echo "请确保当前目录有openwrt-test.sh文件"
        exit 1
    fi
    
    # 上传部署脚本
    scp -i "$SSH_KEY_PATH" openwrt-test.sh "$REMOTE_USER@$OPENWRT_HOST:/tmp/openwrt-test.sh"
    
    # 在远程执行部署
    ssh -i "$SSH_KEY_PATH" "$REMOTE_USER@$OPENWRT_HOST" << 'EOF'
echo "开始执行Smart Forward部署..."

cd /tmp
chmod +x openwrt-test.sh

# 如果有自定义配置，复制到正确位置
if [[ -f "smart-forward-config.yaml" ]]; then
    echo "使用自定义配置文件"
    cp smart-forward-config.yaml openwrt-config.yaml
fi

# 执行自动部署 (非交互模式)
echo "y" | ./openwrt-test.sh

echo "部署完成"
EOF

    echo -e "${GREEN}✅ Smart Forward部署完成${NC}"
}

# 启动服务并测试
start_and_test() {
    echo -e "${YELLOW}🔧 启动服务并测试...${NC}"
    
    ssh -i "$SSH_KEY_PATH" "$REMOTE_USER@$OPENWRT_HOST" << 'EOF'
echo "启动Smart Forward服务..."

# 启动服务
/etc/init.d/smart-forward start

# 等待服务启动
sleep 3

# 检查服务状态
echo "检查服务状态..."
/etc/init.d/smart-forward status

# 检查进程
echo "检查进程..."
ps | grep smart-forward || echo "未找到smart-forward进程"

# 检查端口监听
echo "检查端口监听..."
netstat -tulpn | grep smart-forward || echo "未找到smart-forward端口监听"

# 检查内核规则
echo "检查内核转发规则..."
if command -v nft >/dev/null 2>&1; then
    echo "nftables规则:"
    nft list table inet smart_forward 2>/dev/null || echo "未找到nftables规则"
fi

if command -v iptables >/dev/null 2>&1; then
    echo "iptables规则:"
    iptables -t nat -L SMART_FORWARD_PREROUTING 2>/dev/null || echo "未找到iptables规则"
fi

# 检查日志
echo "最近的日志:"
logread | grep smart-forward | tail -10 || echo "未找到相关日志"

echo "服务启动和测试完成"
EOF

    echo -e "${GREEN}✅ 服务启动和测试完成${NC}"
}

# 显示测试命令
show_test_commands() {
    echo ""
    echo -e "${BLUE}🧪 测试命令:${NC}"
    echo "=================================="
    echo ""
    echo "在您的本地机器上测试："
    echo -e "${YELLOW}# 测试HTTPS转发 (443端口)${NC}"
    echo "curl -v -k https://$OPENWRT_HOST"
    echo ""
    echo -e "${YELLOW}# 测试RDP转发 (99端口)${NC}"
    echo "telnet $OPENWRT_HOST 99"
    echo ""
    echo -e "${YELLOW}# 测试网盘转发 (6690端口)${NC}"
    echo "curl -v http://$OPENWRT_HOST:6690"
    echo ""
    echo "SSH到OpenWrt查看详细状态："
    echo -e "${YELLOW}ssh -i $SSH_KEY_PATH $REMOTE_USER@$OPENWRT_HOST${NC}"
    echo ""
    echo "在OpenWrt上的管理命令："
    echo "  查看状态: /etc/init.d/smart-forward status"
    echo "  重启服务: /etc/init.d/smart-forward restart"
    echo "  查看日志: logread | grep smart-forward"
    echo "  强制内核态: /etc/init.d/smart-forward enable_kernel_mode"
}

# 主函数
main() {
    echo "目标主机: $OPENWRT_HOST"
    echo "SSH密钥: $SSH_KEY_PATH"
    echo "远程用户: $REMOTE_USER"
    echo ""
    
    check_ssh_key
    test_ssh_connection
    cleanup_existing_rules
    upload_config
    deploy_smart_forward
    start_and_test
    show_test_commands
    
    echo ""
    echo -e "${GREEN}🎉 Smart Forward部署完成！${NC}"
    echo "现在您可以测试内核态转发功能了。"
}

# 执行主函数
main "$@"
