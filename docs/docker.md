# Docker 使用说明

## 🐳 镜像信息
- **基础镜像**: Alpine Linux 3.18
- **大小**: ~15MB
- **架构**: AMD64/ARM64
- **功能**: 完整日志、健康检查、非root用户

## 🚀 快速使用

### 拉取并运行
```bash
# 拉取镜像
docker pull ghcr.io/cls3389/smart-forward:latest

# 运行容器
docker run -d \
  --name smart-forward \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest
```

### 使用 Docker Compose
```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f
```

## 📝 配置示例
```yaml
# config.yaml
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
      - "your-server:443"
```
