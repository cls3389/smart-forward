# 智能网络转发器

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

> 🚀 基于 Rust 的高性能智能网络转发器，支持多协议转发、动态地址更新和智能故障转移

## 📚 目录

- [🚀 快速开始](#快速开始)
- [⚙️ 配置指南](#配置指南)
- [🛠️ 使用场景](#使用场景)
- [📊 部署指南](#部署指南)
- [🔧 故障排除](#故障排除)
- [📝 API 文档](docs/API.md)
- [👾 开发指南](docs/Development.md)
- [❓ 常见问题](docs/FAQ.md)

## ✨ 特性

- ✅ **多协议支持**: TCP、UDP、HTTP/HTTPS 转发
- 🔄 **智能故障转移**: 自动检测目标健康状态
- 🌐 **动态地址更新**: 支持域名解析和 TXT 记录
- ⚡ **高性能**: 基于 Tokio 异步框架
- 📝 **灵活配置**: YAML 配置文件，支持热重载
- 🛡️ **稳定可靠**: 专注核心功能，避免过度复杂化

---

## 快速开始

### 系统要求

- Windows 10/11 或 Linux 系统
- Rust 1.70+ (用于编译)

### 安装运行

1. **克隆仓库**
```bash
git clone <repository-url>
cd rust转发
```

2. **编译项目**
```bash
cargo build --release
```

3. **配置服务**
编辑 `config.yaml` 文件，配置您的转发规则

4. **运行服务**
```bash
cargo run --release
```

## 配置指南

### 基础配置结构

```yaml
# 日志配置
logging:
  level: "info"        # 日志级别: debug, info, warn, error
  format: "json"       # 日志格式: json, text

# 网络配置
network:
  listen_addr: "0.0.0.0"  # 监听地址

# 全局缓冲区大小 (16KB)
buffer_size: 16384

# 全局动态更新配置
dynamic_update:
  check_interval: 30      # 检查间隔（秒）
  connection_timeout: 300 # 连接超时（秒）
  auto_reconnect: true    # 自动重连
  health_check_interval: 60 # 健康检查间隔（秒）

# 转发规则
rules:
  - name: "规则名称"
    listen_port: 443
    protocol: "tcp"    # 或 "udp", "http"
    protocols: ["tcp", "udp"]  # 多协议支持
    buffer_size: 8192  # 专用缓冲区大小
    targets:
      - "192.168.1.100:443"
      - "backup.example.com:443"
    dynamic_update:
      enabled: true
      check_interval: 30
```

### 目标地址格式

转发器支持多种目标地址格式：

1. **IP:端口格式**
```yaml
targets:
  - "192.168.1.100:443"
  - "10.0.0.1:8080"
```

2. **域名:端口格式**
```yaml
targets:
  - "example.com:443"
  - "backup.example.com:8080"
```

3. **纯域名格式** (通过 TXT 记录解析)
```yaml
targets:
  - "service.example.com"  # 需要配置 TXT 记录为 "IP:PORT"
```

### 协议配置

#### TCP 转发
```yaml
- name: "HTTPS转发"
  listen_port: 443
  protocol: "tcp"
  targets:
    - "192.168.1.100:443"
```

#### UDP 转发
```yaml
- name: "DNS转发"
  listen_port: 53
  protocol: "udp"
  targets:
    - "8.8.8.8:53"
```

#### 多协议转发
```yaml
- name: "RDP转发"
  listen_port: 3389
  protocols: ["tcp", "udp"]  # 同时支持 TCP 和 UDP
  targets:
    - "192.168.1.100:3389"
```

#### HTTP 重定向
```yaml
- name: "HTTP重定向"
  listen_port: 80
  protocol: "http"
  # HTTP 协议会自动重定向到 HTTPS
```

## 功能详解

### 智能故障转移

转发器会定期检查目标服务器的健康状态，并根据以下策略选择最佳目标：

1. **健康状态检查**: 每60秒检查一次目标连接状态
2. **失败阈值**: 连续3次失败后标记为不健康
3. **智能选择**: 综合考虑健康状态、响应延迟和失败次数
4. **自动恢复**: 不健康的目标会定期重新检查并自动恢复

### 动态地址更新

支持动态解析目标地址，适用于动态 IP 环境：

- **域名解析**: 自动解析 A/AAAA 记录
- **TXT 记录**: 支持通过 TXT 记录配置 IP:PORT
- **定期更新**: 可配置检查间隔
- **DNS 服务器**: 使用阿里云 DNS (223.5.5.5, 223.6.6.6)

### 缓冲区配置

根据不同应用场景优化缓冲区大小：

- **HTTPS**: 8KB (适合加密流量)
- **RDP**: 32KB (优化远程桌面性能)
- **网盘**: 32KB (适合大文件传输)
- **默认**: 16KB (通用场景)

## 使用场景

### 1. 群晖 NAS 远程访问

配置 HTTPS 和网盘服务转发，解决群晖设备连接问题：

```yaml
rules:
  - name: "群晖HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "192.168.5.254:443"
      - "backup-nas.example.com:443"
    
  - name: "群晖Drive"
    listen_port: 6690
    protocol: "tcp"
    targets:
      - "192.168.5.3:6690"
      - "drive.backup.com"
```

### 2. 远程桌面转发

优化 RDP 连接性能：

```yaml
rules:
  - name: "RDP转发"
    listen_port: 3389
    protocols: ["tcp", "udp"]
    buffer_size: 32768
    targets:
      - "192.168.1.100:3389"
      - "backup-pc.example.com:3389"
```

### 3. Web 服务负载均衡

```yaml
rules:
  - name: "Web负载均衡"
    listen_port: 8080
    protocol: "tcp"
    targets:
      - "web1.internal:8080"
      - "web2.internal:8080"
      - "web3.internal:8080"
```

## 部署指南

### Windows 服务部署

1. **编译发布版本**
```cmd
cargo build --release
```

2. **创建服务目录**
```cmd
mkdir C:\ForwarderService
copy target\release\rust转发.exe C:\ForwarderService\
copy config.yaml C:\ForwarderService\
```

3. **使用 NSSM 创建 Windows 服务**
```cmd
nssm install ForwarderService C:\ForwarderService\rust转发.exe
nssm set ForwarderService AppDirectory C:\ForwarderService
nssm start ForwarderService
```

### Linux Systemd 部署

1. **创建服务文件**
```ini
# /etc/systemd/system/forwarder.service
[Unit]
Description=Smart Network Forwarder
After=network.target

[Service]
Type=simple
User=forwarder
WorkingDirectory=/opt/forwarder
ExecStart=/opt/forwarder/rust转发
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

2. **启用服务**
```bash
sudo systemctl enable forwarder
sudo systemctl start forwarder
```

### Docker 部署

1. **创建 Dockerfile**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust转发 /usr/local/bin/forwarder
COPY config.yaml /etc/forwarder/config.yaml
WORKDIR /etc/forwarder
EXPOSE 80 443
CMD ["forwarder"]
```

2. **构建和运行**
```bash
docker build -t smart-forwarder .
docker run -d --name forwarder -p 80:80 -p 443:443 smart-forwarder
```

## 故障排除

### 常见问题

#### 1. 端口绑定失败
**错误**: `Address already in use`
**解决方案**: 
- 检查端口是否被其他程序占用
- Windows: `netstat -ano | findstr :端口号`
- Linux: `lsof -i :端口号`

#### 2. 目标连接失败
**错误**: `Connection refused`
**解决方案**:
- 检查目标服务器是否运行
- 验证防火墙设置
- 确认网络连通性

#### 3. DNS 解析失败
**错误**: `DNS resolution failed`
**解决方案**:
- 检查 DNS 配置
- 验证域名是否正确
- 测试网络连接

#### 4. 配置文件错误
**错误**: `Configuration parse error`
**解决方案**:
- 检查 YAML 语法
- 验证配置项是否完整
- 使用 YAML 验证器检查格式

### 日志分析

启用详细日志进行问题诊断：

```yaml
logging:
  level: "debug"  # 启用详细日志
  format: "text"  # 便于阅读的格式
```

关键日志信息：
- `启动TCP转发器`: 转发器启动成功
- `接受客户端连接`: 新连接建立
- `规则 X 更新目标地址`: 动态地址更新
- `健康检查失败`: 目标服务器不可用

### 性能优化

#### 缓冲区调优
根据应用类型调整缓冲区大小：
- 低延迟应用: 4KB-8KB
- 高吞吐应用: 32KB-64KB
- 平衡配置: 16KB

#### 检查间隔优化
根据网络稳定性调整检查间隔：
- 稳定网络: 60秒
- 不稳定网络: 15-30秒
- 关键应用: 10秒

## 开发指南

### 项目结构

```
src/
├── main.rs           # 程序入口
├── config.rs         # 配置管理
├── common.rs         # 公共模块
├── utils.rs          # 工具函数
└── forwarder/        # 转发器模块
    ├── mod.rs        # 模块入口
    ├── tcp.rs        # TCP转发器
    ├── udp.rs        # UDP转发器
    ├── http.rs       # HTTP转发器
    └── unified.rs    # 统一转发器
```

### 添加新协议

1. **创建协议转发器**
```rust
// src/forwarder/your_protocol.rs
use async_trait::async_trait;
use super::Forwarder;

pub struct YourProtocolForwarder {
    // 字段定义
}

#[async_trait]
impl Forwarder for YourProtocolForwarder {
    async fn start(&mut self) -> Result<()> {
        // 实现启动逻辑
    }
    
    async fn stop(&mut self) {
        // 实现停止逻辑
    }
    
    // 其他必需方法...
}
```

2. **注册到配置系统**
```rust
// src/config.rs
match rule.protocol.as_str() {
    "tcp" => { /* TCP 处理 */ }
    "udp" => { /* UDP 处理 */ }
    "your_protocol" => { /* 您的协议处理 */ }
    _ => return Err(/* 错误处理 */),
}
```

### 贡献指南

1. Fork 项目
2. 创建功能分支
3. 编写测试
4. 提交 Pull Request

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 编写必要的文档注释
- 保持函数简洁（< 50 行）

## 常见问题 FAQ

### Q: 为什么选择 Rust？
A: Rust 提供了内存安全、高性能和并发支持，非常适合网络代理应用。

### Q: 支持 IPv6 吗？
A: 是的，转发器自动支持 IPv4 和 IPv6 地址。

### Q: 可以转发多少个连接？
A: 理论上只受系统资源限制，通常可以处理数千个并发连接。

### Q: 配置更改需要重启吗？
A: 是的，当前版本需要重启才能应用配置更改。

### Q: 支持加密传输吗？
A: 转发器工作在传输层，支持透明转发 TLS/SSL 加密流量。

### Q: 如何监控转发器状态？
A: 可以通过日志文件监控，或者集成外部监控系统。

### Q: 支持负载均衡算法吗？
A: 目前支持基于健康状态和延迟的智能选择，未来可能添加更多算法。

## 版本历史

### v1.0.0
- 初始版本发布
- 支持 TCP、UDP、HTTP 转发
- 基础故障转移功能

### v1.1.0
- 添加动态地址更新
- 改进健康检查机制
- 优化错误处理

### v1.2.0 (当前版本)
- 简化代码架构
- 去除复杂监控功能
- 专注核心转发功能
- 改进智能故障转移算法
- 支持多协议同时转发

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 联系方式

如有问题或建议，请：
- 提交 Issue
- 发起 Pull Request
- 联系项目维护者

---

*最后更新: 2025-08-28*