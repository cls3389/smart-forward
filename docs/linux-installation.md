# Linux 安装指南

## 📋 目录
1. [系统要求](#系统要求)
2. [一键安装](#一键安装)
3. [版本选择](#版本选择)
4. [手动安装](#手动安装)
5. [管理命令](#管理命令)
6. [配置说明](#配置说明)
7. [故障排除](#故障排除)

---

## 🔧 系统要求

### 支持的发行版
- ✅ **Ubuntu** 14.04+ (推荐 20.04+)
- ✅ **Debian** 8+ (推荐 10+)
- ✅ **CentOS** 7+ (推荐 8+)
- ✅ **RHEL** 7+ (推荐 8+)
- ✅ **Fedora** 25+ (推荐 35+)
- ✅ **Arch Linux** (滚动发布)
- ✅ **Alpine Linux** 3.10+ (musl版本)
- ✅ **其他发行版** (musl版本通用兼容)

### 硬件要求
- **CPU**: x86_64 或 ARM64 (aarch64)
- **内存**: 至少 64MB RAM
- **存储**: 至少 20MB 可用空间
- **权限**: sudo 或 root 权限

### 软件依赖
- `wget` 或 `curl` (下载)
- `tar` (解压)
- `systemd` (可选，用于系统服务)

---

## 🚀 一键安装

### 默认安装 (musl版本，推荐)

```bash
# 下载并运行安装脚本
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 或者使用 wget
wget -qO- https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash
```

### 指定版本类型安装

```bash
# 安装 musl 版本 (推荐)
BINARY_TYPE=musl curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 安装 GNU 版本 (性能稍好)
BINARY_TYPE=gnu curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash
```

### 本地安装

```bash
# 1. 下载脚本
wget https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh
chmod +x linux-install.sh

# 2. 运行安装 (默认 musl 版本)
./linux-install.sh

# 3. 或指定版本
BINARY_TYPE=gnu ./linux-install.sh
```

---

## 🔄 版本选择

### musl版本 🔥 (推荐)
- **特点**: 静态链接，零运行时依赖
- **优势**: 
  - ✅ 兼容所有Linux发行版
  - ✅ 容器化部署友好
  - ✅ 跨发行版迁移无问题
  - ✅ 嵌入式系统支持
- **文件大小**: ~12MB
- **推荐场景**: 生产环境、Docker、跨平台部署

### GNU版本
- **特点**: 动态链接，依赖系统glibc
- **优势**:
  - ⚡ 启动速度稍快
  - 💾 内存使用稍低
  - 🔗 与系统库集成
- **要求**: glibc 2.17+ (CentOS 7+/Ubuntu 14.04+)
- **文件大小**: ~8MB
- **推荐场景**: 单一发行版长期部署

### 版本对比表

| 特性 | musl版本 🔥 | GNU版本 |
|-----|-------------|---------|
| **兼容性** | 所有Linux发行版 | 需要glibc 2.17+ |
| **依赖** | 无依赖 | 依赖系统glibc |
| **文件大小** | ~12MB | ~8MB |
| **启动速度** | 良好 | 稍快 |
| **内存使用** | 正常 | 稍低 |
| **容器部署** | 完美 | 需要基础镜像 |
| **跨发行版** | 完美 | 可能有问题 |

---

## 🛠️ 手动安装

如果自动脚本无法使用，可以手动安装：

### 1. 下载二进制文件

```bash
# 下载 musl 版本 (推荐)
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64-musl.tar.gz

# 或下载 GNU 版本
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64-gnu.tar.gz

# ARM64 架构
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-aarch64-musl.tar.gz
```

### 2. 解压安装

```bash
# 解压
tar -xzf smart-forward-linux-*.tar.gz

# 安装到系统路径
sudo cp smart-forward /usr/local/bin/
sudo chmod +x /usr/local/bin/smart-forward

# 验证安装
smart-forward --version
```

### 3. 创建配置目录

```bash
# 创建配置和日志目录
sudo mkdir -p /etc/smart-forward
sudo mkdir -p /var/log/smart-forward

# 创建基本配置文件
sudo tee /etc/smart-forward/config.yaml > /dev/null << 'EOF'
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"

buffer_size: 8192

rules:
  - name: "示例规则"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "example.com:80"
EOF
```

### 4. 创建systemd服务 (可选)

```bash
sudo tee /etc/systemd/system/smart-forward.service > /dev/null << 'EOF'
[Unit]
Description=Smart Forward - 智能网络转发器
After=network.target

[Service]
Type=simple
User=nobody
Group=nogroup
ExecStart=/usr/local/bin/smart-forward --config /etc/smart-forward/config.yaml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 重载并启用服务
sudo systemctl daemon-reload
sudo systemctl enable smart-forward
```

---

## 🎛️ 管理命令

安装完成后，使用 `smart-forward-ctl` 管理服务：

### 服务管理

```bash
# 启动服务
smart-forward-ctl start

# 停止服务  
smart-forward-ctl stop

# 重启服务
smart-forward-ctl restart

# 查看状态
smart-forward-ctl status

# 查看实时日志
smart-forward-ctl logs
```

### 配置管理

```bash
# 编辑配置文件
smart-forward-ctl config

# 查看版本信息
smart-forward-ctl version
```

### 手动运行 (调试)

```bash
# 前台运行 (用于调试)
/usr/local/bin/smart-forward --config /etc/smart-forward/config.yaml

# 后台运行
nohup /usr/local/bin/smart-forward --config /etc/smart-forward/config.yaml &
```

---

## ⚙️ 配置说明

### 基本配置文件

配置文件位于 `/etc/smart-forward/config.yaml`:

```yaml
# 日志配置
logging:
  level: "info"        # debug, info, warn, error
  format: "text"       # text, json

# 网络配置
network:
  listen_addr: "0.0.0.0"  # 监听地址

# 缓冲区大小 (字节)
buffer_size: 8192

# 转发规则
rules:
  - name: "HTTPS转发"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "backend1.example.com:443"
      - "backend2.example.com:443"
      
  - name: "HTTP转发" 
    listen_port: 80
    protocol: "tcp"
    targets:
      - "backend.example.com:8080"
```

### 高级配置示例

```yaml
# 健康检查配置
rules:
  - name: "负载均衡"
    listen_port: 443
    protocol: "tcp"
    targets:
      - host: "server1.com"
        port: 443
        priority: 1
        health_check: true
      - host: "server2.com"  
        port: 443
        priority: 2
        health_check: true

# DNS配置
dns:
  cache_ttl: 300
  timeout: 5
  
# 限流配置 (如支持)
rate_limit:
  enabled: true
  requests_per_second: 1000
```

---

## 🔍 故障排除

### 常见问题

#### 1. 权限不足

**症状**: `Permission denied`

**解决方案**:
```bash
# 检查文件权限
ls -la /usr/local/bin/smart-forward

# 修复权限
sudo chmod +x /usr/local/bin/smart-forward

# 检查配置目录权限
sudo chown -R root:root /etc/smart-forward
sudo chmod 755 /etc/smart-forward
sudo chmod 644 /etc/smart-forward/config.yaml
```

#### 2. 端口被占用

**症状**: `Address already in use`

**解决方案**:
```bash
# 检查端口占用
sudo netstat -tlnp | grep :443

# 或使用 ss
sudo ss -tlnp | grep :443

# 修改配置文件中的端口
smart-forward-ctl config
```

#### 3. 服务启动失败

**症状**: systemd服务无法启动

**解决方案**:
```bash
# 查看详细错误
sudo journalctl -u smart-forward -f

# 检查配置文件语法
/usr/local/bin/smart-forward --config /etc/smart-forward/config.yaml --check

# 手动测试
/usr/local/bin/smart-forward --config /etc/smart-forward/config.yaml
```

#### 4. 网络连接问题

**症状**: 无法连接到目标服务器

**解决方案**:
```bash
# 测试网络连通性
ping target-server.com

# 测试端口连通性
telnet target-server.com 443

# 检查DNS解析
nslookup target-server.com

# 检查防火墙
sudo iptables -L
sudo ufw status
```

#### 5. glibc版本不兼容 (GNU版本)

**症状**: `version 'GLIBC_X.XX' not found`

**解决方案**:
```bash
# 检查系统glibc版本
ldd --version

# 如果版本过低，使用musl版本
BINARY_TYPE=musl ./linux-install.sh
```

### 日志分析

#### 查看系统日志
```bash
# systemd 日志
sudo journalctl -u smart-forward -f

# 或查看日志文件 (如果配置了)
sudo tail -f /var/log/smart-forward/smart-forward.log
```

#### 调试模式
```bash
# 启用调试日志
# 编辑配置文件，设置 logging.level: "debug"
smart-forward-ctl config

# 重启服务
smart-forward-ctl restart
```

### 性能调优

#### 系统优化
```bash
# 增加文件描述符限制
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# 优化网络参数
echo "net.core.somaxconn = 65536" | sudo tee -a /etc/sysctl.conf
echo "net.core.netdev_max_backlog = 5000" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

#### 应用优化
```yaml
# 配置文件优化
buffer_size: 65536  # 增加缓冲区

# 连接池优化 (如支持)
connection_pool:
  max_connections: 1000
  timeout: 30
```

---

## 🔄 更新升级

### 自动更新
```bash
# 重新运行安装脚本即可更新到最新版本
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash
```

### 手动更新
```bash
# 1. 停止服务
smart-forward-ctl stop

# 2. 备份配置
sudo cp /etc/smart-forward/config.yaml /etc/smart-forward/config.yaml.bak

# 3. 下载新版本
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64-musl.tar.gz

# 4. 更新二进制文件
tar -xzf smart-forward-linux-*.tar.gz
sudo cp smart-forward /usr/local/bin/
sudo chmod +x /usr/local/bin/smart-forward

# 5. 启动服务
smart-forward-ctl start
```

---

## 🗑️ 卸载

### 完整卸载
```bash
# 停止并禁用服务
sudo systemctl stop smart-forward
sudo systemctl disable smart-forward

# 删除文件
sudo rm -f /usr/local/bin/smart-forward
sudo rm -f /usr/local/bin/smart-forward-ctl  
sudo rm -f /etc/systemd/system/smart-forward.service
sudo rm -rf /etc/smart-forward
sudo rm -rf /var/log/smart-forward

# 重载systemd
sudo systemctl daemon-reload
```

---

## 📞 技术支持

### 获取帮助
1. **查看日志**: `smart-forward-ctl logs`
2. **检查配置**: `smart-forward-ctl config`
3. **版本信息**: `smart-forward-ctl version`
4. **系统状态**: `smart-forward-ctl status`

### 相关链接
- [GitHub 仓库](https://github.com/cls3389/smart-forward)
- [问题反馈](https://github.com/cls3389/smart-forward/issues)
- [版本发布](https://github.com/cls3389/smart-forward/releases)

### 报告问题
提交Issue时请提供：
- 操作系统和版本
- 使用的二进制版本 (musl/gnu)
- 配置文件内容
- 错误日志
- 复现步骤

---

## 🎯 总结

Linux安装提供了灵活的部署选择：

1. **一键安装** - 适合快速部署，自动处理所有细节
2. **版本选择** - musl(通用) vs GNU(性能)，按需选择
3. **系统集成** - systemd服务，标准化管理
4. **完整文档** - 涵盖安装、配置、故障排除

**推荐流程**:
```bash
# 1. 一键安装 (musl版本)
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 2. 编辑配置
smart-forward-ctl config

# 3. 启动服务  
smart-forward-ctl start

# 4. 查看状态
smart-forward-ctl status
```

享受智能网络转发带来的便利！🚀
