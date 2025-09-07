# Docker 镜像说明

## 📊 镜像信息

| 属性 | 值 |
|------|-----|
| 基础镜像 | Alpine Linux 3.18 |
| 预计大小 | ~15MB |
| 功能完整性 | ✅ 完整功能 |
| 日志支持 | ✅ 完整日志 |
| 健康检查 | ✅ 支持 |
| 安全运行 | ✅ 非root用户 |

## 🎯 使用方法

### 拉取并运行
```bash
# 拉取镜像
docker pull ghcr.io/your-repo:latest

# 运行容器
docker run -d \
  --name smart-forward \
  -p 443:443 -p 99:99 -p 6690:6690 -p 999:999 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/your-repo:latest
```

**优势：**
- ✅ 体积小 (~15MB)
- ✅ 完整日志支持
- ✅ 健康检查
- ✅ 非root用户运行
- ✅ 包含时区支持
- ✅ 多架构支持 (AMD64/ARM64)

## 📝 配置示例

### Alpine 版本配置
```yaml
# config.yaml
logging:
  level: "info"
  format: "text"  # 或 "json"

network:
  listen_addr: "0.0.0.0"

rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "your-server:443"
```

### Distroless 版本配置
```bash
# 通过环境变量配置
docker run -e RUST_LOG=info \
  -e CONFIG='{"logging":{"level":"info"},"rules":[{"name":"HTTPS","listen_port":443,"protocol":"tcp","targets":["your-server:443"]}]}' \
  ghcr.io/your-repo:distroless
```

## 🚀 部署命令

### 使用 Docker Compose
```yaml
version: '3.8'
services:
  smart-forward:
    image: ghcr.io/your-repo:alpine
    container_name: smart-forward
    restart: unless-stopped
    network_mode: host
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
    environment:
      - RUST_LOG=info
      - TZ=Asia/Shanghai
```

### 使用 Docker Run
```bash
# Alpine 版本 (推荐)
docker run -d \
  --name smart-forward \
  --restart unless-stopped \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  -e RUST_LOG=info \
  -e TZ=Asia/Shanghai \
  ghcr.io/your-repo:alpine

# Distroless 版本 (最小)
docker run -d \
  --name smart-forward \
  --restart unless-stopped \
  --network host \
  -e RUST_LOG=info \
  ghcr.io/your-repo:distroless
```
