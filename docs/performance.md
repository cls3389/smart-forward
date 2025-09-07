# ⚡ 性能优化指南

## 🎯 **性能概览**

Smart Forward 基于 Rust 异步网络处理，具有出色的性能表现：

| 指标 | 性能表现 | 说明 |
|------|----------|------|
| **并发连接** | 10,000+ | 单实例支持万级并发 |
| **吞吐量** | 1GB/s+ | 高速网络转发 |
| **延迟** | <1ms | 极低转发延迟 |
| **内存占用** | <50MB | 极低资源消耗 |
| **CPU 使用** | <5% | 高效异步处理 |

---

## 🔧 **配置优化**

### **1. 网络缓冲区优化**

```yaml
network:
  buffer_size: 65536      # 64KB - 高吞吐量场景
  # buffer_size: 8192     # 8KB - 低延迟场景
  # buffer_size: 131072   # 128KB - 大文件传输
```

#### **缓冲区大小选择指南**

| 场景 | 推荐大小 | 说明 |
|------|----------|------|
| **Web 服务** | 8KB - 16KB | 平衡延迟和吞吐量 |
| **文件传输** | 64KB - 128KB | 最大化吞吐量 |
| **游戏服务** | 4KB - 8KB | 最小化延迟 |
| **流媒体** | 32KB - 64KB | 稳定的数据流 |

### **2. 连接超时优化**

```yaml
network:
  timeout: 30             # 标准场景
  # timeout: 5            # 快速故障检测
  # timeout: 300          # 长连接场景
```

#### **超时配置策略**

| 场景 | 连接超时 | 说明 |
|------|----------|------|
| **API 服务** | 10-30秒 | 快速响应 |
| **文件下载** | 300-600秒 | 允许大文件传输 |
| **实时通信** | 5-10秒 | 快速故障检测 |
| **批处理** | 600-1800秒 | 长时间处理 |

### **3. 负载均衡优化**

```yaml
rules:
  - name: "高性能转发"
    listen_port: 443
    protocol: "tcp"
    load_balance: "least_connections"  # 最优性能
    targets:
      - "server1.example.com:443"
      - "server2.example.com:443"
      - "server3.example.com:443"
    health_check:
      enabled: true
      interval: 10        # 快速检测
      timeout: 3          # 短超时
      retries: 1          # 快速故障转移
```

#### **负载均衡算法性能对比**

| 算法 | CPU 开销 | 内存开销 | 适用场景 |
|------|----------|----------|----------|
| `round_robin` | 最低 | 最低 | 均匀负载 |
| `random` | 低 | 低 | 简单场景 |
| `least_connections` | 中等 | 中等 | 不均匀负载 |

---

## 🚀 **系统级优化**

### **1. Linux 内核参数优化**

创建 `/etc/sysctl.d/99-smart-forward.conf`：

```ini
# 网络性能优化
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.core.rmem_default = 65536
net.core.wmem_default = 65536
net.ipv4.tcp_rmem = 4096 65536 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728

# 连接数优化
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 65535

# TCP 优化
net.ipv4.tcp_congestion_control = bbr
net.ipv4.tcp_slow_start_after_idle = 0
net.ipv4.tcp_tw_reuse = 1

# 文件描述符限制
fs.file-max = 1000000
```

应用配置：
```bash
sudo sysctl -p /etc/sysctl.d/99-smart-forward.conf
```

### **2. 文件描述符限制**

编辑 `/etc/security/limits.conf`：

```ini
# Smart Forward 用户限制
smart-forward soft nofile 65536
smart-forward hard nofile 65536
smart-forward soft nproc 4096
smart-forward hard nproc 4096
```

### **3. systemd 服务优化**

更新 `/etc/systemd/system/smart-forward.service`：

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
Restart=always
RestartSec=1

# 性能优化
LimitNOFILE=65536
LimitNPROC=4096
OOMScoreAdjust=-100

# CPU 亲和性 (可选)
CPUAffinity=0-3

# 内存优化
MemoryAccounting=true
MemoryMax=1G

[Install]
WantedBy=multi-user.target
```

---

## 🐳 **Docker 性能优化**

### **1. 优化的 Docker 配置**

```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/cls3389/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    
    # 网络优化
    network_mode: host
    
    # 资源限制和预留
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '1.0'
        reservations:
          memory: 256M
          cpus: '0.5'
    
    # 性能相关环境变量
    environment:
      - RUST_LOG=info
      - RUST_BACKTRACE=0        # 禁用回溯提高性能
    
    # 卷挂载优化
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "logs:/app/logs"        # 使用命名卷
    
    # 健康检查优化
    healthcheck:
      test: ["CMD", "/usr/local/bin/smart-forward", "--validate-config"]
      interval: 30s
      timeout: 5s
      retries: 2
      start_period: 5s
    
    # 安全和性能
    security_opt:
      - no-new-privileges:true
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m

volumes:
  logs:
    driver: local
```

### **2. Docker 运行时优化**

```bash
# 使用高性能运行时
docker run -d \
  --name smart-forward \
  --network host \
  --restart unless-stopped \
  --memory 512m \
  --cpus 1.0 \
  --oom-kill-disable \
  --security-opt no-new-privileges:true \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/cls3389/smart-forward:latest
```

---

## 📊 **性能监控**

### **1. 系统监控指标**

#### **关键指标**
```bash
# CPU 使用率
top -p $(pidof smart-forward)

# 内存使用
ps aux | grep smart-forward

# 网络连接数
ss -tuln | grep smart-forward
netstat -an | grep :443 | wc -l

# 文件描述符使用
lsof -p $(pidof smart-forward) | wc -l
```

#### **网络性能监控**
```bash
# 网络吞吐量
iftop -i eth0

# 连接状态统计
ss -s

# TCP 连接详情
ss -tuln
```

### **2. 应用层监控**

#### **日志监控**
```yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward/performance.log"
```

#### **性能日志分析**
```bash
# 连接数统计
grep "connection_count" /var/log/smart-forward/performance.log | tail -100

# 延迟统计
grep "latency" /var/log/smart-forward/performance.log | awk '{print $5}' | sort -n

# 错误率统计
grep "ERROR" /var/log/smart-forward/performance.log | wc -l
```

---

## 🎯 **性能调优实践**

### **1. 高并发场景优化**

```yaml
# 配置示例: 支持 10,000+ 并发连接
network:
  listen_addr: "0.0.0.0"
  buffer_size: 32768      # 32KB 缓冲区
  timeout: 60             # 适中超时

rules:
  - name: "高并发服务"
    listen_port: 443
    protocol: "tcp"
    load_balance: "round_robin"  # 最低开销
    targets:
      - "backend1.example.com:443"
      - "backend2.example.com:443"
      - "backend3.example.com:443"
      - "backend4.example.com:443"
    health_check:
      enabled: true
      interval: 30          # 降低检查频率
      timeout: 5
      retries: 2
```

### **2. 高吞吐量场景优化**

```yaml
# 配置示例: 最大化数据传输速度
network:
  listen_addr: "0.0.0.0"
  buffer_size: 131072     # 128KB 大缓冲区
  timeout: 300            # 长超时支持大文件

rules:
  - name: "高吞吐量服务"
    listen_port: 8080
    protocol: "tcp"
    load_balance: "least_connections"
    targets:
      - "storage1.example.com:8080"
      - "storage2.example.com:8080"
    health_check:
      enabled: false        # 禁用以减少开销
```

### **3. 低延迟场景优化**

```yaml
# 配置示例: 最小化转发延迟
network:
  listen_addr: "0.0.0.0"
  buffer_size: 4096       # 小缓冲区减少延迟
  timeout: 5              # 短超时快速故障检测

rules:
  - name: "低延迟服务"
    listen_port: 25565
    protocol: "tcp"
    load_balance: "round_robin"
    targets:
      - "game1.example.com:25565"
      - "game2.example.com:25565"
    health_check:
      enabled: true
      interval: 5           # 频繁检查
      timeout: 1            # 极短超时
      retries: 1            # 快速故障转移
```

---

## 🔬 **性能测试**

### **1. 压力测试工具**

#### **wrk - HTTP 压力测试**
```bash
# 安装 wrk
sudo apt install wrk

# 测试 HTTP 转发性能
wrk -t12 -c400 -d30s http://localhost:8080/

# 结果分析
# Requests/sec: 每秒请求数
# Latency: 延迟分布
# Transfer/sec: 传输速度
```

#### **iperf3 - 网络吞吐量测试**
```bash
# 服务端
iperf3 -s -p 5201

# 客户端测试
iperf3 -c localhost -p 5201 -t 30 -P 10

# 通过 Smart Forward 测试
iperf3 -c smart-forward-host -p 443 -t 30
```

#### **netperf - 网络性能测试**
```bash
# 安装 netperf
sudo apt install netperf

# TCP 流测试
netperf -H smart-forward-host -p 443 -t TCP_STREAM

# 延迟测试
netperf -H smart-forward-host -p 443 -t TCP_RR
```

### **2. 性能基准测试**

#### **并发连接测试**
```bash
#!/bin/bash
# concurrent_test.sh

for i in {1..1000}; do
    nc -z localhost 443 &
done
wait

echo "并发连接测试完成"
```

#### **吞吐量测试**
```bash
#!/bin/bash
# throughput_test.sh

# 生成测试文件
dd if=/dev/zero of=test_1gb.bin bs=1M count=1024

# 测试上传速度
time curl -X POST -T test_1gb.bin http://localhost:8080/upload

# 测试下载速度
time curl -o /dev/null http://localhost:8080/test_1gb.bin
```

---

## 📈 **性能调优检查清单**

### **配置优化**
- [ ] ✅ 根据场景调整缓冲区大小
- [ ] ✅ 设置合适的连接超时
- [ ] ✅ 选择最优负载均衡算法
- [ ] ✅ 配置适当的健康检查频率

### **系统优化**
- [ ] ✅ 调整内核网络参数
- [ ] ✅ 增加文件描述符限制
- [ ] ✅ 启用 BBR 拥塞控制
- [ ] ✅ 优化 systemd 服务配置

### **监控设置**
- [ ] ✅ 部署性能监控工具
- [ ] ✅ 设置关键指标告警
- [ ] ✅ 定期进行性能测试
- [ ] ✅ 分析性能瓶颈

### **硬件考虑**
- [ ] ✅ 使用 SSD 存储
- [ ] ✅ 配置足够的内存
- [ ] ✅ 选择高性能网卡
- [ ] ✅ 考虑 CPU 核心数

---

## 🎯 **性能优化最佳实践**

1. **渐进式优化**: 从默认配置开始，逐步调优
2. **基准测试**: 每次修改后进行性能测试
3. **监控驱动**: 基于监控数据进行优化决策
4. **场景适配**: 根据具体使用场景选择优化策略
5. **定期评估**: 定期重新评估性能配置的有效性

通过遵循这些优化指南，Smart Forward 可以在各种场景下提供卓越的性能表现！⚡
