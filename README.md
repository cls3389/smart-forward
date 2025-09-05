# 智能网络转发器（Smart Forward）

一个专注稳定与高性能的多协议网络转发器，支持 TCP/UDP/HTTP，具备动态地址解析、健康检查与智能故障转移。适合个人/家庭网络服务、RDP/HTTPS/网盘等场景。

## 📚 项目概述

### 设计理念
本项目专为**个人网络服务优化**设计，通过智能地址管理和端口映射，实现以下目标：
- **免端口访问**: 通过域名劫持到本地443端口，实现无端口号访问
- **单端口汇聚**: TCP+UDP同端口监听，支持STUN穿透和RDP等协议
- **动态地址管理**: 后端地址变化时自动更新，保持连接稳定性
- **性能优化**: 绕过系统代理，直接连接，提升网络性能

### 核心应用场景
1. **域名劫持免端口访问**: 将后端动态地址映射到本地443端口，通过域名直接访问
2. **HTTP自动跳转HTTPS**: 自动将HTTP请求重定向到HTTPS，简化连接并提升安全性
3. **STUN穿透同端口连接**: TCP+UDP同端口监听，支持RDP等需要双协议的应用
   - RDP通过STUN地址连接时，由于穿透端口不固定，需要同时支持TCP和UDP
   - 单端口汇聚简化了NAT穿透配置，提升连接成功率
4. **固定端口动态地址**: 本地固定端口连接远程变化的服务地址（如网盘服务）
5. **高性能网络连接**: 绕过系统代理，直接连接，提升网络性能和安全性

## 🚀 核心特性

- **多协议**: TCP / UDP / HTTP（80 自动 301 到 HTTPS）
- **动态地址**: 支持 A/AAAA 与 TXT 记录（`hostname` -> TXT `IP:PORT`）
- **健康检查**: TCP连接检查 + UDP DNS解析检查，自动切换最佳目标
- **会话粘性**: 严格按配置顺序选择，保持连接稳定性
- **UDP 会话映射**: 客户端独立上游 socket，已实现回程与 60 秒闲置清理
- **灵活缓冲**: 全局与规则级 `buffer_size`

## 🏗️ 系统架构设计

### 项目本质
本项目是一个**智能端口映射和地址管理器**，专为个人网络服务优化设计。核心功能是：
- **端口映射**: 将后端动态地址端口映射到本地固定端口（如443）
- **协议汇聚**: TCP+UDP同端口监听，支持STUN穿透和RDP等应用
  - RDP可通过STUN地址进行TCP+UDP连接（STUN穿透时端口不固定）
  - 单端口同时处理TCP和UDP协议，简化防火墙配置
- **动态地址管理**: 后端地址变化时自动更新，保持连接稳定性
- **性能优化**: 绕过系统代理，直接连接，提升网络性能

### 代码架构 (极限精简版)
```
src/
├── main.rs      # 程序入口 (170行)
├── config.rs    # 配置管理 (172行)  
├── common.rs    # 核心管理器 (501行) - DNS解析+健康检查+目标选择
├── utils.rs     # 工具函数 (211行) - 网络工具+统计功能
└── forwarder.rs # 转发器实现 (683行) - TCP/UDP/HTTP/统一/智能转发

总计: 5个模块, ~1600行代码
特点: 模块精简, 逻辑清晰, 性能优化
```

### 整体架构
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   配置管理模块   │    │   转发器模块     │    │   公共管理模块   │
│   (config.rs)   │    │ (forwarder.rs)  │    │ (common.rs)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   YAML 配置     │    │   端口映射      │    │   动态地址      │
│   动态更新      │    │   TCP+UDP汇聚   │    │   健康检查      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 代码结构
```
src/
├── main.rs              # 程序入口，命令行解析，日志初始化
├── config.rs            # 配置管理，YAML解析，配置验证
├── common.rs            # 公共管理器，统计信息，健康检查
├── forwarder/           # 转发器模块
│   ├── mod.rs           # 转发器统一接口和智能转发器
│   ├── tcp.rs           # TCP协议转发器
│   ├── udp.rs           # UDP协议转发器
│   ├── http.rs          # HTTP协议转发器
│   └── unified.rs       # 统一转发器（多协议支持）
├── stats.rs             # 统计信息收集和管理
└── utils.rs             # 工具函数，网络工具，时间工具
```

## 🛠️ 技术选型

### 核心语言与运行时
- **Rust 2024 Edition**: 选择 Rust 作为主要开发语言
  - 零成本抽象，高性能
  - 内存安全，无数据竞争
  - 并发安全，适合网络编程
  - 跨平台编译支持

### 异步运行时
- **Tokio 1.0**: 异步运行时框架
  - `rt-multi-thread`: 多线程运行时
  - `net`: 网络 I/O 支持
  - `time`: 时间相关功能
  - `macros`: 异步宏支持
  - `sync`: 同步原语
  - `signal`: 信号处理
  - `io-util`: I/O 工具函数

### 序列化与配置
- **Serde 1.0**: 序列化/反序列化框架
  - 支持 YAML 配置文件
  - 类型安全的配置管理
- **Serde YAML 0.9**: YAML 格式支持
- **Serde JSON 1.0**: JSON 格式支持（日志输出）

### 错误处理
- **Anyhow 1.0**: 错误处理库
  - 简化的错误传播
  - 类型擦除的错误类型
- **Thiserror 1.0**: 自定义错误类型
  - 编译时错误定义
  - 自动实现标准 trait

### 日志系统
- **Log 0.4**: 日志门面
- **Env_logger 0.10**: 日志实现
  - 支持环境变量配置
  - 灵活的日志格式

### 命令行解析
- **Clap 4.0**: 命令行参数解析
  - 派生宏支持
  - 自动生成帮助信息

### 并发与同步
- **Futures 0.3**: 异步编程基础
- **Async-trait 0.1**: 异步 trait 支持
- **Dashmap 5.0**: 并发 HashMap
  - 高性能并发访问
  - 无锁数据结构

### 时间处理
- **Chrono 0.4**: 时间日期库
  - 时区支持
  - 序列化集成

### 网络与 DNS
- **Trust-dns-resolver 0.23**: DNS 解析器
  - 系统配置支持
  - Tokio 运行时集成
- **Local-ip-address 0.6**: 本地 IP 地址获取

## 🔧 核心技术特性

### 1. 端口映射机制
- **本地端口绑定**: 将后端动态地址映射到本地固定端口（如443）
- **域名劫持**: 通过本地DNS劫持，实现免端口号访问
- **动态更新**: 后端地址变化时自动更新映射关系

### 2. 协议汇聚支持
- **TCP+UDP同端口**: 单个端口同时监听TCP和UDP协议
- **HTTP自动跳转**: 自动将HTTP请求重定向到HTTPS，简化连接
- **STUN穿透**: 支持NAT穿透和STUN协议
- **RDP优化**: 针对RDP等需要双协议的应用进行优化
  - RDP通过STUN地址连接时，穿透端口不固定，需要动态处理TCP+UDP
  - 自动识别协议类型，智能路由到对应的处理逻辑
  - 支持动态端口映射，适应STUN穿透的端口变化

### 3. 性能优化设计
- **绕过系统代理**: 直接连接，避免代理层性能损耗
- **虚拟组网**: 本地网络栈优化，提升连接性能
- **动态地址缓存**: 减少DNS查询，提升响应速度

## 🎯 核心设计模式

### 1. 策略模式
- 不同协议使用不同的转发策略
- 通过 trait 实现统一的转发接口
- 支持运行时策略切换

### 2. 工厂模式
- 根据配置自动创建对应的转发器
- 支持多协议同时转发
- 动态加载和卸载转发器

### 3. 观察者模式
- 健康检查结果通知
- 统计信息更新通知
- 配置变更通知

### 4. 状态模式
- 连接状态管理
- 故障转移状态机
- 会话生命周期管理

## ⚡ 性能优化设计

### 1. 异步 I/O
- 全异步网络 I/O 操作
- 非阻塞 I/O 模型
- 高并发连接处理

### 2. 内存管理
- 零拷贝数据传输
- 智能缓冲区管理
- 内存池优化

### 3. 并发控制
- 无锁数据结构
- 读写锁分离
- 原子操作优化

### 4. 端口映射优化
- 动态地址解析和缓存
- 本地端口到远程地址的智能映射
- 协议汇聚和STUN穿透支持

## 🔒 安全设计

### 1. 输入验证
- 配置参数验证
- 网络数据验证
- 协议合规性检查

### 2. 错误处理
- 优雅的错误恢复
- 详细的错误日志
- 安全的错误传播

### 3. 资源管理
- 自动资源清理
- 内存泄漏防护
- 连接超时管理

## 📊 核心算法设计

### 1. 故障转移算法

#### 会话粘性策略
```rust
// 配置顺序优先
for target in targets {
    if target.healthy {
        return target;
    }
}

// 保持当前连接
if current_target.is_healthy() {
    return current_target;
}

// 故障转移逻辑：优先选择健康的目标
if let Some(healthy_target) = healthy_targets.first() {
    return healthy_target.resolved;
}
// 如果没有健康目标，选择失败次数最少的目标
return fallback_target.resolved;
```

#### 故障转移算法
- **健康检查**: TCP连接测试，快速检测服务可用性
- **失败阈值**: 连续失败2次标记为不健康
- **恢复机制**: 成功响应后立即恢复健康状态
- **优先级策略**: 按配置顺序选择，保持连接稳定性

### 2. 缓冲区管理

#### 智能缓冲区分配
```rust
pub fn get_effective_buffer_size(&self, default_size: usize) -> usize {
    self.buffer_size
        .or(self.global_buffer_size)
        .unwrap_or(default_size)
}
```

#### 缓冲区优化策略
- 全局默认缓冲区大小
- 规则级缓冲区覆盖
- 协议特定缓冲区优化
- 动态缓冲区调整

### 3. 连接管理

#### TCP 连接处理
```rust
// 每次连接都创建新的TCP连接
let mut target_stream = connect_with_timeout_and_retry(
    target, 
    1,  // max_retries
    8,  // timeout_secs
    2,  // retry_delay_secs
    &format!("规则 {}", rule_name)
).await?;

// 并发双向数据转发
let (client_to_target, target_to_client) = tokio::join!(
    Self::forward_data(&mut client_read, &mut target_write, &mut client_buffer, &stats, true),
    Self::forward_data(&mut target_read, &mut client_write, &mut target_buffer, &stats, false),
);
```

#### UDP 会话映射
```rust
struct UDPSession {
    client_addr: SocketAddr,
    upstream_socket: UdpSocket,
    last_activity: Instant,
    data: Vec<u8>,
}
```

#### 会话清理机制
- TCP: 连接断开时自动清理
- UDP: 60秒闲置超时
- 定期清理任务
- 内存使用监控
- 连接数限制

## 🚀 快速开始

### 环境要求
- Rust 1.70+ 
- Cargo
- Windows/Linux/macOS

### 快速编译运行
```bash
# 编译生产版本
cargo build --release

# Windows运行
.\target\release\smart-forward.exe --config config.yaml

# Linux运行  
./target/release/smart-forward --config config.yaml

# 验证配置
./target/release/smart-forward --validate-config
```

Windows 批处理脚本也可用：`run.bat` / `run-daemon.bat`。

## 📈 性能指标

### 设计目标
- **高并发**: 支持数千并发连接
- **低延迟**: 毫秒级响应时间
- **高吞吐**: 支持 Gbps 级数据传输
- **稳定性**: 7x24 小时稳定运行

### 优化策略
- 异步 I/O 模型
- 端口映射和协议汇聚
- HTTP自动跳转HTTPS
- 动态地址管理和缓存
- 绕过系统代理直接连接
- 无锁数据结构

## 🔧 运行与日志
- 若未设置 `RUST_LOG`，读取 `config.yaml` 的 `logging.level`；格式支持 `json` / `text`
- 程序自动设置时区 `Asia/Shanghai`

## ⚙️ 配置示例（config.yaml）
```yaml
logging:
  level: "info"   # debug/info/warn/error
  format: "json"  # json/text

network:
  listen_addr: "0.0.0.0"

buffer_size: 8192  # 全局默认缓冲区

rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    buffer_size: 4096
    targets:
      - "192.168.5.254:443"
      - "121.40.167.222:50443"
      - "stun-443.4.ipto.top"   # 纯域名，TXT 记录解析

  - name: "RDP"
    listen_port: 99
    # 未显式指定时，默认同时支持 tcp+udp
    buffer_size: 16384  # 建议 16K~32K；一般约30Mbps 足够
    targets:
      - "192.168.5.12:3389"
      - "121.40.167.222:57111"
      - "ewin10.4.ipto.top"
      - "stun-rdp.example.com"  # STUN地址，穿透端口不固定，自动处理TCP+UDP

# 提示：UDP 不进行健康检查，仅进行 UDP 转发；健康检查只对 TCP 生效。

  - name: "Drive"
    listen_port: 6690
    protocol: "tcp"
    buffer_size: 32768
    targets:
      - "192.168.5.3:6690"
      - "121.40.167.222:6690"
      - "drive.4.ipto.top"
```

### 协议字段
- `protocol`: 单协议（`tcp` | `udp` | `http`）
- `protocols`: 多协议列表（如 `["tcp","udp"]`）；若都未设置，默认启用 `tcp+udp`
- **STUN地址支持**: 当目标为STUN地址时，自动启用TCP+UDP双协议支持，适应穿透端口不固定的特性

### 动态更新（与实现一致）
- `check_interval`（默认 15s）
- `connection_timeout`（默认 300s）
- `auto_reconnect`（默认 true）

## 🔮 未来规划

### 1. 功能扩展
- WebSocket 协议支持
- 加密传输支持
- 压缩传输支持
- 缓存机制

### 2. 性能优化
- 零拷贝优化
- 内存池优化
- 网络栈优化
- 并发模型优化

### 3. 运维增强
- 监控指标完善
- 告警机制
- 自动化部署
- 故障自愈

### 4. 生态集成
- Prometheus 指标导出
- Grafana 仪表板
- Kubernetes 部署支持
- 云原生集成

## 📦 发布部署

### Windows版本
```bash
cargo build --release
# 产物：target/release/smart-forward.exe (~5.3MB)
```

### Docker版本
```bash
# 在WSL2 Ubuntu中构建
./build-docker.sh
./run-docker.sh
```

### 完整部署指南
详细的编译、部署和运行指南：
- **[FINAL-GUIDE.md](FINAL-GUIDE.md)** - 完整使用指南（编译、部署、运行）
- **[TECHNICAL-DOCS.md](TECHNICAL-DOCS.md)** - 技术文档（架构设计、实现细节）

可选择将 `smart-forward.exe` 与 `config.yaml` 放入同目录直接运行，或按需封装为服务/打包分发。

## 📞 支持与反馈

如果您在使用过程中遇到问题或有改进建议，欢迎：
- 查看现有文档
- 提交 Issue
- 参与讨论
- 贡献代码

---

*最后更新时间: 2024年*
