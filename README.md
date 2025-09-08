# Smart Forward - 智能网络转发器

[![🚀 全平台发布](https://github.com/cls3389/smart-forward/actions/workflows/release.yml/badge.svg)](https://github.com/cls3389/smart-forward/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

一个高性能的智能网络转发器，支持 TCP、UDP、HTTP 协议转发，具有动态地址解析、故障转移和健康检查功能。

## ✨ 功能特性

- 🚀 **多协议支持**: TCP、UDP、HTTP 协议转发，默认TCP+UDP双协议监听
- 🔄 **智能故障转移**: 自动检测目标服务器状态并切换
- 🌐 **动态地址解析**: 支持 A/AAAA 记录和 TXT 记录解析
- ⚡ **高性能**: 基于 Rust 异步网络处理
- 🔧 **灵活配置**: YAML 配置文件，支持多规则配置
- 🐳 **Docker 支持**: 提供多架构 Docker 镜像
- 📊 **健康检查**: 自动监控目标服务器状态
- 🔒 **AutoHTTP**: 自动HTTP跳转HTTPS，智能端口检测

## 🚀 快速开始

### 1. 下载

#### 📦 一键安装 (Linux)
```bash
# 推荐：musl 版本 (零依赖)
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash
```

#### 🐳 Docker 运行
```bash
docker run -d --name smart-forward --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest
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
  listen_addr: "0.0.0.0"

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
# Linux/macOS
./smart-forward

# Windows
smart-forward.exe

# Docker Compose
docker-compose up -d
```

## 📚 完整文档

- 📦 **[安装指南](INSTALLATION.md)** - 所有平台的详细安装说明
- ⚙️ **[配置指南](CONFIGURATION.md)** - 完整的配置选项和示例
- 📝 **[使用示例](EXAMPLES.md)** - 实际场景配置案例
- 🚀 **[部署指南](DEPLOYMENT.md)** - 生产环境部署最佳实践
- 🔧 **[故障排除](TROUBLESHOOTING.md)** - 常见问题解决方案

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

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**🚀 立即开始**: [安装指南](INSTALLATION.md) | [配置示例](EXAMPLES.md) | [Docker部署](DEPLOYMENT.md#docker-部署)