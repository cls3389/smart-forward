# 智能网络转发器（Smart Forward）

一个专注稳定与高性能的多协议网络转发器，支持 TCP/UDP/HTTP，具备动态地址解析、健康检查与智能故障转移。适合个人/家庭内网穿透、RDP/HTTPS/网盘等场景。

## 特性
- 多协议：TCP / UDP / HTTP（80 自动 301 到 HTTPS）
- 动态地址：支持 A/AAAA 与 TXT 记录（`hostname` -> TXT `IP:PORT`）
- 健康检查：快速检查 + 定期检查，自动切换最佳目标
- 会话粘性：同网段优先、本地优先、失败次数与延迟综合权衡
- UDP 会话映射：客户端独立上游 socket，已实现回程与 60 秒闲置清理
- 灵活缓冲：全局与规则级 `buffer_size`

## 快速开始
```bash
# 编译
cargo build --release

# 运行（前台）
./target/release/smart-forward --config config.yaml
```
Windows 批处理脚本也可用：`run.bat` / `run-daemon.bat`。

> 说明：仓库已精简，仅保留本 README 与必要源码与配置；临时测试脚本与运行日志均已移除，并加入 `.gitignore` 忽略。

## 配置示例（config.yaml）
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
    buffer_size: 32768  # 建议 16K~32K；外网约30Mbps 足够
    targets:
      - "192.168.5.12:3389"
      - "121.40.167.222:57111"
      - "ewin10.4.ipto.top"

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

### 动态更新（与实现一致）
- `check_interval`（默认 15s）
- `connection_timeout`（默认 300s）
- `auto_reconnect`（默认 true）

## 运行与日志
- 若未设置 `RUST_LOG`，读取 `logging.level`；格式支持 `json` / `text`
- 程序自动设置时区 `Asia/Shanghai`

## 本次修复与改进（要点）
- HTTP 301：精准 `Content-Length`（避免不一致导致的客户端异常）
- TCP：移除常规路径 `flush()`，提升吞吐
- 日志：读取 `config.yaml` 的 `level/format`，修正不必要的 `unsafe` 使用
- UDP：实现会话映射 + 独立异步回程（移除每包等待），提升吞吐；60 秒闲置清理
- 文档合并：仅保留本 `README.md`，并加入 `.gitignore` 忽略运行日志

## 发布（Windows）
```bash
cargo build --release
# 产物：target/release/smart-forward.exe
```

可选择将 `smart-forward.exe` 与 `config.yaml` 放入同目录直接运行，或按需封装为服务/打包分发。
