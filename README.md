# Smart Forward - 智能网络转发器 v1.5.2

[![🚀 全平台发布](https://github.com/cls3389/smart-forward/actions/workflows/release.yml/badge.svg)](https://github.com/cls3389/smart-forward/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

一个高性能的智能网络转发器，支持 TCP、UDP、HTTP 协议转发，具有动态地址解析、故障转移和健康检查功能。支持用户态和内核态转发，在OpenWrt等资源受限环境下提供极致性能。

## ✨ 功能特性

- 🚀 **多协议支持**: TCP、UDP、HTTP 协议转发，默认TCP+UDP双协议监听
- 🔄 **智能故障转移**: 自动检测目标服务器状态并切换
- 🌐 **动态地址解析**: 支持 A/AAAA 记录和 TXT 记录解析
- ⚡ **内核态转发**: Linux下支持iptables/nftables内核级转发，性能提升10倍+
- 🔧 **混合模式**: 用户态健康检查 + 内核态数据转发，智能故障切换
- 🛡️ **防火墙优化**: 自动处理Firewall4优先级，避免规则冲突
- 🔧 **灵活配置**: YAML 配置文件，支持多规则配置
- 📊 **健康检查**: 自动监控目标服务器状态
- 🔒 **AutoHTTP**: 自动HTTP跳转HTTPS，智能端口检测
- 🏃 **轻量高效**: 专为路由器等资源受限环境优化

## 🚀 快速开始

### 1. 安装

#### 📦 原生系统安装 (推荐)
```bash
# 交互式安装脚本 - 自动检测Linux/OpenWrt环境
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/install.sh | bash

# 或使用 wget
wget -qO- https://raw.githubusercontent.com/cls3389/smart-forward/main/install.sh | bash
```

**💡 提示**: Docker用户请直接使用Docker命令，此脚本仅用于原生Linux/OpenWrt系统安装

**安装特性**:
- 🔍 **自动检测**: 自动识别Linux或OpenWrt环境
- 🎛️ **交互式配置**: 选择监听地址和转发规则
- 📋 **智能升级**: 保留现有配置，清理旧程序
- ⚙️ **零配置**: 默认配置即可运行，支持后续修改

### 2. 配置

编辑 `config.yaml` 配置文件（已包含完整的生产配置示例）：

```yaml
# 网络配置
network:
  listen_addrs:
    - "10.5.1.1"    # 指定具体IP避免劫持

# 转发规则
rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "192.168.5.254:443"
      - "121.40.167.222:50443"
      - "stun-443.4.ipto.top"
```

### 3. 启动

```bash
# 前台运行（推荐用于测试）
./start.sh

# 后台运行（推荐用于生产）
./start.sh daemon

# 停止服务
./stop.sh
```

**特性**:
- 🔍 **自动检测**: Linux、OpenWrt、Docker环境
- 🚀 **优先内核态**: 自动启用内核态转发，智能回退用户态
- 🌐 **IPv4/IPv6**: 支持现代混合网络环境
- ⚡ **零配置**: 开箱即用，自动优化

#### 🐳 Docker 容器部署
```bash
# 用户态转发 (推荐，跨平台兼容)
docker run -d --name smart-forward --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest

# 内核态转发 (Linux高性能，需要特权模式)
docker run -d --name smart-forward --privileged --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest \
  --kernel-mode
```

**💡 Docker vs 原生安装**:
- 🐳 **Docker**: 容器化部署，易于分发和管理
- 🖥️ **原生安装**: 直接安装到系统，性能更好，资源占用更少

#### 💾 手动下载二进制文件
[📥 GitHub Releases](https://github.com/cls3389/smart-forward/releases/latest) - 支持 Windows、macOS、Linux

#### 🟢 简单启动脚本 (小白友好)
```bash
# 快速启动
./start.sh

# 快速停止
./stop.sh

# 查看状态
ps aux | grep smart-forward
```

**💡 小白用户提示**:
- ✅ **一键启动**: `./start.sh` - 自动检查配置并启动服务
- ✅ **一键停止**: `./stop.sh` - 安全停止服务，支持强制停止
- ✅ **智能检查**: 自动检查配置文件是否存在
- ✅ **友好提示**: 彩色输出，清晰的状态信息

### 2. 运行

```bash
# 自动模式（推荐）
./smart-forward

# 强制用户态转发（跨平台兼容）
./smart-forward --user-mode

# 强制内核态转发（Linux高性能模式，需要root权限）
./smart-forward --kernel-mode

# 系统服务管理（安装后自动配置）
# OpenWrt:
# 启动服务: /etc/init.d/smart-forward start
# 查看状态: /etc/init.d/smart-forward status
# 查看日志: logread | grep smart-forward

# Linux:
# 启动服务: systemctl start smart-forward  # 或 sudo systemctl start smart-forward
# 查看状态: systemctl status smart-forward  # 或 sudo systemctl status smart-forward
# 查看日志: journalctl -u smart-forward -f  # 或 sudo journalctl -u smart-forward -f
```

## 📊 日志查看

### 简化日志查看
```bash
# 后台运行时的日志文件
tail -f /tmp/smart-forward.log          # 手动后台模式
tail -f /var/log/smart-forward.log      # Linux systemd

# 系统日志
# OpenWrt:
logread -f | grep smart-forward

# Linux:
sudo journalctl -u smart-forward -f
```

### 日志级别说明
- **INFO**: 正常运行信息（启动、服务状态、健康检查等）
- **WARN**: 警告信息（端口占用、目标异常等）
- **ERROR**: 错误信息（配置文件错误、权限不足等）
- **DEBUG**: 调试信息（详细的内部状态变化）

### 运行模式识别
- **内核态转发**: 日志显示 `🚀 内核态转发模式`，无端口监听，有nftables规则
- **用户态转发**: 日志显示端口监听，无nftables规则

## 📚 文档

所有配置和故障排除信息已整合到本README中，包括：
- ⚙️ **配置说明** - 完整的配置选项和示例
- 🔧 **故障排除** - 常见问题解决方案
- 📊 **日志查看** - 详细的日志分析方法

## 📁 项目结构

```
smart-forward/
├── 📁 src/                    # 🦀 Rust 源代码
├── 📁 docker/                 # 🐳 Docker 配置文件 (独立部署使用)
├── 📄 install.sh              # 🚀 原生系统安装脚本 (Linux/OpenWrt)
├── 📄 start.sh                # 🟢 简单启动脚本 (小白友好)
├── 📄 stop.sh                 # 🔴 简单停止脚本 (小白友好)
├── 📄 config.yaml            # ⚙️  主配置文件 (生产配置)
├── 📄 config.yaml.example     # 🎯 配置文件示例
├── 📄 README.md               # 📖 完整文档
└── 🏗️ Cargo.toml              # 📦 Rust 项目配置
```

**文件说明**:
- ✅ **原生安装脚本** - 专门用于Linux/OpenWrt系统的原生安装
- ✅ **简单启动脚本** - 小白友好的快速启动/停止工具
- ✅ **核心配置文件** - 包含完整生产配置示例
- ✅ **统一文档** - README包含所有必要信息
- ✅ **用户友好** - 提供多种部署选择，照顾不同用户群体

## 📈 版本更新

### v1.5.2 (2025-09-20) 🚀 **用户友好 + 简单启动**
🔥 **小白友好 + 多种部署选择**

⚡ **核心改进**：
- **简单启动脚本** - `./start.sh` 和 `./stop.sh` 照顾小白用户
- **原生安装脚本** - 专门用于Linux/OpenWrt系统的原生安装
- **Docker环境检测** - 自动识别Docker容器环境并给出指导
- **部署方式分离** - 原生安装和Docker部署各司其职

🎛️ **交互特性**：
- **网络接口检测** - 自动识别可用网络接口和IP地址
- **监听地址配置** - 选择具体IP或监听所有接口(0.0.0.0)
- **转发规则配置** - 交互式添加HTTP、HTTPS、RDP等转发规则
- **配置预览** - 安装前显示配置内容供确认

🛡️ **安装特性**：
- **跨平台兼容** - 自动检测并适配Linux/OpenWrt环境
- **配置保护** - 升级时自动备份用户配置文件
- **服务管理** - 自动配置systemd/procd服务
- **路径统一** - 解决二进制文件路径不一致问题

🎯 **部署选择**：
- 🟢 **简单启动**: `./start.sh` - 小白友好的快速启动
- 🖥️ **原生安装**: `install.sh` - 适合生产环境的高性能安装
- 🐳 **Docker部署**: `docker run` - 适合开发测试的容器化部署
- 📦 **手动下载**: GitHub Releases - 灵活部署，支持更多平台

### v1.5.0 (2025-09-20) 🚀 **内核态转发重大更新**
🔥 **革命性性能提升 - 内核态转发支持**

⚡ **核心功能**：
- **内核态转发** - Linux下支持iptables/nftables，性能提升10倍+
- **混合架构** - 用户态健康检查 + 内核态数据转发
- **智能模式** - 自动尝试内核态，失败回退用户态
- **防火墙优化** - 自动处理Firewall4优先级，避免规则冲突

🛡️ **防火墙支持**：
- **nftables** - 优先级-150 (高于Firewall4默认-100)
- **iptables** - 插入到链首位置，确保优先执行
- **自动检测** - 智能选择最佳防火墙后端

🎯 **OpenWrt专项优化**：
- **一键安装脚本** - 自动检测架构并安装
- **procd服务管理** - 完整的OpenWrt服务集成
- **资源优化** - 专为路由器性能优化

## 📊 性能对比

| 转发模式 | 延迟 | 吞吐量 | CPU占用 | 内存占用 | 适用场景 |
|---------|------|--------|---------|----------|----------|
| **内核态转发** | < 0.1ms | 10Gbps+ | < 5% | < 10MB | 高性能生产环境 |
| **用户态转发** | 1-2ms | 1Gbps | 10-20% | 20-50MB | 开发测试环境 |

> 🚀 **内核态转发优势**：数据包直接在内核空间处理，避免用户态/内核态切换开销，特别适合OpenWrt等资源受限环境。

## 🎯 特色功能

### AutoHTTP 自动跳转
当配置了443端口但没有80端口时，自动启用HTTP→HTTPS跳转：
```
✅ 检测到HTTPS配置但无HTTP配置，自动启用HTTP跳转服务
✅ HTTP监听器绑定到: 0.0.0.0:80
✅ HTTP转发器启动成功: AutoHTTP
```

### TCP+UDP 双协议支持
默认情况下，未指定协议的规则同时监听TCP和UDP：
```
✅ TCP监听器 RDP_TCP 绑定成功: 0.0.0.0:99
✅ UDP监听器绑定成功: 0.0.0.0:99
```

### 内核态转发 (Linux)
高性能内核级数据转发，支持智能故障切换：
```bash
# 自动模式 (推荐)
./smart-forward

# 手动指定防火墙后端
sudo ./smart-forward --kernel-mode --firewall-backend nftables
sudo ./smart-forward --kernel-mode --firewall-backend iptables

# 查看内核规则
sudo nft list table inet smart_forward  # nftables
sudo iptables -t nat -L SMART_FORWARD_PREROUTING  # iptables
```

### 智能故障转移
按优先级自动切换目标服务器：
```yaml
targets:
  - "primary.example.com:443"    # 优先级1
  - "backup.example.com:443"     # 优先级2  
  - "fallback.example.com:443"   # 优先级3
```

## 🔧 开发构建

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆仓库
git clone https://github.com/cls3389/smart-forward.git
cd smart-forward

# 编译
cargo build --release

# 运行
./target/release/smart-forward -c config.yaml
```

## 🤝 贡献

欢迎贡献代码！请直接提交 Pull Request 或 Issue。

## ⚙️ 配置说明

### 基础配置
```yaml
# 日志配置
logging:
  level: "info"           # 日志级别: trace, debug, info, warn, error
  format: "text"          # 日志格式: text (OpenWrt推荐), json (Linux推荐)

# 网络配置
network:
  listen_addrs:
    - "10.5.1.1"    # 指定监听地址，避免劫持所有请求
                     # 设置0.0.0.0会监听所有接口，请谨慎使用

# 缓冲区大小 (仅用户态模式有效，内核态模式忽略)
buffer_size: 8192

# 全局动态更新配置
dynamic_update:
  check_interval: 5       # 健康检查间隔 (秒)
  connection_timeout: 2   # 连接超时 (秒)
  auto_reconnect: true    # 自动重连

# 转发规则
rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"        # tcp, udp, 或 ["tcp", "udp"]
    buffer_size: 4096      # 规则级缓冲区大小
    targets:
      - "192.168.1.100:443"  # 内网服务器 (最高优先级)
      - "backup.example.com:443"  # 外网备用
    dynamic_update:
      check_interval: 5
      connection_timeout: 2
      auto_reconnect: true
```

### 高级配置
```yaml
# 多协议转发
rules:
  - name: "RDP"
    listen_port: 3389
    protocol: ["tcp", "udp"]  # 同时支持TCP和UDP
    targets:
      - "192.168.1.10:3389"

# TXT记录解析 (动态IP)
rules:
  - name: "Dynamic"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "dynamic.example.com"  # 自动解析TXT记录
```

## 🔧 故障排除

### 常见问题

#### 1. 内核态转发失败
**症状**: 日志显示 `内核态转发初始化失败`
```bash
# 检查防火墙后端
sudo nft --version  # nftables
sudo iptables --version  # iptables

# 检查权限
sudo ./smart-forward --kernel-mode

# 强制用户态模式
./smart-forward --user-mode
```

#### 2. 端口被占用
**症状**: `Address already in use`
```bash
# 查看端口占用
sudo netstat -tulpn | grep :443
sudo lsof -i :443

# 停止冲突服务
sudo systemctl stop nginx  # 示例
```

#### 3. 健康检查失败
**症状**: 所有目标显示异常
```bash
# 手动测试连接
telnet target.example.com 443
nc -zv target.example.com 443

# 检查DNS解析
nslookup target.example.com
dig target.example.com TXT
```

#### 4. 配置文件错误
**症状**: `配置文件解析失败`
```bash
# 验证YAML语法
./start.sh                    # 启动时会自动验证配置
./smart-forward --validate-config -c config.yaml

# 使用示例配置
cp config.yaml.example config.yaml
```

### 日志分析
```bash
# 实时查看日志
# Linux:
sudo journalctl -u smart-forward -f

# OpenWrt:
logread -f | grep smart-forward

# 查看启动日志
# Linux:
sudo journalctl -u smart-forward --since "1 hour ago"

# OpenWrt:
logread | grep smart-forward | tail -20

# 查看错误和警告
logread | grep smart-forward | grep -E 'ERROR|WARN|错误'
```

### 性能监控
```bash
# 检查运行状态
# OpenWrt:
ps w | grep smart-forward
nft list table inet smart_forward | grep dnat | wc -l  # 规则数量

# Linux:
systemctl status smart-forward
journalctl -u smart-forward --since "1 hour ago" | grep -E "INFO|ERROR"
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**🚀 立即开始**: `curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/install.sh | bash`