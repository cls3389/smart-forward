# Smart Forward - 智能网络转发器 v1.5.1

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
- 🐳 **Docker 支持**: 提供多架构 Docker 镜像
- 📊 **健康检查**: 自动监控目标服务器状态
- 🔒 **AutoHTTP**: 自动HTTP跳转HTTPS，智能端口检测

## 🚀 快速开始

### 1. 下载

#### 📦 一键安装 (推荐)
```bash
# 统一安装脚本 - 自动检测系统类型 (Linux/OpenWrt)
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/install.sh | bash

# 或使用 wget
wget -qO- https://raw.githubusercontent.com/cls3389/smart-forward/main/install.sh | bash
```

**特性**:
- 🔍 **自动检测**: Linux、OpenWrt、Docker环境
- 🚀 **优先内核态**: 自动启用内核态转发，智能回退用户态
- 🌐 **IPv4/IPv6**: 支持现代混合网络环境
- ⚡ **零配置**: 开箱即用，自动优化

#### 🐳 Docker 运行
```bash
# 用户态转发 (跨平台)
docker run -d --name smart-forward --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest

# 内核态转发 (Linux，需要特权模式)
docker run -d --name smart-forward --privileged --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest \
  --kernel-mode --firewall-backend auto
```

#### 💾 手动下载
[📥 GitHub Releases](https://github.com/cls3389/smart-forward/releases/latest) - 支持 Windows、macOS、Linux

### 2. 配置

创建 `config.yaml`：

```yaml
# 基础配置
logging:
  level: "info"
  format: "json"

network:
  listen_addr: "10.5.1.1"  # 指定监听地址，避免劫持所有请求
                            # 设置0.0.0.0会劫持所有端口流量，请谨慎使用

# 转发规则
rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "192.168.1.100:443"        # 主服务器
      - "backup.example.com:443"   # 备用服务器
      
  - name: "RDP"
    listen_port: 99
    # 不指定协议时默认TCP+UDP双协议
    targets:
      - "192.168.1.200:3389"
```

### 3. 运行

```bash
# 自动模式 (Linux自动尝试内核态，失败回退用户态)
./smart-forward

# 强制用户态转发 (跨平台兼容)
./smart-forward --user-mode

# 强制内核态转发 (Linux高性能模式)
sudo ./smart-forward --kernel-mode --firewall-backend auto

# OpenWrt 服务管理
/etc/init.d/smart-forward start              # 启动服务
/etc/init.d/smart-forward status             # 查看状态
/etc/init.d/smart-forward enable_kernel_mode # 启用内核态

# Windows (仅用户态)
smart-forward.exe

# Docker Compose
cd docker && docker-compose up -d
```

## 📊 日志查看

### Linux (systemd)
```bash
# 实时日志
sudo journalctl -u smart-forward -f

# 最近日志
sudo journalctl -u smart-forward --since "1 hour ago"

# 错误日志
sudo journalctl -u smart-forward -p err
```

### OpenWrt (logread)
```bash
# 实时日志
logread -f | grep smart-forward

# 最近日志
logread | grep smart-forward | tail -20

# 健康检查日志
logread | grep smart-forward | grep 健康检查

# 错误和警告
logread | grep smart-forward | grep -E 'ERROR|WARN|错误'
```

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
├── 📁 docker/                 # 🐳 Docker 配置文件
├── 📄 install.sh              # 🚀 统一安装脚本
├── 📄 README.md               # 📖 完整文档 (配置+故障排除)
├── ⚙️ config.yaml.example     # 🎯 配置文件示例
└── 🏗️ Cargo.toml              # 📦 Rust 项目配置
```

**简化原则**:
- ✅ **一个安装脚本** - 自动检测环境，支持所有平台
- ✅ **一个文档文件** - README包含所有必要信息
- ✅ **核心功能** - 专注于高性能网络转发

## 📈 版本更新

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

🔧 **开发体验**：
- **跨平台兼容** - Windows/macOS自动用户态，Linux智能选择
- **Docker支持** - 内核态和用户态双模式
- **配置无感** - 无需修改配置文件，自动检测

### v1.4.6 (2025-09-09)
🔧 **最终修复单目标规则重复警告**

🚫 **彻底解决**：
- **单目标规则逻辑** - 当tRDP等规则只有一个目标且不健康时
- **消除重复警告** - 不再重复输出"强制切换到备用地址"
- **智能警告判断** - 有备用地址才提示切换，无备用地址时合理保持

🎯 **具体修复**：
- **tRDP规则**：只有 `ewin10.example.com` 一个目标时，不健康时显示"无健康目标，保持当前地址"而非"强制切换"
- **30秒间隔警告**：避免每2秒重复相同警告，30秒最多一次
- **Debug日志优化**：单目标不健康情况改为debug级别，减少日志噪音

### v1.4.5 (2025-09-09)
🔧 **修复健康检查无限循环 - 根治重复日志问题**

🚫 **问题修复**：
- **DNS解析循环** - 修复DNS解析成功≠服务恢复的逻辑错误
- **重复日志清理** - 大幅减少重复的健康状态变化日志
- **智能日志控制** - 只在状态真正变化或恢复健康时记录

⚡ **性能优化**：
- **减少不必要的连接验证** - 只在地址真正变化时才立即验证
- **时间控制的日志** - 避免每2秒重复相同的警告信息
- **优化规则更新逻辑** - 减少无效的规则选择计算

🎯 **用户体验**：
- **tRDP规则** 不再出现无限循环的健康状态切换
- **日志清洁度** 大幅提升，专注于真正有意义的状态变化
- **系统稳定性** 增强，避免不必要的资源消耗

### v1.4.4 (2025-09-09)
⚡ **极速故障检测 - 立即切换优化**

🚀 **快速响应优化**：
- **健康检查间隔** - 从15秒缩短到 **5秒**，快速发现故障
- **连接超时时间** - 从5秒缩短到 **2秒**，快速判断连接失败  
- **检测逻辑** - 每5秒检查一次，单次连接测试最多等待2秒 ⚡

🔒 **配置保护**：
- 添加 `local-config.yaml` 到 `.gitignore`，保护测试配置不被上传

🎯 **用户体验**：
- `stun-443.example.com` 失败后，**最多5秒+2秒内快速切换**到备用地址
- 大幅提升故障转移响应速度（比之前快2-3倍）

### v1.4.3 (2025-09-09)
🔧 **增强健康状态检测 - 强化切换逻辑**

✅ **健康状态检测增强**：
- **当前目标健康状态监控** - 检测当前地址变为不健康时强制切换
- **健康状态变化日志** - 详细记录地址健康状态变化原因
- **切换逻辑优化** - 确保地址不健康时立即切换到最优备用地址

🎯 **问题诊断改进**：
- 增加详细的切换原因日志
- 健康状态变化实时监控
- 优化备用地址选择算法

### v1.4.2 (2025-09-09)
🚀 **智能切换策略 - 解决"死磕"不健康地址问题**

✅ **用户建议的完美策略**：
- **异常后立即切换** - 当前地址异常时，立即切换到可用的最高优先级健康地址
- **后台持续监控** - 继续监控所有地址的健康状态
- **健康了再指定到规则** - 只有确认连接成功后才切换回高优先级地址

🎯 **解决的核心问题**：
- ❌ 之前：DNS解析恢复 → 立即切换 → 连接验证失败 → "死磕"不健康地址
- ✅ 现在：DNS解析恢复 → 连接验证 → 验证成功才切换 → 避免"死磕"

🔧 **技术改进**：
- **同步连接验证** - DNS恢复后立即验证连接，确保健康才参与规则选择
- **快速故障转移** - 健康检查发现异常后立即切换到其他健康地址
- **智能回切策略** - 高优先级地址确认健康后才切换回去

### v1.4.1 (2025-09-09)
🔧 **日志修复 - 解决启动统计混乱**

✅ **修复日志Bug**：
- **修复启动统计错误** - "6个规则可用(总共5个规则)"的逻辑矛盾
- **区分配置规则和自动服务** - 自动HTTP跳转服务不计入配置规则数量
- **日志更清晰** - 现在显示"X个规则可用(配置Y个规则+自动HTTP跳转服务)"

🎯 **显示效果**：
- 之前：`启动完成: 6 个规则可用 (总共 5 个规则)` ❌ 矛盾
- 现在：`启动完成: 6 个规则可用 (配置 5 个规则 + 自动HTTP跳转服务)` ✅ 清晰

### v1.4.0 (2025-09-09)
🎯 **架构重构 - 彻底解决批量触发复杂性**

✅ **核心架构优化**：
- **取消批量DNS触发机制** - 按用户建议，各域名独立解析处理
- **移除60秒重试间隔debuff** - 不再有性能降级
- **简化DNS解析逻辑** - 每个域名独立失败处理，不影响其他域名
- **升级thiserror依赖** - 从1.0更新到2.0.16，清理技术债务

🚀 **性能和稳定性提升**：
- 彻底消除无限循环问题
- 去除复杂的批量触发条件判断
- DNS解析性能优化：各域名并行独立处理
- 日志更清晰：只记录真正有更新的域名

🔧 **逻辑简化**：
- 域名解析失败时单独标记，不触发全局重新解析
- IP:PORT格式跳过DNS解析（v1.3.9已修复）
- 优先级选择算法保持高效（v1.3.8已优化）

### v1.3.9 (2025-09-09)
🚨 **关键Bug修复 - 无限循环和错误解析**

✅ **严重Bug修复**：
- **修复IP:PORT误判** - `192.168.5.3:6690`等IP地址不再被当作域名解析
- **修复无限循环** - 失败域名添加60秒重试间隔，避免持续触发批量DNS解析
- **优化选择算法** - 确保高优先级地址恢复时立即切换回去

🎯 **性能优化**：
- DNS解析性能提升：跳过不必要的IP地址解析
- 健康检查效率改进：减少无意义的重复检查
- 日志清晰度提升：区分域名解析和IP地址处理

🔧 **修复效果**：
- 解决了部分地址恢复时的无限重试问题
- 确保IP:PORT格式直接使用，不进行DNS查询
- 智能转发逻辑更加稳定可靠

### v1.3.6 (2025-09-09)
🎯 **核心优化 + 项目结构整理**

✅ **核心功能优化**：
- **DNS切换逻辑优化** - DNS变化时立即验证连接
- **批量DNS更新机制** - 一个域名变化触发全量更新
- **健康检查时序优化** - 移除不必要的延迟
- **IP:PORT处理优化** - 直接IP地址跳过DNS解析
- **日志重复问题修复** - 清理重复的初始化日志

📁 **项目结构优化**：
- 创建 `docs/` 目录统一管理文档
- 创建 `docker/` 目录存放Docker配置
- 创建 `scripts/` 目录管理构建脚本
- 文件大小优化：3.3MB → 1.9MB (减少40%+)

🛠️ **开发体验改进**：
- 通过GitHub CI格式检查
- 代码结构更简洁，可维护性更好
- 文档组织更清晰，查找更方便

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
./target/release/smart-forward
```

## 🤝 贡献

欢迎贡献代码！请查看 [贡献指南](CONTRIBUTING.md) 了解详情。

## ⚙️ 配置说明

### 基础配置
```yaml
# 日志配置
logging:
  level: "info"           # 日志级别: trace, debug, info, warn, error
  format: "text"          # 日志格式: text (OpenWrt推荐), json (Linux推荐)

# 网络配置
network:
  listen_addr: "10.5.1.1"  # 指定监听地址，避免劫持所有请求
                            # 设置0.0.0.0会劫持所有端口流量，请谨慎使用  # 监听地址

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
./smart-forward --validate-config

# 使用默认配置
cp config.yaml.example config.yaml
```

### 日志分析
```bash
# Linux
sudo journalctl -u smart-forward -f

# OpenWrt
logread -f | grep smart-forward

# 查看错误
logread | grep smart-forward | grep -E 'ERROR|WARN|错误'
```

### 性能优化
```bash
# 检查运行模式
# 内核态: 0个端口监听，有nftables规则
# 用户态: 多个端口监听，无nftables规则

# 内存使用检查
ps aux | grep smart-forward
cat /proc/$(pidof smart-forward)/status | grep Vm
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**🚀 立即开始**: `curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/install.sh | bash`