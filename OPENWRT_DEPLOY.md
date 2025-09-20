# OpenWrt Smart Forward v1.5.0 部署指南

## 🚀 快速部署 (推荐)

### 1. 自动安装脚本
```bash
# 下载并运行自动安装脚本
wget https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/openwrt-install.sh
chmod +x openwrt-install.sh
sudo ./openwrt-install.sh
```

### 2. 本地测试脚本
如果您已经有了配置文件和测试脚本：
```bash
# 使用本地测试脚本
chmod +x openwrt-test.sh
sudo ./openwrt-test.sh
```

## 📦 手动部署

### 1. 下载二进制文件

根据您的OpenWrt架构下载对应的文件：

**x86_64 架构 (常见):**
```bash
wget https://github.com/cls3389/smart-forward/releases/download/v1.5.0/smart-forward-linux-x86_64-musl.tar.gz
```

**ARM64 架构:**
```bash
wget https://github.com/cls3389/smart-forward/releases/download/v1.5.0/smart-forward-linux-aarch64-musl.tar.gz
```

**MIPS 架构:**
```bash
wget https://github.com/cls3389/smart-forward/releases/download/v1.5.0/smart-forward-linux-mips-musl.tar.gz
```

### 2. 安装二进制文件
```bash
# 解压
tar -xzf smart-forward-linux-*-musl.tar.gz

# 安装
sudo mkdir -p /usr/local/bin
sudo cp smart-forward /usr/local/bin/
sudo chmod +x /usr/local/bin/smart-forward
```

### 3. 创建配置目录和文件
```bash
# 创建配置目录
sudo mkdir -p /etc/smart-forward

# 复制配置文件
sudo cp openwrt-config.yaml /etc/smart-forward/config.yaml
```

### 4. 创建服务脚本
```bash
# 复制服务脚本 (从openwrt-test.sh中提取)
sudo cp scripts/openwrt-service.sh /etc/init.d/smart-forward
sudo chmod +x /etc/init.d/smart-forward
```

## 🔧 配置和启动

### 1. 编辑配置文件
```bash
sudo vi /etc/smart-forward/config.yaml
```

根据您的需求修改：
- 监听端口
- 目标地址
- 协议类型
- 日志级别

### 2. 验证配置
```bash
# 验证配置文件
/usr/local/bin/smart-forward -c /etc/smart-forward/config.yaml --validate-config

# 测试内核态支持
sudo /usr/local/bin/smart-forward -c /etc/smart-forward/config.yaml --kernel-mode --validate-config
```

### 3. 启动服务
```bash
# 启动服务 (自动模式，优先内核态)
/etc/init.d/smart-forward start

# 查看状态
/etc/init.d/smart-forward status

# 设置开机启动
/etc/init.d/smart-forward enable
```

## 🚀 转发模式选择

### 自动模式 (推荐)
```bash
/etc/init.d/smart-forward start
```
- 自动检测防火墙后端
- 优先使用内核态转发
- 失败时自动回退到用户态

### 强制内核态模式
```bash
/etc/init.d/smart-forward enable_kernel_mode
```
- 强制使用内核态转发
- 需要root权限
- 性能最佳

### 强制用户态模式
```bash
/etc/init.d/smart-forward enable_user_mode
```
- 强制使用用户态转发
- 兼容性最好
- 无需特殊权限

## 🔍 监控和调试

### 查看日志
```bash
# 实时日志
logread -f | grep smart-forward

# 历史日志
logread | grep smart-forward
```

### 检查内核规则
```bash
# nftables规则 (Firewall4)
sudo nft list table inet smart_forward

# iptables规则 (传统)
sudo iptables -t nat -L SMART_FORWARD_PREROUTING
```

### 性能测试
```bash
# 测试HTTP转发
curl -v http://your-openwrt-ip:8080

# 测试DNS转发
dig @your-openwrt-ip -p 8053 google.com

# 测试RDP转发
telnet your-openwrt-ip 99
```

## 🛠️ 故障排除

### 1. 内核态转发失败
```bash
# 检查防火墙支持
nft --version
iptables --version

# 检查权限
sudo /usr/local/bin/smart-forward --kernel-mode --validate-config
```

### 2. 端口冲突
```bash
# 检查端口占用
netstat -tulpn | grep :443
ss -tulpn | grep :443
```

### 3. DNS解析问题
```bash
# 测试DNS解析
nslookup ewin10.4.ipto.top
dig TXT ewin10.4.ipto.top
```

### 4. 防火墙规则冲突
```bash
# 检查现有规则
iptables -t nat -L -n --line-numbers
nft list ruleset
```

## 📊 性能对比

| 模式 | 延迟 | 吞吐量 | CPU占用 | 适用场景 |
|------|------|--------|---------|----------|
| 内核态 | <0.1ms | 10Gbps+ | <5% | 生产环境 |
| 用户态 | 1-2ms | 1Gbps | 10-20% | 测试环境 |

## 🎯 最佳实践

1. **生产环境**: 使用内核态转发 + nftables
2. **测试环境**: 使用自动模式
3. **调试问题**: 使用用户态 + debug日志
4. **高负载**: 调整缓冲区大小 (仅用户态有效)
5. **多规则**: 合理设置优先级和健康检查间隔

## 📞 技术支持

如果遇到问题，请提供以下信息：
- OpenWrt版本: `cat /etc/openwrt_release`
- 架构信息: `uname -a`
- 防火墙版本: `nft --version` 或 `iptables --version`
- 错误日志: `logread | grep smart-forward`
- 配置文件: `/etc/smart-forward/config.yaml`
