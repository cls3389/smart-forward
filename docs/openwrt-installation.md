# OpenWrt 安装指南

## 📋 目录
1. [系统要求](#系统要求)
2. [安装方式](#安装方式)
3. [配置说明](#配置说明)
4. [管理命令](#管理命令)
5. [故障排除](#故障排除)

---

## 🔧 系统要求

### 硬件要求
- **CPU**: ARM64 (如 MT7981) 或 x86_64
- **内存**: 至少 128MB RAM
- **存储**: 至少 32MB 可用空间
- **网络**: 支持端口转发

### 软件要求
- **OpenWrt**: 21.02 或更高版本
- **架构**: aarch64, x86_64, armv7
- **依赖**: wget/curl, tar

---

## 🚀 安装方式

### 直接安装（推荐）

**适用于**：所有 OpenWrt 设备

```bash
# 1. 下载安装脚本
wget -O /tmp/install.sh https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/openwrt-install.sh

# 2. 运行安装脚本
chmod +x /tmp/install.sh
/tmp/install.sh

# 3. 编辑配置
smart-forward-ctl config

# 4. 重启服务
smart-forward-ctl restart
```

### 架构支持

| 架构 | 位数 | 性能 | 推荐度 | 设备示例 |
|------|------|------|--------|----------|
| **aarch64** | 64位 | 高 | ⭐⭐⭐⭐⭐ | MT7981, 树莓派 4B |
| **x86_64** | 64位 | 高 | ⭐⭐⭐⭐⭐ | x86 路由器 |
| **armv7** | 32位 | 中 | ⭐⭐⭐ | 树莓派 3B |
| **mips** | 32位 | 低 | ⭐⭐ | 老路由器 |

---

## ⚙️ 配置说明

### 配置文件位置
- **直接安装**: `/etc/smart-forward/config.yaml`
- **Docker 安装**: `/etc/smart-forward/config.yaml`

### 基本配置示例

```yaml
# 全局配置
global:
  log_level: "info"
  log_file: "/var/log/smart-forward/smart-forward.log"
  health_check_interval: 30
  dns_cache_ttl: 300

# 转发规则
rules:
  - name: "HTTPS转发"
    listen_port: 443
    protocol: "tcp"
    targets:
      - host: "your-server.com"
        port: 443
        priority: 1
        health_check: true
  
  - name: "HTTP转发"
    listen_port: 80
    protocol: "tcp"
    targets:
      - host: "your-server.com"
        port: 80
        priority: 1
        health_check: true
```

### 端口配置

| 端口 | 协议 | 用途 | 说明 |
|------|------|------|------|
| 443 | TCP | HTTPS | 加密流量转发 |
| 80 | TCP | HTTP | 明文流量转发 |
| 99 | TCP | 自定义 | 可配置端口 |
| 6690 | TCP | 自定义 | 可配置端口 |
| 999 | TCP | 自定义 | 可配置端口 |

---

## 🎛️ 管理命令

```bash
# 启动服务
smart-forward-ctl start

# 停止服务
smart-forward-ctl stop

# 重启服务
smart-forward-ctl restart

# 查看状态
smart-forward-ctl status

# 查看日志
smart-forward-ctl logs

# 编辑配置
smart-forward-ctl config
```

---

## 🔍 故障排除

### 常见问题

#### 1. 服务启动失败

**症状**: 服务无法启动

**解决方案**:
```bash
# 检查日志
smart-forward-ctl logs

# 检查配置文件
cat /etc/smart-forward/config.yaml

# 检查二进制文件
ls -la /usr/local/bin/smart-forward
```

#### 2. 端口冲突

**症状**: 端口被占用

**解决方案**:
```bash
# 检查端口占用
netstat -tlnp | grep :443

# 修改配置文件中的端口
smart-forward-ctl config
```

#### 3. 网络连接问题

**症状**: 无法连接目标服务器

**解决方案**:
```bash
# 检查网络连通性
ping your-server.com

# 检查 DNS 解析
nslookup your-server.com

# 检查防火墙规则
iptables -L
```

#### 4. 权限问题

**症状**: 权限不足

**解决方案**:
```bash
# 检查文件权限
ls -la /usr/local/bin/smart-forward
ls -la /etc/smart-forward/

# 修复权限
chmod +x /usr/local/bin/smart-forward
chmod 644 /etc/smart-forward/config.yaml
```

### 日志分析

#### 查看实时日志
```bash
# 直接安装
tail -f /var/log/smart-forward/smart-forward.log

# Docker 安装
docker logs -f smart-forward
```

#### 日志级别
- **debug**: 详细调试信息
- **info**: 一般信息（推荐）
- **warn**: 警告信息
- **error**: 错误信息

---

## 📊 性能优化

### 内存优化
```yaml
# 在配置文件中添加
global:
  max_connections: 1000
  buffer_size: 8192
```

### 网络优化
```yaml
# 调整超时设置
global:
  connect_timeout: 5
  read_timeout: 30
  write_timeout: 30
```

### 日志优化
```yaml
# 减少日志输出
global:
  log_level: "warn"
  log_rotation: true
  max_log_size: "10MB"
```

---

## 🔄 更新升级

### 直接安装更新
```bash
# 重新运行安装脚本
wget -O /tmp/install.sh https://raw.githubusercontent.com/cls3389/smart-forward/main/scripts/openwrt-install.sh
chmod +x /tmp/install.sh
/tmp/install.sh
```

### Docker 安装更新
```bash
# 更新镜像
smart-forward-docker-ctl update
```

---

## 📞 技术支持

### 相关链接
- [GitHub 仓库](https://github.com/cls3389/smart-forward)
- [问题反馈](https://github.com/cls3389/smart-forward/issues)
- [OpenWrt 文档](https://openwrt.org/docs)

### 获取帮助
1. 查看日志文件
2. 检查配置文件
3. 在 GitHub 提交 Issue
4. 提供设备信息和错误日志

---

## 🎯 总结

OpenWrt 安装提供了两种方式：

1. **直接安装** - 适合大多数设备，资源占用少
2. **Docker 安装** - 适合支持 Docker 的设备，管理方便

选择适合您设备的方式，按照指南安装即可！
