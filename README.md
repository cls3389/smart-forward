# Smart Forward - 智能网络转发器 v1.4.3

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
cd docker && docker-compose up -d
```

## 📚 完整文档

- 📦 **[安装指南](docs/INSTALLATION.md)** - 所有平台的详细安装说明
- ⚙️ **[配置指南](docs/CONFIGURATION.md)** - 完整的配置选项和示例
- 📝 **[使用示例](docs/EXAMPLES.md)** - 实际场景配置案例
- 🚀 **[部署指南](docs/DEPLOYMENT.md)** - 生产环境部署最佳实践
- 🔧 **[故障排除](docs/TROUBLESHOOTING.md)** - 常见问题解决方案

## 📁 项目结构

```
smart-forward/
├── 📁 src/              # 🦀 Rust 源代码
├── 📁 docs/             # 📚 详细文档
├── 📁 docker/           # 🐳 Docker 配置文件
├── 📁 scripts/          # 🔧 构建和安装脚本
├── 📄 README.md         # 📖 项目说明
├── ⚙️ config.yaml       # 🎯 主配置文件
└── 🏗️ Cargo.toml        # 📦 Rust 项目配置
```

## 📈 版本更新

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