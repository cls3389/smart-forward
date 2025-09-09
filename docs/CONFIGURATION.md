# 📋 配置文档

## 🎯 **配置文件结构**

Smart Forward 使用 YAML 格式的配置文件，支持灵活的多规则配置。

### **基础结构**

```yaml
# 日志配置
logging:
  level: "info"           # 日志级别: trace, debug, info, warn, error
  format: "text"          # 日志格式: text, json
  file: "logs/app.log"    # 日志文件路径 (可选)

# 网络配置
network:
  listen_addr: "0.0.0.0"  # 监听地址
  buffer_size: 8192       # 缓冲区大小 (字节)
  timeout: 30             # 连接超时 (秒)

# 转发规则
rules:
  - name: "规则名称"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "target1.example.com:443"
      - "target2.example.com:443"
```

---

## 🔧 **详细配置选项**

### **1. 日志配置 (logging)**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `level` | String | `"info"` | 日志级别 |
| `format` | String | `"text"` | 输出格式 |
| `file` | String | 可选 | 日志文件路径 |

#### **日志级别说明**
- `trace`: 最详细的调试信息
- `debug`: 调试信息
- `info`: 一般信息 (推荐)
- `warn`: 警告信息
- `error`: 仅错误信息

#### **示例**
```yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward.log"
```

### **2. 网络配置 (network)**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `listen_addr` | String | `"0.0.0.0"` | 监听地址 |
| `buffer_size` | Integer | `8192` | 缓冲区大小 |
| `timeout` | Integer | `30` | 连接超时 |

#### **监听地址说明**
- `0.0.0.0`: 监听所有网络接口
- `127.0.0.1`: 仅本地访问
- `::`: IPv6 所有接口

#### **示例**
```yaml
network:
  listen_addr: "0.0.0.0"
  buffer_size: 16384      # 16KB 缓冲区
  timeout: 60             # 60秒超时
```

### **3. 转发规则 (rules)**

每个规则包含以下参数：

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | String | ✅ | 规则名称 |
| `listen_port` | Integer | ✅ | 监听端口 |
| `protocol` | String | ✅ | 协议类型 |
| `targets` | Array | ✅ | 目标服务器列表 |
| `health_check` | Object | ❌ | 健康检查配置 |
| `load_balance` | String | ❌ | 负载均衡策略 |

#### **协议类型**
- `tcp`: TCP 协议转发
- `udp`: UDP 协议转发
- `http`: HTTP 协议转发

#### **负载均衡策略**
- `round_robin`: 轮询 (默认)
- `random`: 随机选择
- `least_connections`: 最少连接

#### **健康检查配置**
```yaml
health_check:
  enabled: true
  interval: 30          # 检查间隔 (秒)
  timeout: 5            # 检查超时 (秒)
  retries: 3            # 重试次数
  path: "/health"       # HTTP 健康检查路径
```

---

## 📝 **完整配置示例**

### **基础配置**
```yaml
logging:
  level: "info"
  format: "text"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 8192
  timeout: 30

rules:
  - name: "HTTPS转发"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "web1.example.com:443"
      - "web2.example.com:443"
```

### **高级配置**
```yaml
logging:
  level: "info"
  format: "json"
  file: "/var/log/smart-forward.log"

network:
  listen_addr: "0.0.0.0"
  buffer_size: 16384
  timeout: 60

rules:
  # HTTPS 转发 (带健康检查)
  - name: "HTTPS负载均衡"
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

  # HTTP API 转发
  - name: "API转发"
    listen_port: 8080
    protocol: "http"
    load_balance: "least_connections"
    targets:
      - "api1.example.com:8080"
      - "api2.example.com:8080"
    health_check:
      enabled: true
      interval: 15
      timeout: 3
      retries: 2
      path: "/health"

  # UDP DNS 转发
  - name: "DNS转发"
    listen_port: 53
    protocol: "udp"
    targets:
      - "8.8.8.8:53"
      - "1.1.1.1:53"

  # 游戏服务器转发
  - name: "游戏服务器"
    listen_port: 25565
    protocol: "tcp"
    targets:
      - "game1.example.com:25565"
      - "game2.example.com:25565"
```

---

## 🔍 **配置验证**

### **验证配置文件**
```bash
# 验证配置语法
smart-forward -c config.yaml --validate

# 测试配置并显示详细信息
smart-forward -c config.yaml --test
```

### **常见配置错误**

#### **1. 端口冲突**
```yaml
# ❌ 错误: 多个规则使用相同端口
rules:
  - name: "规则1"
    listen_port: 80
  - name: "规则2"
    listen_port: 80    # 冲突!
```

#### **2. 无效的协议**
```yaml
# ❌ 错误: 不支持的协议
rules:
  - name: "错误规则"
    protocol: "ftp"    # 不支持!
```

#### **3. 缺少必需字段**
```yaml
# ❌ 错误: 缺少 targets
rules:
  - name: "不完整规则"
    listen_port: 80
    protocol: "tcp"
    # targets: []      # 必需!
```

---

## 🎯 **最佳实践**

### **1. 性能优化**
```yaml
network:
  buffer_size: 65536    # 大缓冲区提高吞吐量
  timeout: 120          # 长连接场景增加超时
```

### **2. 安全配置**
```yaml
network:
  listen_addr: "127.0.0.1"  # 仅本地访问
  timeout: 10               # 短超时防止资源耗尽
```

### **3. 高可用配置**
```yaml
rules:
  - name: "高可用服务"
    targets:
      - "primary.example.com:443"
      - "backup1.example.com:443"
      - "backup2.example.com:443"
    health_check:
      enabled: true
      interval: 10      # 快速检测故障
      retries: 1        # 快速故障转移
```

### **4. 监控配置**
```yaml
logging:
  level: "info"
  format: "json"        # 便于日志分析
  file: "/var/log/smart-forward.log"
```

---

## 📊 **环境变量支持**

可以使用环境变量覆盖配置：

| 环境变量 | 配置项 | 示例 |
|----------|--------|------|
| `SF_LOG_LEVEL` | `logging.level` | `export SF_LOG_LEVEL=debug` |
| `SF_LISTEN_ADDR` | `network.listen_addr` | `export SF_LISTEN_ADDR=127.0.0.1` |
| `SF_BUFFER_SIZE` | `network.buffer_size` | `export SF_BUFFER_SIZE=16384` |

### **Docker 环境变量**
```bash
docker run -d \
  -e SF_LOG_LEVEL=debug \
  -e SF_LISTEN_ADDR=0.0.0.0 \
  ghcr.io/cls3389/smart-forward:latest
```
