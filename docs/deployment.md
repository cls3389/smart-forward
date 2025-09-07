# 🚀 部署指南

## 🎯 **部署方式概览**

Smart Forward 支持多种部署方式，适应不同的使用场景：

| 部署方式 | 适用场景 | 难度 | 推荐指数 |
|----------|----------|------|----------|
| 🐳 **Docker** | 生产环境、容器化部署 | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 📦 **二进制文件** | 简单部署、测试环境 | ⭐ | ⭐⭐⭐⭐ |
| ☁️ **云服务** | 云原生、自动扩缩容 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 🏠 **本地开发** | 开发测试 | ⭐ | ⭐⭐⭐ |

---

## 🐳 **Docker 部署 (推荐)**

### **快速开始**

```bash
# 1. 下载配置文件模板
curl -O https://raw.githubusercontent.com/cls3389/smart-forward/main/config.yaml.example
mv config.yaml.example config.yaml

# 2. 编辑配置文件
vim config.yaml

# 3. 运行容器
docker run -d \
  --name smart-forward \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  -v $(pwd)/logs:/app/logs \
  ghcr.io/cls3389/smart-forward:latest
```

### **Docker Compose 部署**

创建 `docker-compose.yml`：

```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    network_mode: host
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "./logs:/app/logs"
    environment:
      - RUST_LOG=info
      - TZ=Asia/Shanghai
    healthcheck:
      test: ["CMD", "/usr/local/bin/smart-forward", "--validate-config"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
```

启动服务：
```bash
# 启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止
docker-compose down
```

### **生产环境 Docker 配置**

#### **Host网络模式** (简单但可能有端口冲突)
```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    
    # 网络配置
    network_mode: host
    
    # 资源限制
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
        reservations:
          memory: 128M
          cpus: '0.1'
    
    # 卷挂载
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "./logs:/app/logs"
      - "/etc/ssl/certs:/etc/ssl/certs:ro"  # SSL 证书
    
    # 环境变量
    environment:
      - RUST_LOG=info
      - TZ=Asia/Shanghai
      - SF_LOG_LEVEL=info
    
    # 健康检查
    healthcheck:
      test: ["CMD", "/usr/local/bin/smart-forward", "--validate-config"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    
    # 日志配置
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  # 可选: 日志收集
  fluentd:
    image: fluent/fluentd:latest
    volumes:
      - "./logs:/fluentd/log"
    depends_on:
      - smart-forward
```

#### **macvlan网络模式** (推荐，解决端口冲突)
```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    
    # 使用 macvlan 网络，容器获得独立IP
    networks:
      macvlan_network:
        ipv4_address: 192.168.1.100  # 修改为您网络中可用的IP
    
    # 卷挂载
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "logs:/app/logs"
      - "/etc/ssl/certs:/etc/ssl/certs:ro"
    
    # 环境变量
    environment:
      - RUST_LOG=info
      - TZ=Asia/Shanghai
    
    # 健康检查
    healthcheck:
      test: ["CMD", "/usr/local/bin/smart-forward", "--validate-config"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    
    # 日志配置
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

# macvlan 网络配置
networks:
  macvlan_network:
    driver: macvlan
    driver_opts:
      parent: eth0  # 修改为您的网卡名称 (如 ens33, enp0s3 等)
    ipam:
      config:
        - subnet: 192.168.1.0/24      # 修改为您的网络段
          gateway: 192.168.1.1        # 修改为您的网关
          ip_range: 192.168.1.100/32  # 容器IP范围

volumes:
  logs:
    driver: local
```

#### **macvlan网络配置步骤**

1. **检查网络配置**
```bash
# 查看网卡名称
ip addr show

# 查看网络段
ip route show
```

2. **修改配置文件**
```bash
# 复制 macvlan 配置模板
cp docker-compose.yml docker-compose.macvlan.yml

# 编辑配置，修改以下参数:
# - parent: 您的网卡名称 (如 eth0, ens33, enp0s3)
# - subnet: 您的网络段 (如 192.168.1.0/24, 10.0.0.0/24)
# - gateway: 您的网关 (如 192.168.1.1, 10.0.0.1)
# - ipv4_address: 容器IP (确保不与其他设备冲突)
```

3. **启动服务**
```bash
# 使用 macvlan 配置启动
docker-compose -f docker-compose.macvlan.yml up -d

# 验证容器IP
docker inspect smart-forward | grep IPAddress

# 测试连接
ping 192.168.1.100  # 使用您配置的容器IP
```

#### **macvlan优势**
- ✅ **完全避免端口冲突** - 容器有独立IP
- ✅ **性能最优** - 直接网络访问，无NAT开销
- ✅ **配置简单** - 一次配置，永久解决
- ✅ **网络隔离** - 容器网络与主机分离

---

## 📦 **二进制文件部署**

### **下载和安装**

```bash
# 1. 下载对应平台的二进制文件
# Linux x86_64
wget https://github.com/cls3389/smart-forward/releases/latest/download/smart-forward-linux-x86_64.tar.gz

# 解压
tar -xzf smart-forward-linux-x86_64.tar.gz
cd smart-forward-linux-x86_64

# 2. 复制到系统路径
sudo cp smart-forward /usr/local/bin/
sudo chmod +x /usr/local/bin/smart-forward

# 3. 创建配置目录
sudo mkdir -p /etc/smart-forward
sudo cp config.yaml /etc/smart-forward/

# 4. 创建日志目录
sudo mkdir -p /var/log/smart-forward
```

### **systemd 服务配置**

创建服务文件 `/etc/systemd/system/smart-forward.service`：

```ini
[Unit]
Description=Smart Forward - 智能网络转发器
After=network.target
Wants=network.target

[Service]
Type=simple
User=smart-forward
Group=smart-forward
ExecStart=/usr/local/bin/smart-forward --config /etc/smart-forward/config.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=smart-forward

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/smart-forward

# 资源限制
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

创建用户和启动服务：

```bash
# 创建专用用户
sudo useradd -r -s /bin/false smart-forward

# 设置权限
sudo chown -R smart-forward:smart-forward /var/log/smart-forward
sudo chown smart-forward:smart-forward /etc/smart-forward/config.yaml

# 启动服务
sudo systemctl daemon-reload
sudo systemctl enable smart-forward
sudo systemctl start smart-forward

# 查看状态
sudo systemctl status smart-forward
```

---

## ☁️ **云服务部署**

### **Kubernetes 部署**

创建 `k8s-deployment.yaml`：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: smart-forward
  labels:
    app: smart-forward
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
        image: ghcr.io/cls3389/smart-forward:latest
        ports:
        - containerPort: 443
        - containerPort: 80
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
        volumeMounts:
        - name: config
          mountPath: /app/config.yaml
          subPath: config.yaml
        - name: logs
          mountPath: /app/logs
        livenessProbe:
          exec:
            command:
            - /usr/local/bin/smart-forward
            - --validate-config
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          exec:
            command:
            - /usr/local/bin/smart-forward
            - --validate-config
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: config
        configMap:
          name: smart-forward-config
      - name: logs
        emptyDir: {}

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: smart-forward-config
data:
  config.yaml: |
    logging:
      level: "info"
      format: "json"
    network:
      listen_addr: "0.0.0.0"
      buffer_size: 8192
    rules:
      - name: "HTTPS转发"
        listen_port: 443
        protocol: "tcp"
        targets:
          - "backend1.example.com:443"
          - "backend2.example.com:443"

---
apiVersion: v1
kind: Service
metadata:
  name: smart-forward-service
spec:
  selector:
    app: smart-forward
  ports:
  - name: https
    port: 443
    targetPort: 443
  - name: http
    port: 80
    targetPort: 80
  type: LoadBalancer
```

部署到 Kubernetes：

```bash
# 应用配置
kubectl apply -f k8s-deployment.yaml

# 查看状态
kubectl get pods -l app=smart-forward
kubectl get services

# 查看日志
kubectl logs -l app=smart-forward -f
```

### **AWS ECS 部署**

创建任务定义 `ecs-task-definition.json`：

```json
{
  "family": "smart-forward",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "executionRoleArn": "arn:aws:iam::ACCOUNT:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "smart-forward",
      "image": "ghcr.io/cls3389/smart-forward:latest",
      "portMappings": [
        {
          "containerPort": 443,
          "protocol": "tcp"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/smart-forward",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ]
    }
  ]
}
```

---

## 🔧 **配置管理**

### **配置文件位置**

| 部署方式 | 配置文件路径 |
|----------|--------------|
| Docker | `/app/config.yaml` |
| 系统服务 | `/etc/smart-forward/config.yaml` |
| 本地开发 | `./config.yaml` |

### **配置热重载**

```bash
# 发送 SIGHUP 信号重载配置
kill -HUP $(pidof smart-forward)

# 或使用 systemctl
sudo systemctl reload smart-forward
```

### **配置验证**

```bash
# 验证配置文件语法
smart-forward --config /path/to/config.yaml --validate

# 测试配置并显示解析结果
smart-forward --config /path/to/config.yaml --test
```

---

## 📊 **监控和日志**

### **日志配置**

```yaml
logging:
  level: "info"
  format: "json"           # 便于日志分析
  file: "/var/log/smart-forward/app.log"
```

### **Prometheus 监控**

Smart Forward 支持 Prometheus 指标导出：

```yaml
# 在配置中启用指标
metrics:
  enabled: true
  listen_addr: "0.0.0.0:9090"
  path: "/metrics"
```

### **日志聚合**

#### **ELK Stack**
```yaml
# Filebeat 配置
filebeat.inputs:
- type: log
  paths:
    - /var/log/smart-forward/*.log
  fields:
    service: smart-forward
  fields_under_root: true
```

#### **Fluentd**
```xml
<source>
  @type tail
  path /var/log/smart-forward/*.log
  pos_file /var/log/fluentd/smart-forward.log.pos
  tag smart-forward
  format json
</source>
```

---

## 🚨 **故障排查**

### **常见问题**

#### **1. 端口被占用**
```bash
# 检查端口占用
sudo netstat -tlnp | grep :443
sudo lsof -i :443

# 解决方案: 修改配置或停止冲突服务
```

#### **2. 权限问题**
```bash
# 检查文件权限
ls -la /etc/smart-forward/config.yaml

# 修复权限
sudo chown smart-forward:smart-forward /etc/smart-forward/config.yaml
sudo chmod 644 /etc/smart-forward/config.yaml
```

#### **3. 网络连接问题**
```bash
# 测试目标服务器连通性
telnet target.example.com 443
nc -zv target.example.com 443

# 检查防火墙规则
sudo iptables -L
sudo ufw status
```

### **调试模式**

```bash
# 启用调试日志
export RUST_LOG=debug
smart-forward --config config.yaml

# 或在配置文件中设置
logging:
  level: "debug"
```

---

## 🎯 **最佳实践**

### **1. 安全配置**
- 使用非 root 用户运行
- 限制文件权限 (644 for config, 755 for binary)
- 启用防火墙规则
- 定期更新镜像版本

### **2. 性能优化**
- 根据负载调整 `buffer_size`
- 使用 SSD 存储日志文件
- 配置适当的连接超时
- 监控资源使用情况

### **3. 高可用部署**
- 部署多个实例
- 使用负载均衡器
- 配置健康检查
- 实施故障转移策略

### **4. 监控告警**
- 设置关键指标监控
- 配置日志告警规则
- 定期检查服务状态
- 建立故障响应流程
