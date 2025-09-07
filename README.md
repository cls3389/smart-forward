# Smart Forward - 智能网络转发器

[![CI](https://github.com/cls3389/smart-forward/workflows/发布构建/badge.svg)](https://github.com/cls3389/smart-forward/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

一个高性能的智能网络转发器，支持 TCP、UDP、HTTP 协议转发，具有动态地址解析、故障转移和健康检查功能。

## ✨ 功能特性

- 🚀 **多协议支持**: TCP、UDP、HTTP 协议转发
- 🔄 **智能故障转移**: 自动检测目标服务器状态并切换
- 🌐 **动态地址解析**: 支持 A/AAAA 记录和 TXT 记录解析
- ⚡ **高性能**: 基于 Rust 异步网络处理
- 🔧 **灵活配置**: YAML 配置文件，支持多规则配置
- 🐳 **Docker 支持**: 提供多架构 Docker 镜像
- 📊 **健康检查**: 自动监控目标服务器状态
- 🔒 **安全可靠**: 支持 HTTPS 自动跳转

## 📦 下载

### 最新版本 (v1.3.0)
- **Windows x86_64**: [smart-forward-windows-x86_64.zip](https://github.com/cls3389/smart-forward/releases/latest)
- **macOS Intel**: [smart-forward-macos-x86_64.tar.gz](https://github.com/cls3389/smart-forward/releases/latest)
- **macOS Apple Silicon**: [smart-forward-macos-aarch64.tar.gz](https://github.com/cls3389/smart-forward/releases/latest)
- **Linux x86_64 (GNU)**: [smart-forward-linux-x86_64-gnu.tar.gz](https://github.com/cls3389/smart-forward/releases/latest)
- **Linux ARM64 (GNU)**: [smart-forward-linux-aarch64-gnu.tar.gz](https://github.com/cls3389/smart-forward/releases/latest)
- **Linux x86_64 (musl)**: [smart-forward-linux-x86_64-musl.tar.gz](https://github.com/cls3389/smart-forward/releases/latest) 🔥
- **Linux ARM64 (musl)**: [smart-forward-linux-aarch64-musl.tar.gz](https://github.com/cls3389/smart-forward/releases/latest) 🔥

### 🔄 版本选择指南
- **musl版本** 🔥: 静态链接，零依赖，推荐用于容器和跨发行版部署
- **GNU版本**: 动态链接，性能稍好，适用于有glibc的传统Linux系统

### 🚀 一键安装 (Linux)
```bash
# 默认安装 musl 版本 (推荐)
curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash

# 安装 GNU 版本
BINARY_TYPE=gnu curl -fsSL https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/linux-install.sh | bash
```
详细说明请查看 [Linux安装指南](docs/linux-installation.md)

### 🐳 Docker 镜像 (Alpine 3.18 + musl - 仅15MB)
```bash
# 拉取最新镜像 (支持 AMD64/ARM64)
docker pull ghcr.io/cls3389/smart-forward:latest

# 运行容器 (使用 host 网络模式)
docker run -d \
  --name smart-forward \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest

# 或使用 Docker Compose
docker-compose up -d
```

**镜像特性**:
- 🏔️ **Alpine Linux 3.18** - 极致小体积
- 📦 **仅 15MB** - musl静态链接优化，比传统镜像小70%
- 🔐 **root运行** - 支持特权端口绑定  
- 🏥 **健康检查** - 自动监控
- 🌍 **多架构** - AMD64/ARM64原生支持
- ⚡ **零依赖** - musl静态链接，适用所有环境

## 🚀 快速开始

### 1. 下载并解压
```bash
# Linux/macOS
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64.tar.gz
tar -xzf smart-forward-linux-x86_64.tar.gz

# Windows
# 下载 smart-forward-windows-x86_64.zip 并解压
```

### 2. 配置
复制 `config.yaml.example` 为 `config.yaml` 并根据需求修改：

```yaml
# 日志配置
logging:
  level: "info"
  format: "json"

# 网络配置
network:
  listen_addr: "0.0.0.0"

# 转发规则
rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    buffer_size: 4096
    targets:
      - "192.168.1.100:443"        # 主服务器
      - "backup.example.com:443"   # 备用服务器
      
  - name: "RDP"
    listen_port: 99
    # 支持 TCP+UDP 双协议
    buffer_size: 16384
    targets:
      - "192.168.1.200:3389"
      - "rdp.example.com"
```

### 3. 运行
```bash
# Linux/macOS
./smart-forward

# Windows
smart-forward.exe

# Docker
docker run -d --name smart-forward -p 443:443 -v $(pwd)/config.yaml:/app/config.yaml:ro ghcr.io/cls3389/smart-forward:latest
```

## 🔧 配置说明

### 基本配置
- `logging.level`: 日志级别 (debug/info/warn/error)
- `logging.format`: 日志格式 (json/text)
- `network.listen_addr`: 监听地址 (默认: 0.0.0.0)
- `buffer_size`: 全局缓冲区大小 (字节)

### 转发规则
每个规则包含以下字段：
- `name`: 规则名称
- `listen_port`: 监听端口
- `protocol`: 协议类型 (tcp/udp，不指定则支持双协议)
- `buffer_size`: 缓冲区大小 (可选，覆盖全局设置)
- `targets`: 目标服务器列表 (按优先级排序)

### 目标地址格式
- `IP:端口`: 直接 IP 地址
- `域名:端口`: 域名解析
- `域名`: 使用默认端口 (与监听端口相同)

## 🛠️ 开发

### 环境要求
- Rust 1.70+
- Cargo

### 构建
```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 跨平台构建
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-unknown-linux-gnu
```

### 测试
```bash
# 运行测试
cargo test

# 代码检查
cargo clippy -- -D warnings

# 格式化检查
cargo fmt -- --check
```

### Docker 构建
```bash
# 构建镜像
docker build -t smart-forward .

# 构建多架构镜像
docker buildx build --platform linux/amd64,linux/arm64 -t smart-forward .
```

## 📋 使用场景

### 1. 故障转移
当主服务器故障时，自动切换到备用服务器。支持按优先级顺序切换。

### 2. 服务代理
作为中间代理，处理网络请求转发。

### 3. 端口映射
将外部端口映射到内部服务器的不同端口。

### 4. 协议转换
支持不同协议之间的转发，如 TCP 到 UDP。

### 5. 高可用性
通过多目标配置和健康检查，提供高可用的网络服务。

## 🔍 故障排除

### 常见问题

1. **端口被占用**
   ```bash
   # 检查端口占用
   netstat -tulpn | grep :443
   
   # 修改配置文件中的端口
   ```

2. **目标服务器不可达**
   - 检查网络连接
   - 验证目标地址和端口
   - 查看日志输出

3. **权限问题**
   ```bash
   # Linux 需要 root 权限绑定特权端口
   sudo ./smart-forward
   ```

### 日志分析
```bash
# 查看详细日志
RUST_LOG=debug ./smart-forward

# 查看 JSON 格式日志
tail -f logs/smart-forward.log | jq .
```

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Tokio](https://tokio.rs/) - 异步运行时
- [Serde](https://serde.rs/) - 序列化框架
- [Clap](https://clap.rs/) - 命令行参数解析
- [Tracing](https://tracing.rs/) - 日志和追踪

---

**注意**: 本项目仅供学习和研究使用，请遵守相关法律法规。