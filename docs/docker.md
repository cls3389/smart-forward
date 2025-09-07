# 🐳 Docker 使用说明

## 📊 镜像信息
- **基础镜像**: Alpine Linux 3.18
- **大小**: **仅 8MB** 🎯
- **架构**: AMD64/ARM64
- **功能**: 完整日志、健康检查、非root用户
- **优化**: 极致编译优化 + 最小依赖

## 🚀 快速使用

### 方式1: Host网络模式 (简单)
```bash
# 拉取镜像
docker pull ghcr.io/cls3389/smart-forward:latest

# 运行容器 (使用主机网络)
docker run -d \
  --name smart-forward \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest
```

⚠️ **注意**: Host模式可能与主机的80/443端口冲突

### 方式2: macvlan网络模式 (推荐，解决端口冲突)
```bash
# 1. 创建 macvlan 网络
docker network create -d macvlan \
  --subnet=192.168.1.0/24 \
  --gateway=192.168.1.1 \
  -o parent=eth0 \
  macvlan_network

# 2. 运行容器 (获得独立IP)
docker run -d \
  --name smart-forward \
  --network macvlan_network \
  --ip 192.168.1.100 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest
```

📝 **配置说明**:
- 将 `192.168.1.0/24` 修改为您的网络段
- 将 `192.168.1.1` 修改为您的网关
- 将 `eth0` 修改为您的网卡名称
- 将 `192.168.1.100` 修改为可用的IP地址

### 使用 Docker Compose

#### Host网络模式 (docker-compose.yml)
```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

#### macvlan网络模式 (推荐，解决端口冲突)
```bash
# 使用 macvlan 配置文件
docker-compose -f docker-compose.macvlan.yml up -d

# 查看日志
docker-compose -f docker-compose.macvlan.yml logs -f

# 停止服务
docker-compose -f docker-compose.macvlan.yml down
```

📋 **macvlan配置说明**:
- 容器获得独立IP地址 (如 192.168.1.100)
- 完全避免端口冲突问题
- 可以直接通过容器IP访问服务
- 需要修改网络配置以匹配您的环境

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
