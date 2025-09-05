# 智能网络转发器 - 完整构建指南

## 📊 项目概览

**模块结构 (极限精简版)**:
```
src/
├── main.rs      # 程序入口
├── config.rs    # 配置管理
├── common.rs    # 核心管理器 (DNS+健康检查)
├── utils.rs     # 工具函数+统计
└── forwarder.rs # 完整转发器实现 (TCP/UDP/HTTP/统一/智能)
```

**特性**: 5个模块，代码精简，逻辑清晰，性能优化

---

## 🖥️ Windows环境编译

### 1. 开发版本编译
```cmd
# 调试版本 (快速编译，用于开发)
cargo build

# 可执行文件位置: target\debug\smart-forward.exe
```

### 2. 生产版本编译
```cmd
# 发布版本 (优化编译，用于生产)
cargo build --release

# 可执行文件位置: target\release\smart-forward.exe
# 文件大小: ~5.3MB
```

### 3. 验证配置
```cmd
# 验证配置文件正确性
.\target\release\smart-forward.exe --validate-config

# 查看帮助信息
.\target\release\smart-forward.exe --help
```

### 4. 运行服务
```cmd
# 前台运行 (查看实时日志)
.\target\release\smart-forward.exe

# 后台运行 (Windows服务模式)
.\target\release\smart-forward.exe --daemon
```

---

## 🐧 Linux环境编译

### 1. 系统要求
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel
```

### 2. 编译步骤
```bash
# 克隆或复制项目到Linux环境
cd /path/to/smart-forward

# 安装Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 编译发布版本
cargo build --release

# 可执行文件: target/release/smart-forward
```

### 3. 运行服务
```bash
# 验证配置
./target/release/smart-forward --validate-config

# 前台运行
./target/release/smart-forward

# 后台运行
nohup ./target/release/smart-forward > logs/smart-forward.log 2>&1 &

# 系统服务安装
sudo cp target/release/smart-forward /usr/local/bin/
sudo systemctl enable smart-forward
sudo systemctl start smart-forward
```

---

## 🐳 Docker环境部署

### 1. WSL2 Ubuntu环境

#### 进入WSL2环境
```cmd
# 从Windows进入WSL2
wsl
```

#### 在WSL2中构建
```bash
# 进入项目目录 (从Windows盘符挂载)
cd /mnt/d/Cursor/rust转发20250905

# 检查Docker环境
docker --version
sudo systemctl start docker

# 设置执行权限
chmod +x build-docker.sh run-docker.sh

# 构建Docker镜像 (自动配置127.0.0.1:7897代理)
./build-docker.sh
```

### 2. Docker运行命令

#### 方式一: 使用脚本
```bash
# 一键运行 (推荐)
./run-docker.sh

# 查看日志
docker logs -f smart-forward-container
```

#### 方式二: 手动命令
```bash
# 运行容器
docker run -d \
  --name smart-forward-container \
  --restart unless-stopped \
  -p 443:443 \
  -p 99:99 \
  -p 6690:6690 \
  -p 999:999 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  smart-forward:latest

# 查看状态
docker ps

# 查看日志
docker logs -f smart-forward-container

# 停止容器
docker stop smart-forward-container
docker rm smart-forward-container
```

### 3. Docker Compose部署

#### 启动服务
```bash
# 启动 (推荐用于生产环境)
docker-compose up -d

# 查看状态
docker-compose ps

# 查看日志
docker-compose logs -f smart-forward

# 重启服务
docker-compose restart

# 停止服务
docker-compose down
```

#### 扩展配置
```yaml
# docker-compose.yml 自定义配置
version: '3.8'
services:
  smart-forward:
    image: smart-forward:latest
    container_name: smart-forward-container
    restart: unless-stopped
    ports:
      - "443:443"   # HTTPS服务
      - "99:99"     # RDP服务 (TCP+UDP)
      - "6690:6690" # 网盘服务
      - "999:999"   # 分离式RDP (TCP+UDP)
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
      - "./logs:/app/logs"
    environment:
      - RUST_LOG=info
      - TZ=Asia/Shanghai
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.5'
```

---

## 🔧 常见问题解决

### Windows环境
```cmd
# 解决编译慢的问题
set RUSTC_WRAPPER=sccache

# 清理缓存重新编译
cargo clean
cargo build --release
```

### Linux环境
```bash
# 解决权限问题
sudo chown -R $USER:$USER ~/.cargo

# 解决SSL问题
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
```

### Docker环境
```bash
# 解决权限问题
sudo usermod -aG docker $USER
# 重新登录WSL

# 解决代理问题
export HTTP_PROXY=http://127.0.0.1:7897
export HTTPS_PROXY=http://127.0.0.1:7897

# 查看构建日志
docker build --no-cache -t smart-forward:latest .
```

---

## 📝 配置文件说明

### 基本配置
```yaml
# config.yaml
network:
  listen_addr: "0.0.0.0"

logging:
  level: "info"
  format: "plain"

rules:
  - name: "HTTPS"
    listen_port: 443
    protocol: "tcp"
    targets:
      - "192.168.5.254:443"      # 主服务器
      - "121.40.167.222:50443"   # 备用服务器
      - "stun-443.4.ipto.top"    # 备用服务器
```

### 健康检查配置
```yaml
global_dynamic_update:
  check_interval: 15      # 15秒检查间隔
  connection_timeout: 300 # 5分钟连接超时
  auto_reconnect: true    # 自动重连
```

---

## 🚀 性能优化建议

### 生产环境
- 使用 `--release` 编译获得最佳性能
- 设置合适的 `buffer_size` (16KB-64KB)
- 启用日志轮转避免日志文件过大
- 使用systemd管理服务生命周期

### Docker环境
- 设置合理的资源限制
- 使用健康检查确保服务可用性
- 挂载日志目录便于调试
- 使用docker-compose统一管理

### 网络优化
- 配置DNS缓存减少解析开销
- 调整健康检查间隔平衡响应性和负载
- 使用多个备用地址提高可用性

---

## 🎉 部署验证

### 验证转发功能
```bash
# 测试HTTPS转发
curl -I https://localhost:443

# 测试RDP端口
telnet localhost 99

# 测试网盘端口
nc -zv localhost 6690
```

### 查看运行状态
```bash
# 查看进程
ps aux | grep smart-forward

# 查看端口监听
netstat -tlnp | grep smart-forward

# 查看日志
tail -f logs/smart-forward.log
```

---

## 📞 技术支持

- **配置问题**: 使用 `--validate-config` 验证配置
- **网络问题**: 检查防火墙和端口开放情况
- **性能问题**: 调整缓冲区大小和检查间隔
- **Docker问题**: 检查端口映射和挂载目录权限
