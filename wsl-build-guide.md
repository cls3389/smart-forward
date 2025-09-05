# WSL2 Ubuntu Docker构建指南

## 1. 进入WSL2并切换到项目目录
```bash
# 在Windows中打开WSL2
wsl

# 进入项目目录（假设在D盘）
cd /mnt/d/Cursor/rust转发20250905
```

## 2. 检查和设置权限
```bash
# 检查文件是否存在
ls -la *.sh Dockerfile docker-compose.yml

# 设置执行权限
chmod +x build-docker.sh run-docker.sh
```

## 3. 检查Docker是否运行
```bash
# 检查Docker状态
docker --version
sudo systemctl status docker

# 如果Docker未启动
sudo systemctl start docker
```

## 4. 构建Docker镜像
```bash
# 执行构建脚本（自动配置127.0.0.1:7897代理）
./build-docker.sh
```

## 5. 运行容器
```bash
# 方式1：使用运行脚本
./run-docker.sh

# 方式2：使用docker-compose
docker-compose up -d

# 查看日志
docker logs -f smart-forward-container

# 停止容器
docker-compose down
```

## 6. 验证运行
```bash
# 检查容器状态
docker ps

# 测试端口
curl -I http://localhost:443
```

## 常见问题

### 权限问题
```bash
# 如果权限不足
sudo usermod -aG docker $USER
# 然后重新登录WSL
```

### 代理问题
```bash
# 手动设置代理
export HTTP_PROXY=http://127.0.0.1:7897
export HTTPS_PROXY=http://127.0.0.1:7897
```
