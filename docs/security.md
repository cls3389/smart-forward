# 🛡️ 安全配置指南

## 🎯 **安全概览**

Smart Forward 提供多层安全防护，确保网络转发的安全性和可靠性：

| 安全层面 | 防护措施 | 重要性 |
|----------|----------|--------|
| **网络安全** | 防火墙、端口控制 | ⭐⭐⭐⭐⭐ |
| **访问控制** | 用户权限、文件权限 | ⭐⭐⭐⭐⭐ |
| **数据安全** | TLS/SSL、加密传输 | ⭐⭐⭐⭐ |
| **运行时安全** | 容器安全、资源限制 | ⭐⭐⭐⭐ |
| **监控审计** | 日志记录、异常检测 | ⭐⭐⭐ |

---

## 🔐 **基础安全配置**

### **1. 用户和权限管理**

#### **创建专用用户**
```bash
# 创建系统用户 (不能登录)
sudo useradd -r -s /bin/false -d /var/lib/smart-forward smart-forward

# 创建必要目录
sudo mkdir -p /var/lib/smart-forward
sudo mkdir -p /var/log/smart-forward
sudo mkdir -p /etc/smart-forward

# 设置目录权限
sudo chown smart-forward:smart-forward /var/lib/smart-forward
sudo chown smart-forward:smart-forward /var/log/smart-forward
sudo chown root:smart-forward /etc/smart-forward
```

#### **文件权限设置**
```bash
# 配置文件权限 (只读)
sudo chmod 640 /etc/smart-forward/config.yaml
sudo chown root:smart-forward /etc/smart-forward/config.yaml

# 二进制文件权限
sudo chmod 755 /usr/local/bin/smart-forward
sudo chown root:root /usr/local/bin/smart-forward

# 日志目录权限
sudo chmod 750 /var/log/smart-forward
sudo chown smart-forward:smart-forward /var/log/smart-forward
```

### **2. 网络安全配置**

#### **防火墙规则 (iptables)**
```bash
# 清空现有规则 (谨慎操作)
sudo iptables -F

# 默认策略
sudo iptables -P INPUT DROP
sudo iptables -P FORWARD DROP
sudo iptables -P OUTPUT ACCEPT

# 允许本地回环
sudo iptables -A INPUT -i lo -j ACCEPT

# 允许已建立的连接
sudo iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# 允许 SSH (修改为实际端口)
sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# 允许 Smart Forward 端口
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT

# 限制连接速率 (防 DDoS)
sudo iptables -A INPUT -p tcp --dport 443 -m limit --limit 25/minute --limit-burst 100 -j ACCEPT

# 保存规则
sudo iptables-save > /etc/iptables/rules.v4
```

#### **防火墙规则 (ufw)**
```bash
# 启用 ufw
sudo ufw enable

# 默认策略
sudo ufw default deny incoming
sudo ufw default allow outgoing

# 允许 SSH
sudo ufw allow ssh

# 允许 Smart Forward 端口
sudo ufw allow 443/tcp
sudo ufw allow 80/tcp

# 限制连接速率
sudo ufw limit 443/tcp

# 查看状态
sudo ufw status verbose
```

### **3. TLS/SSL 配置**

#### **证书管理**
```yaml
# 配置 TLS 证书 (如果支持)
tls:
  enabled: true
  cert_file: "/etc/ssl/certs/smart-forward.crt"
  key_file: "/etc/ssl/private/smart-forward.key"
  ca_file: "/etc/ssl/certs/ca-bundle.crt"
```

#### **Let's Encrypt 证书**
```bash
# 安装 certbot
sudo apt install certbot

# 获取证书
sudo certbot certonly --standalone -d your-domain.com

# 证书路径
# /etc/letsencrypt/live/your-domain.com/fullchain.pem
# /etc/letsencrypt/live/your-domain.com/privkey.pem

# 自动续期
sudo crontab -e
# 添加: 0 12 * * * /usr/bin/certbot renew --quiet
```

---

## 🐳 **Docker 安全配置**

### **1. 安全的 Docker 配置**

```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    
    # 网络安全
    network_mode: bridge  # 不使用 host 模式
    ports:
      - "443:443"
      - "80:80"
    
    # 用户安全
    user: "1000:1000"     # 非 root 用户
    
    # 文件系统安全
    read_only: true       # 只读文件系统
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
    
    # 安全选项
    security_opt:
      - no-new-privileges:true
      - apparmor:docker-default
    
    # 资源限制
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
    
    # 卷挂载 (只读)
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "logs:/app/logs"
    
    # 环境变量
    environment:
      - RUST_LOG=info
      - RUST_BACKTRACE=0  # 禁用回溯信息泄露
    
    # 健康检查
    healthcheck:
      test: ["CMD", "/usr/local/bin/smart-forward", "--validate-config"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  logs:
    driver: local
```

### **2. Docker 运行时安全**

```bash
# 使用安全选项运行
docker run -d \
  --name smart-forward \
  --user 1000:1000 \
  --read-only \
  --tmpfs /tmp:noexec,nosuid,size=100m \
  --security-opt no-new-privileges:true \
  --security-opt apparmor:docker-default \
  --memory 256m \
  --cpus 0.5 \
  -p 443:443 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest
```

### **3. 容器扫描**

```bash
# 使用 Trivy 扫描镜像漏洞
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image ghcr.io/cls3389/smart-forward:latest

# 使用 Clair 扫描
docker run -d --name clair-db postgres:latest
docker run -d --name clair --link clair-db:postgres \
  quay.io/coreos/clair:latest
```

---

## 🔒 **访问控制**

### **1. IP 白名单配置**

```yaml
# 配置示例: IP 访问控制
network:
  listen_addr: "0.0.0.0"
  allowed_ips:
    - "192.168.1.0/24"    # 内网段
    - "10.0.0.0/8"        # 私有网络
    - "203.0.113.0/24"    # 特定公网段
  denied_ips:
    - "0.0.0.0/0"         # 默认拒绝所有
```

### **2. 端口访问控制**

```bash
# 使用 iptables 限制源 IP
sudo iptables -A INPUT -p tcp --dport 443 -s 192.168.1.0/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j DROP

# 使用 fail2ban 防暴力破解
sudo apt install fail2ban

# 配置 /etc/fail2ban/jail.local
[smart-forward]
enabled = true
port = 443
filter = smart-forward
logpath = /var/log/smart-forward/app.log
maxretry = 5
bantime = 3600
```

### **3. 认证和授权**

```yaml
# 配置示例: 基础认证 (如果支持)
auth:
  enabled: true
  type: "basic"
  users:
    - username: "admin"
      password_hash: "$2b$12$..."  # bcrypt 哈希
    - username: "user"
      password_hash: "$2b$12$..."
```

---

## 📊 **安全监控**

### **1. 日志安全配置**

```yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/security.log"
  
  # 安全事件记录
  security_events:
    - "connection_refused"
    - "authentication_failed"
    - "rate_limit_exceeded"
    - "invalid_request"
```

### **2. 安全事件监控**

#### **日志分析脚本**
```bash
#!/bin/bash
# security_monitor.sh

LOG_FILE="/var/log/smart-forward/security.log"
ALERT_EMAIL="admin@example.com"

# 检查失败连接
FAILED_CONNECTIONS=$(grep "connection_refused" $LOG_FILE | wc -l)
if [ $FAILED_CONNECTIONS -gt 100 ]; then
    echo "警告: 检测到大量连接失败 ($FAILED_CONNECTIONS)" | mail -s "Smart Forward 安全警报" $ALERT_EMAIL
fi

# 检查异常 IP
grep "connection_refused" $LOG_FILE | awk '{print $5}' | sort | uniq -c | sort -nr | head -10
```

#### **实时监控**
```bash
# 监控实时连接
watch -n 1 'netstat -an | grep :443 | wc -l'

# 监控日志
tail -f /var/log/smart-forward/security.log | grep -E "(WARN|ERROR)"

# 监控系统资源
top -p $(pidof smart-forward)
```

### **3. 入侵检测**

#### **OSSEC 配置**
```xml
<!-- /var/ossec/etc/ossec.conf -->
<localfile>
  <log_format>json</log_format>
  <location>/var/log/smart-forward/security.log</location>
</localfile>

<rule id="100001" level="5">
  <decoded_as>json</decoded_as>
  <field name="level">ERROR</field>
  <description>Smart Forward Error</description>
</rule>
```

#### **Suricata 规则**
```bash
# /etc/suricata/rules/smart-forward.rules
alert tcp any any -> any 443 (msg:"Smart Forward Suspicious Connection"; \
  threshold: type both, track by_src, count 100, seconds 60; \
  sid:1000001; rev:1;)
```

---

## 🚨 **安全事件响应**

### **1. 事件分类**

| 事件级别 | 描述 | 响应时间 | 处理方式 |
|----------|------|----------|----------|
| **严重** | 服务中断、数据泄露 | 立即 | 紧急响应 |
| **高** | 攻击尝试、异常访问 | 15分钟 | 快速响应 |
| **中** | 配置错误、性能问题 | 1小时 | 标准响应 |
| **低** | 一般警告、信息事件 | 24小时 | 例行处理 |

### **2. 应急响应流程**

#### **发现安全事件**
```bash
# 1. 立即隔离
sudo iptables -A INPUT -s <攻击IP> -j DROP

# 2. 收集证据
sudo cp /var/log/smart-forward/security.log /tmp/incident-$(date +%Y%m%d-%H%M%S).log

# 3. 分析日志
grep <攻击IP> /var/log/smart-forward/security.log

# 4. 临时措施
sudo systemctl stop smart-forward  # 如有必要
```

#### **事后分析**
```bash
# 生成安全报告
#!/bin/bash
# security_report.sh

echo "=== Smart Forward 安全报告 ===" > security_report.txt
echo "生成时间: $(date)" >> security_report.txt
echo "" >> security_report.txt

echo "=== 连接统计 ===" >> security_report.txt
netstat -an | grep :443 | wc -l >> security_report.txt

echo "=== 错误统计 ===" >> security_report.txt
grep "ERROR" /var/log/smart-forward/security.log | wc -l >> security_report.txt

echo "=== 异常 IP ===" >> security_report.txt
grep "connection_refused" /var/log/smart-forward/security.log | \
  awk '{print $5}' | sort | uniq -c | sort -nr | head -10 >> security_report.txt
```

---

## 🔧 **安全加固**

### **1. 系统加固**

#### **内核参数优化**
```bash
# /etc/sysctl.d/99-security.conf

# 网络安全
net.ipv4.conf.all.send_redirects = 0
net.ipv4.conf.default.send_redirects = 0
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0

# SYN Flood 防护
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_max_syn_backlog = 2048
net.ipv4.tcp_synack_retries = 2
net.ipv4.tcp_syn_retries = 5

# IP 欺骗防护
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1

# 禁用 ICMP 重定向
net.ipv4.conf.all.secure_redirects = 0
net.ipv4.conf.default.secure_redirects = 0

# 记录可疑包
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.log_martians = 1
```

#### **文件系统保护**
```bash
# 挂载选项加固
# /etc/fstab
/tmp /tmp tmpfs defaults,noexec,nosuid,nodev 0 0
/var/tmp /var/tmp tmpfs defaults,noexec,nosuid,nodev 0 0

# 设置 umask
echo "umask 027" >> /etc/profile
```

### **2. 应用加固**

#### **编译时安全选项**
```toml
# Cargo.toml 安全编译选项
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

# 安全相关编译标志
[target.'cfg(unix)']
rustflags = [
    "-C", "relro-level=full",
    "-C", "control-flow-guard=yes",
    "-Z", "sanitizer=address"  # 开发时使用
]
```

#### **运行时保护**
```bash
# 启用 ASLR
echo 2 > /proc/sys/kernel/randomize_va_space

# 启用 DEP/NX
# 现代系统默认启用

# 设置 core dump 限制
echo "* soft core 0" >> /etc/security/limits.conf
echo "* hard core 0" >> /etc/security/limits.conf
```

---

## 📋 **安全检查清单**

### **部署前检查**
- [ ] ✅ 创建专用用户和组
- [ ] ✅ 设置正确的文件权限
- [ ] ✅ 配置防火墙规则
- [ ] ✅ 启用 TLS/SSL 加密
- [ ] ✅ 配置访问控制
- [ ] ✅ 设置资源限制

### **运行时检查**
- [ ] ✅ 监控异常连接
- [ ] ✅ 检查日志异常
- [ ] ✅ 验证证书有效性
- [ ] ✅ 监控资源使用
- [ ] ✅ 检查系统更新

### **定期检查**
- [ ] ✅ 安全漏洞扫描
- [ ] ✅ 配置审计
- [ ] ✅ 日志分析
- [ ] ✅ 性能监控
- [ ] ✅ 备份验证

---

## 🎯 **安全最佳实践**

1. **最小权限原则**: 只授予必要的最小权限
2. **深度防御**: 多层安全防护措施
3. **持续监控**: 实时监控和日志分析
4. **定期更新**: 及时更新系统和应用
5. **安全培训**: 团队安全意识培训
6. **事件响应**: 建立完善的应急响应流程
7. **合规检查**: 定期进行安全合规检查

通过遵循这些安全配置指南，Smart Forward 可以在各种环境中安全可靠地运行！🛡️
