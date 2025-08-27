# Smart Forward - 智能网络转发器

一个高性能的Rust网络转发器，支持TCP转发、HTTP重定向和智能故障转移。

## 🚀 特性

- **多协议支持**: TCP转发、HTTP到HTTPS重定向
- **智能故障转移**: 自动检测目标健康状态，支持多目标故障转移
- **DNS解析**: 支持IP:PORT、域名:PORT、纯域名TXT记录解析
- **阿里云DNS优先**: 国内网络优化，优先使用阿里云DNS服务器
- **断网检测**: 智能检测网络状态，断网时暂停健康检查
- **高性能**: 基于Tokio异步运行时，支持高并发连接
- **配置灵活**: YAML配置文件，支持动态规则配置

## 🏗️ 架构设计

```
SmartForwarder
├── CommonManager (公共管理器)
│   ├── DNS解析器
│   ├── 健康检查器
│   ├── 目标选择器
│   └── 故障转移逻辑
├── Forwarder (转发器)
│   ├── TCPForwarder (TCP转发)
│   ├── HTTPForwarder (HTTP重定向)
│   └── UnifiedForwarder (统一转发器)
└── Config (配置管理)
```

### 核心组件

1. **CommonManager**: 负责目标管理、健康检查、故障转移
2. **Forwarder**: 处理具体的网络转发逻辑
3. **DNS Resolver**: 智能DNS解析，支持多种格式
4. **Health Checker**: 定期健康检查，支持断网检测

## 📦 安装使用

### 开发环境

```bash
# 编译
cargo build --release

# 运行
.\run.bat
```

### 部署使用

1. 复制 `deploy/` 目录中的所有文件到目标目录
2. 双击 `run.bat` 启动程序

### 文件说明

- `smart-forward.exe` - 主程序 (4.8MB)
- `config.yaml` - 配置文件
- `run.bat` - 启动脚本
- `run.ps1` - PowerShell启动脚本

## ⚙️ 配置

### 配置文件 (config.yaml)

```yaml
logging:
  level: "info"
  format: "json"

network:
  listen_addr: "0.0.0.0"

buffer_size: 16384

rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "hz.ipto.top:50443"
      - "192.168.5.254:443"

  - name: "RDP"
    listen_port: 99
    protocol: "tcp"
    targets:
      - "hz.ipto.top:57111"
      - "192.168.5.12:3389"

  - name: "Drive"
    listen_port: 6690
    protocol: "tcp"
    targets:
      - "hz.ipto.top:6690"
      - "192.168.5.3:6690"
      - "drive.4.ipto.top"
```

### 配置说明

- **name**: 规则名称，用于标识
- **listen_port**: 监听端口
- **protocol**: 协议类型 (tcp/http)
- **targets**: 目标地址列表，支持多种格式

## 🔧 使用方法

### 基本使用

```bash
# 开发环境
cargo build --release
.\run.bat

# 部署环境
双击 run.bat
```

### 停止程序

按 `Ctrl+C` 停止程序

### 环境变量

- `RUST_LOG`: 日志级别 (debug, info, warn, error)
- `TZ`: 时区设置 (默认: Asia/Shanghai)

## 🌐 DNS解析支持

### 支持的格式

1. **IP:PORT**: 直接使用
   ```
   192.168.1.1:8080
   ```

2. **域名:PORT**: 解析A/AAAA记录
   ```
   example.com:8080
   ```

3. **纯域名**: 解析TXT记录获取IP:PORT
   ```
   example.com
   ```

### DNS服务器

- 优先使用阿里云DNS: 223.5.5.5, 223.6.6.6
- 支持系统DNS作为备用

## 🔄 程序运行逻辑

### 1. 启动阶段
```
程序启动 → 设置时区和日志 → 加载配置文件 → 创建公共管理器 → 初始化转发规则
```

### 2. 初始化阶段
```
DNS解析目标地址 → 批量健康检查 → 选择最佳目标 → 创建转发器实例
```

### 3. 运行阶段
```
启动所有转发器 → 启动健康检查任务 → 等待客户端连接
```

### 4. 连接处理流程
```
客户端连接 → 获取最佳目标 → 连接目标服务器 → 双向数据转发 → 连接关闭清理
```

### 5. 健康检查流程
```
每30秒并发检查所有目标 → 更新健康状态 → 重新选择最佳目标 → 记录状态变化
```

### 6. 故障转移逻辑
```
目标状态变化 → 按顺序选择第一个健康目标 → 新连接使用新目标 → 现有连接保持不变
```

### 7. 断网检测逻辑
```
所有目标异常 → 暂停健康检查 → 等待网络恢复 → 立即恢复检查
```

## 🔍 健康检查

- **频率**: 30秒间隔
- **策略**: 严格故障转移，选择第一个健康目标
- **断网检测**: 所有目标异常时暂停检查
- **现有连接保护**: 不主动断开当前连接

## 📊 性能特性

- **程序体积**: 4.8MB (优化后)
- **异步I/O**: 基于Tokio运行时
- **高并发**: 支持数千并发连接
- **内存优化**: 零拷贝数据转发

## 🐛 故障排除

### 常见问题

1. **双击程序闪退**
   - 使用 `run.bat` 启动脚本
   - 确保 `config.yaml` 与程序在同一目录

2. **端口被占用**
   - 修改 `config.yaml` 中的 `listen_port`
   - 或停止占用端口的其他程序

3. **权限不足**
   - 以管理员身份运行程序
   - 或使用1024以上的端口

## 📈 项目亮点

- **性能提升**: Rust版本性能优于Go版本
- **体积优化**: 程序体积减少55%
- **错误处理**: 完善的错误处理和日志记录
- **部署简化**: 提供完整的部署包和启动脚本

## 📄 许可证

MIT License
