# GitHub 仓库设置指南

## 🚀 快速设置

### 1. 创建 GitHub 仓库

1. 访问 [GitHub](https://github.com) 并登录
2. 点击 "New repository" 创建新仓库
3. 仓库名称: `smart-forward`
4. 描述: `智能网络转发器 - 支持TCP、UDP、HTTP协议的高性能转发工具`
5. 选择 Public 或 Private
6. **不要**初始化 README、.gitignore 或 license（我们已经有了）
7. 点击 "Create repository"

### 2. 添加远程仓库

```bash
# 添加远程仓库（替换 your-username）
git remote add origin https://github.com/your-username/smart-forward.git

# 推送到 GitHub
git push -u origin optimize/fault-tolerance-and-config-simplification

# 创建并切换到 main 分支
git checkout -b main
git push -u origin main

# 删除旧分支（可选）
git branch -d optimize/fault-tolerance-and-config-simplification
```

### 3. 配置 GitHub Actions 密钥

在 GitHub 仓库设置中添加以下 Secrets：

#### 必需密钥
- `DOCKER_USERNAME`: Docker Hub 用户名
- `DOCKER_PASSWORD`: Docker Hub 密码或访问令牌

#### 可选密钥
- `CARGO_REGISTRY_TOKEN`: Cargo 发布令牌（如果要发布到 crates.io）

### 4. 启用 GitHub Actions

1. 进入仓库的 "Actions" 标签页
2. 点击 "I understand my workflows, go ahead and enable them"
3. 工作流将自动开始运行

## 📋 工作流说明

### 构建工作流 (`.github/workflows/build.yml`)

**触发条件:**
- Push 到 main/master/develop 分支
- 创建 Pull Request
- 手动触发

**功能:**
- ✅ 代码质量检查 (rustfmt, clippy)
- ✅ 运行测试
- ✅ 多平台构建 (Windows, macOS, Linux x86_64/ARM64)
- ✅ 安全扫描 (cargo audit, cargo deny)
- ✅ 构建产物上传

### Docker 工作流 (`.github/workflows/docker.yml`)

**触发条件:**
- Push 到 main/master 分支
- 创建标签
- 手动触发

**功能:**
- ✅ 多架构 Docker 镜像构建 (linux/amd64, linux/arm64)
- ✅ 推送到 GitHub Container Registry
- ✅ 安全扫描 (Trivy)
- ✅ 镜像测试

### 发布工作流 (`.github/workflows/release.yml`)

**触发条件:**
- 创建版本标签 (v*)
- 手动触发

**功能:**
- ✅ 构建所有平台发布版本
- ✅ 创建 GitHub Release
- ✅ 自动生成发布说明
- ✅ 上传构建产物
- ✅ 发布到 Cargo (可选)

## 🏷️ 版本发布流程

### 1. 创建版本标签

```bash
# 更新版本号
# 编辑 Cargo.toml 中的 version 字段

# 创建标签
git tag -a v0.1.0 -m "Release version 0.1.0"

# 推送标签
git push origin v0.1.0
```

### 2. 自动发布

发布工作流将自动：
- 构建所有平台的二进制文件
- 创建 GitHub Release
- 上传构建产物
- 生成发布说明

### 3. 手动发布

1. 进入 GitHub 仓库的 "Actions" 标签页
2. 选择 "自动发布" 工作流
3. 点击 "Run workflow"
4. 输入版本标签
5. 点击 "Run workflow"

## 📦 构建产物

### 二进制文件

每次构建都会生成以下文件：

- `smart-forward-windows-x86_64.zip` - Windows x86_64
- `smart-forward-macos-x86_64.tar.gz` - macOS Intel
- `smart-forward-macos-aarch64.tar.gz` - macOS Apple Silicon
- `smart-forward-linux-x86_64.tar.gz` - Linux x86_64
- `smart-forward-linux-aarch64.tar.gz` - Linux ARM64

### Docker 镜像

- `ghcr.io/your-username/smart-forward:latest`
- `ghcr.io/your-username/smart-forward:v0.1.0`
- `ghcr.io/your-username/smart-forward:main`

## 🔧 本地构建

### Windows PowerShell

```powershell
# 构建所有平台
.\build-cross-platform.ps1 -Platform all -Release

# 构建特定平台
.\build-cross-platform.ps1 -Platform windows -Release

# 构建 Docker 镜像
.\build-cross-platform.ps1 -Docker

# 清理构建产物
.\build-cross-platform.ps1 -Clean
```

### Linux/macOS

```bash
# 构建所有平台
./build-cross-platform.sh -p all -r

# 构建特定平台
./build-cross-platform.sh -p linux -r

# 构建 Docker 镜像
./build-cross-platform.sh -d

# 清理构建产物
./build-cross-platform.sh -c
```

## 🐳 Docker 部署

### 使用 GitHub Container Registry

```bash
# 拉取镜像
docker pull ghcr.io/your-username/smart-forward:latest

# 运行容器
docker run -d \
  --name smart-forward \
  -p 443:443 \
  -p 99:99 \
  -p 6690:6690 \
  -p 999:999 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/your-username/smart-forward:latest
```

### 使用 Docker Compose

```yaml
version: '3.8'

services:
  smart-forward:
    image: ghcr.io/your-username/smart-forward:latest
    container_name: smart-forward
    restart: unless-stopped
    ports:
      - "443:443"
      - "99:99"
      - "6690:6690"
      - "999:999"
    volumes:
      - "./config.yaml:/app/config.yaml:ro"
    environment:
      - RUST_LOG=info
```

## 📊 监控和状态

### 构建状态

- 查看 [Actions](https://github.com/your-username/smart-forward/actions) 页面
- 绿色 ✅ 表示构建成功
- 红色 ❌ 表示构建失败

### 发布状态

- 查看 [Releases](https://github.com/your-username/smart-forward/releases) 页面
- 查看 [Packages](https://github.com/your-username/smart-forward/pkgs/container/smart-forward) 页面

## 🛠️ 故障排除

### 常见问题

1. **构建失败**
   - 检查 Rust 版本是否兼容
   - 查看构建日志中的错误信息
   - 确保所有依赖都已正确安装

2. **Docker 构建失败**
   - 检查 Docker 是否正在运行
   - 确保有足够的磁盘空间
   - 检查网络连接

3. **发布失败**
   - 检查 GitHub 密钥是否正确设置
   - 确保有发布权限
   - 检查版本标签格式是否正确

### 获取帮助

- 查看 [GitHub Issues](https://github.com/your-username/smart-forward/issues)
- 查看 [GitHub Discussions](https://github.com/your-username/smart-forward/discussions)
- 查看构建日志获取详细错误信息

## 🎉 完成！

现在您的项目已经配置了完整的 CI/CD 流程：

- ✅ 自动构建多平台二进制文件
- ✅ 自动构建 Docker 镜像
- ✅ 自动发布和版本管理
- ✅ 代码质量检查和安全扫描
- ✅ 完整的文档和示例

开始使用吧！🚀
