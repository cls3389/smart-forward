# 🔧 Smart Forward 故障排除指南

本指南涵盖了 Smart Forward 使用过程中可能遇到的所有问题及解决方案。

## 🎯 快速问题定位

### 问题分类导航

| 问题类型 | 常见症状 | 跳转链接 |
|----------|----------|----------|
| **安装问题** | 下载失败、权限错误 | [→ 安装问题](#安装问题) |
| **配置问题** | 配置文件错误、格式问题 | [→ 配置问题](#配置问题) |
| **网络问题** | 连接失败、超时 | [→ 网络问题](#网络问题) |
| **性能问题** | 延迟高、内存占用 | [→ 性能问题](#性能问题) |
| **构建问题** | 编译失败、依赖问题 | [→ 构建问题](#构建问题) |
| **部署问题** | 服务启动失败 | [→ 部署问题](#部署问题) |

---

## 🚨 安装问题

### 问题1: 下载失败或速度慢

**症状**：
```bash
curl: (28) Operation timed out
wget: unable to resolve host address
```

**解决方案**：

```bash
# 方案1: 使用代理下载
export http_proxy=http://proxy.example.com:8080
export https_proxy=http://proxy.example.com:8080
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 方案2: 使用镜像源
# GitHub Proxy
wget https://ghproxy.com/https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64-musl.tar.gz

# 方案3: 手动下载
# 访问 https://github.com/cls3389/smart-forward/releases/latest
# 手动下载对应版本
```

### 问题2: 权限被拒绝

**症状**：
```bash
Permission denied
bash: /usr/local/bin/smart-forward: Permission denied
```

**解决方案**：

```bash
# 方案1: 添加执行权限
chmod +x /usr/local/bin/smart-forward

# 方案2: 使用 sudo 运行安装脚本
sudo curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 方案3: 检查 SELinux (CentOS/RHEL)
sudo setenforce 0
sudo chmod +x /usr/local/bin/smart-forward
sudo setenforce 1
```

### 问题3: 架构不兼容

**症状**：
```bash
cannot execute binary file: Exec format error
```

**解决方案**：

```bash
# 1. 检查系统架构
uname -m
file /usr/local/bin/smart-forward

# 2. 下载对应架构版本
# x86_64 系统
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64-musl.tar.gz

# ARM64 系统
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-aarch64-musl.tar.gz

# 3. 32位系统不支持
echo "Smart Forward 仅支持 64位 系统"
```

---

## ⚙️ 配置问题

### 问题1: YAML 格式错误

**症状**：
```bash
Error: YAML parse error at line 5, column 3
Error: expected `:`, found `Identifier`
```

**解决方案**：

```bash
# 1. 检查配置文件格式
smart-forward --validate-config -c config.yaml

# 2. 常见 YAML 错误修复
# 错误示例：
rules:
- name "HTTPS"    # 缺少冒号
  listen_port 443 # 缺少冒号

# 正确格式：
rules:
  - name: "HTTPS"
    listen_port: 443

# 3. 使用在线 YAML 验证器
# https://yamlchecker.com/
# https://onlineyamltools.com/validate-yaml

# 4. 注意缩进（使用空格，不要使用制表符）
cat -A config.yaml  # 显示隐藏字符
```

### 问题2: 端口冲突

**症状**：
```bash
Error: Address already in use (os error 98)
Error: Failed to bind to 0.0.0.0:443
```

**解决方案**：

```bash
# 1. 检查端口占用
sudo netstat -tulpn | grep :443
sudo lsof -i :443
sudo ss -tulpn | grep :443

# 2. 停止占用端口的服务
sudo systemctl stop nginx
sudo systemctl stop apache2
sudo pkill -f "进程名"

# 3. 修改配置使用其他端口
# config.yaml
rules:
  - name: "HTTPS"
    listen_port: 8443  # 使用其他端口
    targets:
      - "target.example.com:443"

# 4. 使用端口转发 (iptables)
sudo iptables -t nat -A PREROUTING -p tcp --dport 443 -j REDIRECT --to-port 8443
```

### 问题3: 目标地址解析失败

**症状**：
```bash
Error: failed to resolve target: target.example.com
DNS resolution failed
```

**解决方案**：

```bash
# 1. 测试 DNS 解析
nslookup target.example.com
dig target.example.com
host target.example.com

# 2. 配置 DNS 服务器
echo "nameserver 8.8.8.8" >> /etc/resolv.conf
echo "nameserver 1.1.1.1" >> /etc/resolv.conf

# 3. 使用 IP 地址替代域名
rules:
  - name: "HTTPS"
    listen_port: 443
    targets:
      - "192.168.1.100:443"  # 直接使用 IP

# 4. 配置 hosts 文件
echo "192.168.1.100 target.example.com" >> /etc/hosts

# 5. 检查网络连接
ping target.example.com
telnet target.example.com 443
```

---

## 🌐 网络问题

### 问题1: 连接超时

**症状**：
```bash
Connection timeout after 5 seconds
Failed to connect to target server
```

**解决方案**：

```bash
# 1. 增加超时时间
# config.yaml
network:
  timeout: 60  # 增加到60秒

# 2. 测试网络连通性
ping target.example.com
traceroute target.example.com
telnet target.example.com 443

# 3. 检查防火墙设置
# Ubuntu/Debian
sudo ufw status
sudo ufw allow out 443/tcp

# CentOS/RHEL
sudo firewall-cmd --list-all
sudo firewall-cmd --permanent --add-port=443/tcp
sudo firewall-cmd --reload

# 4. 检查代理设置
unset http_proxy https_proxy
# 或配置代理
export https_proxy=http://proxy.example.com:8080
```

### 问题2: SSL/TLS 连接错误

**症状**：
```bash
SSL handshake failed
Certificate verification failed
```

**解决方案**：

```bash
# 1. 测试 SSL 连接
openssl s_client -connect target.example.com:443
curl -k https://target.example.com  # 忽略证书验证

# 2. 更新 CA 证书
# Ubuntu/Debian
sudo apt update && sudo apt install ca-certificates
sudo update-ca-certificates

# CentOS/RHEL
sudo yum update ca-certificates

# 3. 配置忽略 SSL 验证（仅测试用）
# 注意：生产环境不推荐
rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"  # 使用 TCP 模式避免 SSL 处理
    targets:
      - "target.example.com:443"
```

### 问题3: UDP 数据包丢失

**症状**：
```bash
UDP packets not forwarding
High packet loss on UDP connections
```

**解决方案**：

```bash
# 1. 增加缓冲区大小
# config.yaml
network:
  buffer_size: 65536  # 64KB

rules:
  - name: "UDP_SERVICE"
    listen_port: 5060
    protocol: "udp"
    buffer_size: 32768  # 32KB
    targets:
      - "target.example.com:5060"

# 2. 调整系统 UDP 缓冲区
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
sudo sysctl -p

# 3. 检查网络设备 MTU
ip link show
# 调整 MTU 大小
sudo ip link set dev eth0 mtu 1500

# 4. 监控 UDP 统计
cat /proc/net/udp
ss -u -a -n
```

---

## ⚡ 性能问题

### 问题1: 高延迟

**症状**：
```bash
High latency observed
Slow response times
```

**解决方案**：

```bash
# 1. 调整缓冲区大小
# config.yaml
network:
  buffer_size: 4096  # 减小缓冲区以降低延迟

rules:
  - name: "LOW_LATENCY"
    listen_port: 443
    buffer_size: 2048  # 更小的缓冲区
    targets:
      - "target.example.com:443"

# 2. 启用 TCP_NODELAY (代码中已默认启用)
# 确保禁用 Nagle 算法

# 3. 调整系统 TCP 参数
echo 'net.ipv4.tcp_nodelay = 1' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_low_latency = 1' >> /etc/sysctl.conf
sudo sysctl -p

# 4. 使用本地目标减少网络跳数
# 优先配置同网段的目标服务器

# 5. 监控延迟
ping target.example.com
mtr target.example.com
```

### 问题2: 高内存使用

**症状**：
```bash
High memory consumption
Out of memory errors
```

**解决方案**：

```bash
# 1. 减少缓冲区大小
# config.yaml
network:
  buffer_size: 4096  # 从默认 8192 减少到 4096

# 2. 限制并发连接数
# 使用系统限制
ulimit -n 1024  # 限制文件描述符数量

# 3. 监控内存使用
top -p $(pgrep smart-forward)
htop
cat /proc/$(pgrep smart-forward)/status

# 4. 配置内存限制（systemd）
# /etc/systemd/system/smart-forward.service
[Service]
MemoryLimit=128M
MemoryAccounting=true

# 5. 定期重启服务（如果有内存泄漏）
# 添加定时任务
echo "0 3 * * * systemctl restart smart-forward" | crontab -
```

### 问题3: CPU 使用率高

**症状**：
```bash
High CPU usage
System load average high
```

**解决方案**：

```bash
# 1. 检查是否有死循环或频繁重连
# 查看日志
journalctl -u smart-forward -f
tail -f /var/log/smart-forward.log

# 2. 减少健康检查频率
# config.yaml
dynamic_update:
  check_interval: 60  # 增加检查间隔

# 3. 限制 CPU 使用（systemd）
# /etc/systemd/system/smart-forward.service
[Service]
CPUQuota=50%
CPUAccounting=true

# 4. 使用 release 版本而非 debug 版本
# 确保使用 --release 编译的版本

# 5. 调整进程优先级
nice -n 10 smart-forward -c config.yaml
# 或在 systemd 中设置
Nice=10
```

---

## 🔨 构建问题

### 问题1: Rust 工具链问题

**症状**：
```bash
rustc not found
cargo not found
error: could not find Cargo.toml
```

**解决方案**：

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. 更新 Rust
rustup update

# 3. 安装特定版本
rustup install 1.70.0
rustup default 1.70.0

# 4. 检查版本
rustc --version
cargo --version

# 5. 修复路径问题
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 问题2: 依赖编译失败

**症状**：
```bash
error: linking with `cc` failed
error: could not compile `tokio`
failed to run custom build command for `openssl-sys`
```

**解决方案**：

```bash
# 1. 安装构建依赖
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel pkg-config

# Alpine
apk add build-base openssl-dev pkgconfig

# 2. 清理并重新构建
cargo clean
cargo build --release

# 3. 使用 musl 目标（静态链接）
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl --release

# 4. 设置环境变量
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig"
export OPENSSL_DIR="/usr"
```

### 问题3: 交叉编译问题

**症状**：
```bash
error: linker `aarch64-linux-gnu-gcc` not found
error: could not find native static library `ssl`
```

**解决方案**：

```bash
# 1. 安装交叉编译工具链
# Ubuntu/Debian (ARM64)
sudo apt install gcc-aarch64-linux-gnu

# 2. 设置环境变量
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# 3. 使用 Docker 交叉编译
docker run --rm -v $(pwd):/app -w /app \
  rustembedded/cross:aarch64-unknown-linux-gnu \
  cargo build --target aarch64-unknown-linux-gnu --release

# 4. 或使用 cross 工具
cargo install cross
cross build --target aarch64-unknown-linux-gnu --release
```

---

## 🚀 部署问题

### 问题1: Systemd 服务启动失败

**症状**：
```bash
Job for smart-forward.service failed
smart-forward.service: Main process exited, code=exited, status=1/FAILURE
```

**解决方案**：

```bash
# 1. 查看详细错误信息
sudo systemctl status smart-forward
sudo journalctl -u smart-forward -n 50

# 2. 检查服务文件配置
sudo systemctl cat smart-forward

# 3. 修复常见问题
# a) 检查执行文件路径
ls -la /usr/local/bin/smart-forward

# b) 检查配置文件路径
ls -la /etc/smart-forward/config.yaml

# c) 修复权限
sudo chown root:root /usr/local/bin/smart-forward
sudo chmod 755 /usr/local/bin/smart-forward

# d) 测试手动运行
sudo -u root /usr/local/bin/smart-forward -c /etc/smart-forward/config.yaml

# 4. 重新加载服务
sudo systemctl daemon-reload
sudo systemctl restart smart-forward
```

### 问题2: Docker 容器启动失败

**症状**：
```bash
docker: Error response from daemon
Container exits immediately
OCI runtime create failed
```

**解决方案**：

```bash
# 1. 查看容器日志
docker logs smart-forward

# 2. 检查镜像
docker images | grep smart-forward
docker inspect ghcr.io/cls3389/smart-forward:latest

# 3. 修复常见问题
# a) 配置文件挂载问题
ls -la $(pwd)/config.yaml
docker run --rm -v $(pwd)/config.yaml:/app/config.yaml:ro ghcr.io/cls3389/smart-forward:latest cat /app/config.yaml

# b) 网络模式问题
# 使用 host 网络模式
docker run -d --name smart-forward --network host ghcr.io/cls3389/smart-forward:latest

# c) 权限问题
docker run --rm --user root ghcr.io/cls3389/smart-forward:latest smart-forward --version

# 4. 调试运行
docker run -it --rm ghcr.io/cls3389/smart-forward:latest sh
```

### 问题3: Kubernetes 部署问题

**症状**：
```bash
Pod stuck in Pending state
CrashLoopBackOff
ImagePullBackOff
```

**解决方案**：

```bash
# 1. 查看 Pod 状态
kubectl describe pod smart-forward-xxx
kubectl logs smart-forward-xxx

# 2. 检查资源配置
kubectl get pods -o wide
kubectl top pods

# 3. 修复常见问题
# a) 镜像拉取问题
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=username \
  --docker-password=token

# b) 配置文件问题
kubectl get configmap smart-forward-config -o yaml

# c) 网络策略问题
kubectl get networkpolicies
kubectl describe networkpolicy

# 4. 调试 Pod
kubectl exec -it smart-forward-xxx -- sh
kubectl port-forward smart-forward-xxx 8080:443
```

---

## 📊 日志分析

### 启用详细日志

```bash
# 1. 配置文件中启用详细日志
# config.yaml
logging:
  level: "debug"
  format: "json"

# 2. 环境变量设置
export RUST_LOG=debug
smart-forward -c config.yaml

# 3. 特定模块日志
export RUST_LOG=smart_forward=debug,tokio=info
```

### 常见日志模式

```bash
# 成功启动
{"ts":"2024-01-01 10:00:00","level":"INFO","msg":"规则 HTTPS 启动: 0.0.0.0:443 -> 192.168.1.100:443"}

# 连接错误
{"ts":"2024-01-01 10:00:01","level":"ERROR","msg":"连接目标失败: Connection refused"}

# 健康检查
{"ts":"2024-01-01 10:00:02","level":"INFO","msg":"健康检查状态: 2 个地址健康，1 个地址异常"}

# 地址切换
{"ts":"2024-01-01 10:00:03","level":"INFO","msg":"规则 HTTPS 切换: 192.168.1.100:443 -> 192.168.1.101:443"}
```

### 日志监控脚本

```bash
#!/bin/bash
# monitor.sh - 监控关键日志事件

tail -f /var/log/smart-forward.log | while read line; do
    case "$line" in
        *"ERROR"*)
            echo "🚨 错误: $line" | mail -s "Smart Forward Error" admin@example.com
            ;;
        *"切换"*)
            echo "⚠️ 故障转移: $line"
            ;;
        *"启动成功"*)
            echo "✅ 服务启动: $line"
            ;;
    esac
done
```

---

## 🔍 调试工具

### 网络调试

```bash
# 1. 端口扫描
nmap -p 443 target.example.com
nc -zv target.example.com 443

# 2. 抓包分析
tcpdump -i eth0 port 443
wireshark

# 3. 连接测试
curl -v https://target.example.com
openssl s_client -connect target.example.com:443

# 4. 性能测试
ab -n 1000 -c 10 https://target.example.com/
wrk -t12 -c400 -d30s https://target.example.com/
```

### 系统监控

```bash
# 1. 资源使用
htop
iotop
nethogs

# 2. 连接状态
ss -tulpn
netstat -an | grep :443

# 3. 系统日志
dmesg | tail
journalctl -f
```

---

## 📞 获取帮助

如果以上解决方案无法解决您的问题：

### 1. 收集信息

```bash
# 系统信息
uname -a
cat /etc/os-release
smart-forward --version

# 配置信息
smart-forward --validate-config -c config.yaml

# 网络信息
ip addr show
ss -tulpn | grep smart-forward

# 日志信息
journalctl -u smart-forward --since "1 hour ago"
```

### 2. 提交 Issue

访问 [GitHub Issues](https://github.com/cls3389/smart-forward/issues) 并提供：

- 操作系统和版本
- Smart Forward 版本
- 完整的配置文件（移除敏感信息）
- 错误日志
- 复现步骤

### 3. 社区支持

- 💬 [GitHub Discussions](https://github.com/cls3389/smart-forward/discussions)
- 📧 邮件支持: support@smart-forward.io
- 📋 查看 [已知问题](https://github.com/cls3389/smart-forward/issues?q=is%3Aissue+label%3Abug)

---

## 📈 性能优化建议

1. **使用 musl 版本**获得更好的启动性能
2. **调整缓冲区大小**平衡内存和性能
3. **配置合理的超时时间**避免连接堆积
4. **定期清理日志文件**防止磁盘空间不足
5. **监控系统资源**及时发现瓶颈

---

**记住**：大多数问题都有解决方案，耐心调试和详细的日志分析是关键！🚀
