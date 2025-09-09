# 📝 使用示例和案例

## 🎯 **示例概览**

本文档提供了 Smart Forward 在各种实际场景中的配置示例和最佳实践。

| 使用场景 | 复杂度 | 适用环境 | 推荐指数 |
|----------|--------|----------|----------|
| **Web 服务负载均衡** | ⭐⭐ | 生产环境 | ⭐⭐⭐⭐⭐ |
| **API 网关** | ⭐⭐⭐ | 微服务架构 | ⭐⭐⭐⭐⭐ |
| **游戏服务器转发** | ⭐⭐ | 游戏平台 | ⭐⭐⭐⭐ |
| **数据库连接池** | ⭐⭐⭐ | 企业应用 | ⭐⭐⭐⭐ |
| **CDN 边缘节点** | ⭐⭐⭐⭐ | 内容分发 | ⭐⭐⭐⭐⭐ |

---

## 🌐 **场景1: Web 服务负载均衡**

### **需求描述**
- 3台 Web 服务器提供 HTTPS 服务
- 需要负载均衡和故障转移
- 支持健康检查

### **配置示例**

```yaml
# config.yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/web-lb.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 16384
  timeout: 60

rules:
  # HTTPS 负载均衡
  - name: "Web服务负载均衡"
    listen_port: 443
    protocol: "tcp"
    load_balance: "round_robin"
    targets:
      - "web1.example.com:443"
      - "web2.example.com:443"
      - "web3.example.com:443"
    health_check:
      enabled: true
      interval: 30
      timeout: 5
      retries: 3

  # HTTP 重定向到 HTTPS
  - name: "HTTP重定向"
    listen_port: 80
    protocol: "http"
    targets:
      - "web1.example.com:80"
      - "web2.example.com:80"
      - "web3.example.com:80"
```

### **部署脚本**

```bash
#!/bin/bash
# deploy-web-lb.sh

# 1. 创建配置目录
sudo mkdir -p /etc/smart-forward
sudo mkdir -p /var/log/smart-forward

# 2. 复制配置文件
sudo cp config.yaml /etc/smart-forward/

# 3. 设置权限
sudo chown root:smart-forward /etc/smart-forward/config.yaml
sudo chmod 640 /etc/smart-forward/config.yaml

# 4. 启动服务
sudo systemctl start smart-forward
sudo systemctl enable smart-forward

# 5. 验证配置
curl -I https://localhost
```

### **监控脚本**

```bash
#!/bin/bash
# monitor-web-lb.sh

LOG_FILE="/var/log/smart-forward/web-lb.log"

echo "=== Web 负载均衡监控报告 ==="
echo "时间: $(date)"
echo ""

# 连接统计
echo "当前 HTTPS 连接数: $(netstat -an | grep :443 | grep ESTABLISHED | wc -l)"
echo "当前 HTTP 连接数: $(netstat -an | grep :80 | grep ESTABLISHED | wc -l)"

# 健康检查状态
echo ""
echo "=== 健康检查状态 ==="
grep "health_check" $LOG_FILE | tail -10

# 错误统计
echo ""
echo "=== 最近错误 ==="
grep "ERROR" $LOG_FILE | tail -5
```

---

## 🚀 **场景2: API 网关**

### **需求描述**
- 多个微服务 API 统一入口
- 基于路径的路由转发
- API 限流和认证

### **配置示例**

```yaml
# config.yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/api-gateway.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 8192
  timeout: 30

rules:
  # 用户服务 API
  - name: "用户服务API"
    listen_port: 8080
    protocol: "http"
    path_prefix: "/api/users"
    load_balance: "least_connections"
    targets:
      - "user-service-1:8080"
      - "user-service-2:8080"
    health_check:
      enabled: true
      interval: 15
      timeout: 3
      retries: 2
      path: "/health"
    rate_limit:
      enabled: true
      requests_per_minute: 1000

  # 订单服务 API
  - name: "订单服务API"
    listen_port: 8080
    protocol: "http"
    path_prefix: "/api/orders"
    load_balance: "round_robin"
    targets:
      - "order-service-1:8080"
      - "order-service-2:8080"
    health_check:
      enabled: true
      interval: 15
      timeout: 3
      retries: 2
      path: "/health"

  # 支付服务 API
  - name: "支付服务API"
    listen_port: 8080
    protocol: "http"
    path_prefix: "/api/payments"
    load_balance: "least_connections"
    targets:
      - "payment-service-1:8080"
      - "payment-service-2:8080"
    health_check:
      enabled: true
      interval: 10
      timeout: 2
      retries: 1
      path: "/health"
    rate_limit:
      enabled: true
      requests_per_minute: 500  # 支付接口限流更严格
```

### **Docker Compose 部署**

```yaml
# docker-compose.yml
version: '3.8'

services:
  api-gateway:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: api-gateway
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "gateway-logs:/app/logs"
    environment:
      - RUST_LOG=info
      - SF_LOG_LEVEL=info
    networks:
      - microservices
    depends_on:
      - user-service-1
      - order-service-1
      - payment-service-1

  # 用户服务
  user-service-1:
    image: user-service:latest
    networks:
      - microservices
    environment:
      - PORT=8080

  # 订单服务
  order-service-1:
    image: order-service:latest
    networks:
      - microservices
    environment:
      - PORT=8080

  # 支付服务
  payment-service-1:
    image: payment-service:latest
    networks:
      - microservices
    environment:
      - PORT=8080

networks:
  microservices:
    driver: bridge

volumes:
  gateway-logs:
```

### **API 测试脚本**

```bash
#!/bin/bash
# test-api-gateway.sh

BASE_URL="http://localhost:8080"

echo "=== API 网关测试 ==="

# 测试用户服务
echo "测试用户服务..."
curl -s -o /dev/null -w "用户API: %{http_code} - %{time_total}s\n" \
  "$BASE_URL/api/users/1"

# 测试订单服务
echo "测试订单服务..."
curl -s -o /dev/null -w "订单API: %{http_code} - %{time_total}s\n" \
  "$BASE_URL/api/orders/1"

# 测试支付服务
echo "测试支付服务..."
curl -s -o /dev/null -w "支付API: %{http_code} - %{time_total}s\n" \
  "$BASE_URL/api/payments/1"

# 压力测试
echo "进行压力测试..."
ab -n 1000 -c 10 "$BASE_URL/api/users/health"
```

---

## 🎮 **场景3: 游戏服务器转发**

### **需求描述**
- Minecraft 服务器集群
- 低延迟转发
- 玩家负载均衡

### **配置示例**

```yaml
# config.yaml
logging:
  level: "info"
  format: "text"  # 游戏场景使用文本格式更易读
  file: "/var/log/smart-forward/minecraft.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 4096    # 小缓冲区减少延迟
  timeout: 300         # 游戏连接可能较长

rules:
  # Minecraft 服务器转发
  - name: "Minecraft服务器集群"
    listen_port: 25565
    protocol: "tcp"
    load_balance: "least_connections"  # 按玩家数量均衡
    targets:
      - "mc-server-1:25565"
      - "mc-server-2:25565"
      - "mc-server-3:25565"
    health_check:
      enabled: true
      interval: 10       # 快速检测
      timeout: 2
      retries: 1
      custom_check: "minecraft_ping"

  # 游戏管理端口
  - name: "游戏管理接口"
    listen_port: 8123
    protocol: "http"
    targets:
      - "mc-server-1:8123"
      - "mc-server-2:8123"
      - "mc-server-3:8123"
```

### **启动脚本**

```bash
#!/bin/bash
# start-minecraft-proxy.sh

# 设置系统参数优化游戏性能
echo "优化系统参数..."
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
sudo sysctl -w net.ipv4.tcp_congestion_control=bbr

# 启动 Smart Forward
echo "启动游戏代理..."
smart-forward -c /etc/smart-forward/minecraft.yaml &

# 等待启动
sleep 2

# 验证服务
echo "验证服务状态..."
nc -z localhost 25565 && echo "✅ Minecraft 端口正常" || echo "❌ Minecraft 端口异常"
nc -z localhost 8123 && echo "✅ 管理端口正常" || echo "❌ 管理端口异常"

echo "游戏代理启动完成！"
```

### **玩家监控脚本**

```bash
#!/bin/bash
# monitor-minecraft.sh

LOG_FILE="/var/log/smart-forward/minecraft.log"

echo "=== Minecraft 服务器监控 ==="
echo "时间: $(date)"
echo ""

# 当前连接数
CONNECTIONS=$(netstat -an | grep :25565 | grep ESTABLISHED | wc -l)
echo "当前玩家连接数: $CONNECTIONS"

# 服务器状态
echo ""
echo "=== 服务器状态 ==="
for server in mc-server-1 mc-server-2 mc-server-3; do
    if nc -z $server 25565 2>/dev/null; then
        echo "✅ $server: 在线"
    else
        echo "❌ $server: 离线"
    fi
done

# 最近日志
echo ""
echo "=== 最近活动 ==="
tail -10 $LOG_FILE
```

---

## 🗄️ **场景4: 数据库连接池**

### **需求描述**
- MySQL 主从复制集群
- 读写分离
- 连接池管理

### **配置示例**

```yaml
# config.yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/mysql-proxy.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 8192
  timeout: 120         # 数据库连接超时较长

rules:
  # MySQL 写操作 (主库)
  - name: "MySQL主库写操作"
    listen_port: 3306
    protocol: "tcp"
    connection_type: "write"
    targets:
      - "mysql-master:3306"
    health_check:
      enabled: true
      interval: 30
      timeout: 5
      retries: 3
      custom_check: "mysql_ping"
    connection_pool:
      max_connections: 100
      idle_timeout: 300

  # MySQL 读操作 (从库)
  - name: "MySQL从库读操作"
    listen_port: 3307
    protocol: "tcp"
    connection_type: "read"
    load_balance: "round_robin"
    targets:
      - "mysql-slave-1:3306"
      - "mysql-slave-2:3306"
      - "mysql-slave-3:3306"
    health_check:
      enabled: true
      interval: 15
      timeout: 3
      retries: 2
      custom_check: "mysql_ping"
    connection_pool:
      max_connections: 200
      idle_timeout: 600

  # Redis 缓存
  - name: "Redis缓存集群"
    listen_port: 6379
    protocol: "tcp"
    load_balance: "consistent_hash"
    targets:
      - "redis-1:6379"
      - "redis-2:6379"
      - "redis-3:6379"
    health_check:
      enabled: true
      interval: 10
      timeout: 2
      retries: 1
      custom_check: "redis_ping"
```

### **数据库健康检查脚本**

```bash
#!/bin/bash
# check-db-health.sh

# MySQL 主库检查
echo "检查 MySQL 主库..."
mysql -h mysql-master -u monitor -p'password' -e "SELECT 1" >/dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ MySQL 主库: 正常"
else
    echo "❌ MySQL 主库: 异常"
fi

# MySQL 从库检查
for i in 1 2 3; do
    echo "检查 MySQL 从库 $i..."
    mysql -h mysql-slave-$i -u monitor -p'password' -e "SHOW SLAVE STATUS\G" | grep "Slave_IO_Running: Yes" >/dev/null
    if [ $? -eq 0 ]; then
        echo "✅ MySQL 从库 $i: 正常"
    else
        echo "❌ MySQL 从库 $i: 异常"
    fi
done

# Redis 检查
for i in 1 2 3; do
    echo "检查 Redis $i..."
    redis-cli -h redis-$i ping >/dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Redis $i: 正常"
    else
        echo "❌ Redis $i: 异常"
    fi
done
```

---

## 🌍 **场景5: CDN 边缘节点**

### **需求描述**
- 全球 CDN 边缘节点
- 地理位置路由
- 缓存和回源

### **配置示例**

```yaml
# config.yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/cdn-edge.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 65536     # 大缓冲区用于文件传输
  timeout: 180

rules:
  # HTTP 内容分发
  - name: "CDN边缘节点"
    listen_port: 80
    protocol: "http"
    load_balance: "geographic"  # 地理位置路由
    targets:
      - "origin-us-east.example.com:80"
      - "origin-us-west.example.com:80"
      - "origin-eu.example.com:80"
      - "origin-asia.example.com:80"
    health_check:
      enabled: true
      interval: 30
      timeout: 10
      retries: 2
      path: "/health"
    cache:
      enabled: true
      ttl: 3600           # 1小时缓存
      max_size: "10GB"
    geo_routing:
      - region: "us-east"
        target: "origin-us-east.example.com:80"
      - region: "us-west"
        target: "origin-us-west.example.com:80"
      - region: "europe"
        target: "origin-eu.example.com:80"
      - region: "asia"
        target: "origin-asia.example.com:80"

  # HTTPS 内容分发
  - name: "CDN边缘节点HTTPS"
    listen_port: 443
    protocol: "tcp"
    load_balance: "geographic"
    targets:
      - "origin-us-east.example.com:443"
      - "origin-us-west.example.com:443"
      - "origin-eu.example.com:443"
      - "origin-asia.example.com:443"
    tls:
      enabled: true
      cert_file: "/etc/ssl/certs/cdn.crt"
      key_file: "/etc/ssl/private/cdn.key"
```

### **边缘节点部署脚本**

```bash
#!/bin/bash
# deploy-cdn-edge.sh

REGION=${1:-"us-east"}
NODE_ID=${2:-"edge-001"}

echo "部署 CDN 边缘节点: $REGION-$NODE_ID"

# 1. 创建配置
cat > /etc/smart-forward/cdn-edge.yaml << EOF
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/cdn-$REGION-$NODE_ID.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 65536
  timeout: 180

rules:
  - name: "CDN-$REGION-$NODE_ID"
    listen_port: 80
    protocol: "http"
    targets:
      - "origin-$REGION.example.com:80"
    cache:
      enabled: true
      ttl: 3600
      max_size: "50GB"
EOF

# 2. 启动服务
systemctl start smart-forward
systemctl enable smart-forward

# 3. 配置监控
crontab -l | { cat; echo "*/5 * * * * /usr/local/bin/cdn-monitor.sh"; } | crontab -

echo "CDN 边缘节点部署完成！"
```

### **CDN 性能监控**

```bash
#!/bin/bash
# cdn-monitor.sh

LOG_FILE="/var/log/smart-forward/cdn-edge.log"
METRICS_FILE="/var/log/smart-forward/metrics.json"

# 收集性能指标
cat > $METRICS_FILE << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "node_id": "$HOSTNAME",
  "metrics": {
    "connections": $(netstat -an | grep :80 | grep ESTABLISHED | wc -l),
    "cache_hit_rate": $(grep "cache_hit" $LOG_FILE | tail -1000 | grep -c "hit"),
    "response_time": $(grep "response_time" $LOG_FILE | tail -100 | awk '{sum+=$5} END {print sum/NR}'),
    "bandwidth": $(iftop -t -s 1 | grep "Total" | awk '{print $2}')
  }
}
EOF

# 发送到监控中心
curl -X POST -H "Content-Type: application/json" \
  -d @$METRICS_FILE \
  "https://monitoring.example.com/api/metrics"
```

---

## 🔧 **通用工具脚本**

### **配置验证工具**

```bash
#!/bin/bash
# validate-config.sh

CONFIG_FILE=${1:-"config.yaml"}

echo "验证配置文件: $CONFIG_FILE"

# 语法检查
smart-forward -c $CONFIG_FILE --validate
if [ $? -eq 0 ]; then
    echo "✅ 配置语法正确"
else
    echo "❌ 配置语法错误"
    exit 1
fi

# 端口冲突检查
PORTS=$(grep "listen_port:" $CONFIG_FILE | awk '{print $2}' | sort)
UNIQUE_PORTS=$(echo "$PORTS" | uniq)

if [ "$(echo "$PORTS" | wc -l)" != "$(echo "$UNIQUE_PORTS" | wc -l)" ]; then
    echo "❌ 检测到端口冲突"
    exit 1
else
    echo "✅ 端口配置正确"
fi

# 目标服务器连通性检查
echo "检查目标服务器连通性..."
grep -E "^\s*-\s*\".*:.*\"" $CONFIG_FILE | sed 's/.*"\(.*\)".*/\1/' | while read target; do
    host=$(echo $target | cut -d: -f1)
    port=$(echo $target | cut -d: -f2)
    
    if nc -z $host $port 2>/dev/null; then
        echo "✅ $target: 可达"
    else
        echo "⚠️  $target: 不可达"
    fi
done

echo "配置验证完成！"
```

### **性能测试工具**

```bash
#!/bin/bash
# performance-test.sh

TARGET_HOST=${1:-"localhost"}
TARGET_PORT=${2:-"443"}
CONNECTIONS=${3:-"100"}
DURATION=${4:-"30"}

echo "=== Smart Forward 性能测试 ==="
echo "目标: $TARGET_HOST:$TARGET_PORT"
echo "并发: $CONNECTIONS"
echo "持续: ${DURATION}s"
echo ""

# HTTP 性能测试
if [ "$TARGET_PORT" == "80" ] || [ "$TARGET_PORT" == "8080" ]; then
    echo "进行 HTTP 性能测试..."
    wrk -t12 -c$CONNECTIONS -d${DURATION}s http://$TARGET_HOST:$TARGET_PORT/
fi

# TCP 连接测试
echo "进行 TCP 连接测试..."
for i in $(seq 1 $CONNECTIONS); do
    (
        exec 3<>/dev/tcp/$TARGET_HOST/$TARGET_PORT
        echo "Connection $i established"
        sleep $DURATION
        exec 3<&-
    ) &
done

wait
echo "性能测试完成！"
```

### **日志分析工具**

```bash
#!/bin/bash
# analyze-logs.sh

LOG_FILE=${1:-"/var/log/smart-forward/app.log"}
HOURS=${2:-"24"}

echo "=== Smart Forward 日志分析 ==="
echo "日志文件: $LOG_FILE"
echo "分析时间: 最近 ${HOURS} 小时"
echo ""

# 错误统计
echo "=== 错误统计 ==="
grep "ERROR" $LOG_FILE | tail -n +$(( $(wc -l < $LOG_FILE) - $(( $HOURS * 3600 )) )) | \
  awk '{print $3}' | sort | uniq -c | sort -nr

# 连接统计
echo ""
echo "=== 连接统计 ==="
grep "connection" $LOG_FILE | tail -n +$(( $(wc -l < $LOG_FILE) - $(( $HOURS * 3600 )) )) | \
  wc -l | xargs echo "总连接数:"

# 响应时间分析
echo ""
echo "=== 响应时间分析 ==="
grep "response_time" $LOG_FILE | tail -n +$(( $(wc -l < $LOG_FILE) - $(( $HOURS * 3600 )) )) | \
  awk '{print $5}' | sort -n | \
  awk '
    BEGIN { sum = 0; count = 0; }
    { 
        values[count] = $1; 
        sum += $1; 
        count++; 
    }
    END {
        if (count > 0) {
            print "平均响应时间: " sum/count "ms";
            print "最小响应时间: " values[0] "ms";
            print "最大响应时间: " values[count-1] "ms";
            if (count % 2 == 1) {
                print "中位数响应时间: " values[int(count/2)] "ms";
            } else {
                print "中位数响应时间: " (values[count/2-1] + values[count/2])/2 "ms";
            }
        }
    }'

echo ""
echo "日志分析完成！"
```

---

## 📋 **最佳实践总结**

### **配置最佳实践**
1. **根据场景选择合适的负载均衡算法**
2. **设置适当的缓冲区大小和超时时间**
3. **启用健康检查确保高可用性**
4. **使用结构化日志便于分析**

### **部署最佳实践**
1. **使用容器化部署提高可移植性**
2. **配置适当的资源限制**
3. **设置监控和告警**
4. **定期备份配置文件**

### **运维最佳实践**
1. **定期检查服务状态**
2. **分析日志发现潜在问题**
3. **进行性能测试验证配置**
4. **建立故障响应流程**

通过这些实际案例和工具脚本，您可以快速在各种场景中部署和使用 Smart Forward！🚀
