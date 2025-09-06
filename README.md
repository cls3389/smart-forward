# Smart Forward - 智能网络转发器

[![CI](https://github.com/your-username/smart-forward/workflows/CI/badge.svg)](https://github.com/your-username/smart-forward/actions)
[![Release](https://github.com/your-username/smart-forward/workflows/Release/badge.svg)](https://github.com/your-username/smart-forward/releases)
[![Docker](https://github.com/your-username/smart-forward/workflows/Docker/badge.svg)](https://github.com/your-username/smart-forward/pkgs/container/smart-forward)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个高性能的智能网络转发器，支持 TCP、UDP、HTTP 协议，具备动态地址解析、故障转移、DNS 缓存优化等特性。

## ✨ 特性

- 🚀 **多协议支持**: TCP、UDP、HTTP 协议转发
- 🔄 **故障转移**: 自动检测和切换备用服务器
- 🌐 **动态解析**: 支持域名动态解析和 TXT 记录
- ⚡ **高性能**: 基于 Tokio 异步运行时
- 🛡️ **健康检查**: 自动健康检查和连接监控
- 🔧 **灵活配置**: YAML 配置文件，支持多规则
- 📦 **跨平台**: 支持 Windows、macOS、Linux
- 🐳 **Docker 支持**: 多架构 Docker 镜像

## 🚀 快速开始

### 下载二进制文件

从 [Releases](https://github.com/your-username/smart-forward/releases) 页面下载对应平台的二进制文件：

- **Windows**: `smart-forward-windows-x86_64.zip`
- **macOS**: `smart-forward-macos-x86_64.tar.gz` 或 `smart-forward-macos-aarch64.tar.gz`
- **Linux**: `smart-forward-linux-x86_64.tar.gz` 或 `smart-forward-linux-aarch64.tar.gz`

### 使用 Docker

```bash
# 拉取镜像
docker pull ghcr.io/your-username/smart-forward:latest

# 运行容器
docker run -d \
  --name smart-forward \
  -p 443:443 \
  -p 99:99 \
  -p 6690:6690 \
  -p 999:999 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/your-username/smart-forward:latest
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-username/smart-forward.git
cd smart-forward

# 构建
cargo build --release

# 运行
./target/release/smart-forward --config config.yaml
```

## 📋 配置说明

创建 `config.yaml` 配置文件：

```yaml
# 日志配置
logging:
  level: "info"      # debug/info/warn/error
  format: "json"     # json/text

# 网络配置
network:
  listen_addr: "0.0.0.0"

# 全局缓冲区大小
buffer_size: 8192

# 转发规则
rules:
  # HTTPS 服务
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    buffer_size: 4096
    targets:
      - "192.168.1.100:443"
      - "backup.example.com:443"

  # RDP 服务 (TCP+UDP)
  - name: "RDP"
    listen_port: 99
    buffer_size: 16384
    targets:
      - "192.168.1.200:3389"
      - "rdp.example.com"
      
  # 文件服务
  - name: "Drive"
    listen_port: 6690
    protocol: "tcp"
    buffer_size: 32768
    targets:
      - "192.168.1.300:6690"
```

## 🛠️ 开发

### 环境要求

- Rust 1.75+
- Cargo

### 本地开发

```bash
# 克隆仓库
git clone https://github.com/your-username/smart-forward.git
cd smart-forward

# 安装依赖
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 跨平台构建

```bash
# Windows PowerShell
.\build-cross-platform.ps1 -Platform all -Release

# Linux/macOS
./build-cross-platform.sh -p all -r
```

## 📊 性能特性

- **高并发**: 基于 Tokio 异步运行时，支持数万并发连接
- **低延迟**: 优化的缓冲区管理和零拷贝技术
- **内存效率**: 智能内存池和连接复用
- **CPU 优化**: 多线程负载均衡和 CPU 亲和性

## 🔧 高级配置

### 健康检查

```yaml
rules:
  - name: "WebService"
    listen_port: 80
    protocol: "tcp"
    health_check:
      enabled: true
      interval: 30s
      timeout: 5s
      path: "/health"
    targets:
      - "web1.example.com:80"
      - "web2.example.com:80"
```

### DNS 配置

```yaml
dns:
  cache_ttl: 300s
  timeout: 5s
  retries: 3
  nameservers:
    - "8.8.8.8"
    - "1.1.1.1"
```

### 日志配置

```yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward.log"
  max_size: "100MB"
  max_files: 5
```

## 🐳 Docker 部署

### Docker Compose

```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/your-username/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    ports:
      - "443:443"
      - "99:99"
      - "6690:6690"
      - "999:999"
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "./logs:/app/logs"
    environment:
      - RUST_LOG=info
      - TZ=Asia/Shanghai
    healthcheck:
      test: ["/usr/local/bin/smart-forward", "--validate-config"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: smart-forward
spec:
  replicas: 2
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
        image: ghcr.io/your-username/smart-forward:latest
        ports:
        - containerPort: 443
        - containerPort: 99
        - containerPort: 6690
        - containerPort: 999
        volumeMounts:
        - name: config
          mountPath: /app/config.yaml
          subPath: config.yaml
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "128Mi"
            cpu: "200m"
      volumes:
      - name: config
        configMap:
          name: smart-forward-config
```

## 📈 监控和指标

### 内置指标

- 连接数统计
- 流量统计
- 错误率监控
- 延迟统计

### Prometheus 集成

```yaml
metrics:
  enabled: true
  port: 9090
  path: "/metrics"
```

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详细信息。

## 🙏 致谢

- [Tokio](https://tokio.rs/) - 异步运行时
- [Serde](https://serde.rs/) - 序列化框架
- [Clap](https://clap.rs/) - 命令行参数解析
- [Trust DNS](https://github.com/bluejekyll/trust-dns) - DNS 解析

## 📞 支持

- 📧 邮箱: cls3389@example.com
- 🐛 问题: [GitHub Issues](https://github.com/cls3389/smart-forward/issues)
- 💬 讨论: [GitHub Discussions](https://github.com/cls3389/smart-forward/discussions)

---

⭐ 如果这个项目对您有帮助，请给它一个星标！