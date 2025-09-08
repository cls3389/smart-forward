# 📦 Smart Forward 安装指南

本指南涵盖了 Smart Forward 在所有平台的安装方法，选择适合您环境的安装方式。

## 🎯 安装方式选择

| 平台/环境 | 推荐方式 | 难度 | 特点 |
|----------|----------|------|------|
| **Linux 服务器** | 一键脚本 | ⭐ | 自动化，零配置 |
| **容器环境** | Docker | ⭐⭐ | 隔离，易管理 |
| **Windows** | 二进制文件 | ⭐ | 简单直接 |
| **macOS** | 二进制文件 | ⭐ | 原生支持 |
| **OpenWrt 路由器** | 专用脚本 | ⭐⭐⭐ | 嵌入式优化 |
| **云平台** | Docker/K8s | ⭐⭐⭐ | 云原生 |

---

## 🐧 Linux 安装

### 方式1: 一键安装脚本 (推荐)

```bash
# 默认安装 musl 版本 (推荐 - 零依赖)
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 安装 GNU 版本 (需要 glibc)
BINARY_TYPE=gnu curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 指定安装目录
INSTALL_DIR=/opt/smart-forward curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash
```

**脚本功能**：
- ✅ 自动检测系统架构 (x86_64/aarch64)
- ✅ 下载对应版本的二进制文件
- ✅ 创建systemd服务
- ✅ 设置开机自启
- ✅ 创建示例配置文件

### 方式2: 手动安装

#### 1. 下载二进制文件

```bash
# 选择适合的版本下载
VERSION="v1.3.0"

# musl 版本 (推荐 - 静态链接，零依赖)
wget https://github.com/cls3389/smart-forward/releases/download/${VERSION}/smart-forward-linux-x86_64-musl.tar.gz

# GNU 版本 (动态链接，需要 glibc 2.17+)
wget https://github.com/cls3389/smart-forward/releases/download/${VERSION}/smart-forward-linux-x86_64-gnu.tar.gz

# ARM64 架构
wget https://github.com/cls3389/smart-forward/releases/download/${VERSION}/smart-forward-linux-aarch64-musl.tar.gz
```

#### 2. 解压和安装

```bash
# 解压
tar -xzf smart-forward-linux-x86_64-musl.tar.gz

# 移动到系统目录
sudo mv smart-forward /usr/local/bin/
sudo chmod +x /usr/local/bin/smart-forward

# 创建配置目录
sudo mkdir -p /etc/smart-forward
```

#### 3. 创建systemd服务

```bash
sudo tee /etc/systemd/system/smart-forward.service > /dev/null <<EOF
[Unit]
Description=Smart Forward
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/etc/smart-forward
ExecStart=/usr/local/bin/smart-forward -c /etc/smart-forward/config.yaml
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

# 启用服务
sudo systemctl daemon-reload
sudo systemctl enable smart-forward
```

### 版本选择指南

#### musl 版本 🔥 (推荐)
- **特点**: 静态链接，完全独立
- **优势**: 零运行时依赖，完美可移植性
- **适用**: 容器部署、跨发行版、嵌入式系统
- **支持**: 所有 Linux 发行版 (包括 Alpine)

#### GNU 版本
- **特点**: 动态链接，依赖系统 glibc
- **优势**: 启动速度稍快，内存使用稍低
- **要求**: glibc 2.17+ (CentOS 7+/Ubuntu 14.04+)
- **适用**: 传统 Linux 服务器

---

## 🐳 Docker 安装

### 方式1: Docker 命令

```bash
# 拉取镜像 (支持 AMD64/ARM64)
docker pull ghcr.io/cls3389/smart-forward:latest

# 运行容器 (host 网络模式)
docker run -d \
  --name smart-forward \
  --network host \
  --restart unless-stopped \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest

# 查看日志
docker logs -f smart-forward

# 停止服务
docker stop smart-forward
```

### 方式2: Docker Compose

创建 `docker-compose.yml`：

```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    network_mode: host
    volumes:
      - ./config.yaml:/app/config.yaml:ro
    command: ["/app/smart-forward", "-c", "/app/config.yaml"]
```

运行：

```bash
# 启动服务
docker-compose up -d

# 查看状态
docker-compose ps

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

### Docker 镜像特性

| 特性 | 说明 |
|------|------|
| 🏔️ **Alpine Linux 3.18** | 极致小体积基础镜像 |
| 📦 **仅 15MB** | musl 静态链接，比传统镜像小 70% |
| 🌍 **多架构支持** | AMD64/ARM64 原生支持 |
| ⚡ **零依赖** | 静态链接，适用所有环境 |
| 🔐 **安全运行** | 支持特权端口绑定 |
| 🏥 **健康检查** | 自动监控服务状态 |

---

## 🪟 Windows 安装

### 1. 下载

从 [GitHub Releases](https://github.com/cls3389/smart-forward/releases/latest) 下载：
- `smart-forward-windows-x86_64.zip`

### 2. 安装

```powershell
# 解压到程序目录
Expand-Archive -Path smart-forward-windows-x86_64.zip -DestinationPath C:\SmartForward

# 添加到系统PATH (可选)
$env:PATH += ";C:\SmartForward"
```

### 3. 创建 Windows 服务 (可选)

使用 NSSM (Non-Sucking Service Manager)：

```powershell
# 下载 NSSM
Invoke-WebRequest -Uri "https://nssm.cc/release/nssm-2.24.zip" -OutFile "nssm.zip"
Expand-Archive -Path nssm.zip -DestinationPath .

# 安装服务
.\nssm.exe install SmartForward C:\SmartForward\smart-forward.exe
.\nssm.exe set SmartForward AppDirectory C:\SmartForward
.\nssm.exe set SmartForward AppParameters "-c config.yaml"

# 启动服务
.\nssm.exe start SmartForward
```

---

## 🍎 macOS 安装

### 1. 下载对应架构版本

```bash
# Intel Mac
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-macos-x86_64.tar.gz

# Apple Silicon (M1/M2)
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-macos-aarch64.tar.gz
```

### 2. 安装

```bash
# 解压
tar -xzf smart-forward-macos-*.tar.gz

# 移动到系统目录
sudo mv smart-forward /usr/local/bin/
sudo chmod +x /usr/local/bin/smart-forward

# 首次运行可能需要允许
sudo spctl --add /usr/local/bin/smart-forward
```

### 3. 创建 launchd 服务 (可选)

```bash
# 创建服务文件
sudo tee /Library/LaunchDaemons/com.smartforward.plist > /dev/null <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.smartforward</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/smart-forward</string>
        <string>-c</string>
        <string>/usr/local/etc/smart-forward/config.yaml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

# 加载服务
sudo launchctl load /Library/LaunchDaemons/com.smartforward.plist
```

---

## 📡 OpenWrt 安装

### 自动安装脚本

```bash
# 下载并运行安装脚本
wget -O - https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/openwrt-install.sh | sh

# 或手动下载后执行
wget https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/openwrt-install.sh
chmod +x openwrt-install.sh
./openwrt-install.sh
```

### 手动安装

#### 1. 检查架构

```bash
# 检查 CPU 架构
cat /proc/cpuinfo | grep "model name"
uname -m

# 常见架构映射：
# - mips/mipsel -> 不支持 (Rust 限制)
# - aarch64 -> ARM64
# - x86_64 -> AMD64
```

#### 2. 下载适配版本

```bash
# ARM64 架构 (推荐)
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-aarch64-musl.tar.gz

# 解压
tar -xzf smart-forward-linux-aarch64-musl.tar.gz
```

#### 3. 安装配置

```bash
# 移动文件
mv smart-forward /usr/bin/
chmod +x /usr/bin/smart-forward

# 创建配置目录
mkdir -p /etc/smart-forward

# 创建配置文件
cat > /etc/smart-forward/config.yaml << EOF
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"

rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "192.168.1.100:443"
EOF
```

#### 4. 创建启动脚本

```bash
# 创建 init.d 脚本
cat > /etc/init.d/smart-forward << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10

USE_PROCD=1
PROG=/usr/bin/smart-forward
CONF=/etc/smart-forward/config.yaml

start_service() {
    procd_open_instance
    procd_set_param command $PROG -c $CONF
    procd_set_param respawn
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
}

stop_service() {
    killall smart-forward
}
EOF

# 设置权限并启用
chmod +x /etc/init.d/smart-forward
/etc/init.d/smart-forward enable
/etc/init.d/smart-forward start
```

---

## ☁️ 云平台部署

### Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: smart-forward
spec:
  replicas: 1
  selector:
    matchLabels:
      app: smart-forward
  template:
    metadata:
      labels:
        app: smart-forward
    spec:
      containers:
      - name: smart-forward
        image: ghcr.io/cls3389/smart-forward:latest
        ports:
        - containerPort: 443
        - containerPort: 80
        volumeMounts:
        - name: config
          mountPath: /app/config.yaml
          subPath: config.yaml
      volumes:
      - name: config
        configMap:
          name: smart-forward-config
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: smart-forward-config
data:
  config.yaml: |
    logging:
      level: "info"
      format: "json"
    network:
      listen_addr: "0.0.0.0"
    rules:
      - name: "HTTPS"
        listen_port: 443
        protocol: "tcp"
        targets:
          - "backend-service:443"
```

### AWS ECS 部署

```json
{
  "family": "smart-forward",
  "networkMode": "host",
  "containerDefinitions": [
    {
      "name": "smart-forward",
      "image": "ghcr.io/cls3389/smart-forward:latest",
      "memory": 128,
      "cpu": 256,
      "essential": true,
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/smart-forward",
          "awslogs-region": "us-west-2"
        }
      }
    }
  ]
}
```

---

## 🔧 安装后配置

### 1. 创建配置文件

```bash
# 复制示例配置
cp config.yaml.example config.yaml

# 编辑配置
nano config.yaml
```

### 2. 验证安装

```bash
# 检查版本
smart-forward --version

# 验证配置
smart-forward --validate-config

# 测试运行
smart-forward -c config.yaml
```

### 3. 配置防火墙

```bash
# Ubuntu/Debian (ufw)
sudo ufw allow 443/tcp
sudo ufw allow 80/tcp

# CentOS/RHEL (firewalld)
sudo firewall-cmd --permanent --add-port=443/tcp
sudo firewall-cmd --permanent --add-port=80/tcp
sudo firewall-cmd --reload

# iptables
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
```

---

## 🚨 故障排除

### 常见问题

#### 1. 权限问题
```bash
# Linux: 绑定特权端口需要 root 权限
sudo smart-forward

# 或使用 capabilities
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/smart-forward
```

#### 2. 端口占用
```bash
# 检查端口占用
sudo netstat -tulpn | grep :443
sudo lsof -i :443

# 停止占用服务
sudo systemctl stop nginx
```

#### 3. 架构不匹配
```bash
# 检查系统架构
uname -m
file /usr/local/bin/smart-forward

# 重新下载对应架构版本
```

#### 4. DNS 解析问题
```bash
# 测试 DNS 解析
nslookup target.example.com
dig target.example.com

# 配置 DNS 服务器
echo "nameserver 8.8.8.8" >> /etc/resolv.conf
```

### 日志查看

```bash
# 直接运行查看日志
smart-forward -c config.yaml

# systemd 服务日志
journalctl -u smart-forward -f

# Docker 日志
docker logs -f smart-forward
```

---

## 📝 下一步

安装完成后，请查看：

- ⚙️ **[配置指南](CONFIGURATION.md)** - 详细配置选项
- 📝 **[使用示例](EXAMPLES.md)** - 实际场景配置
- 🚀 **[部署指南](DEPLOYMENT.md)** - 生产环境部署

---

**需要帮助？** 查看 [故障排除指南](TROUBLESHOOTING.md) 或 [提交 Issue](https://github.com/cls3389/smart-forward/issues)
