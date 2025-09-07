# GitHub 权限配置完整指南

## 📋 目录
1. [从零开始完整流程](#从零开始完整流程)
2. [仓库权限配置](#仓库权限配置)
3. [Secrets 管理](#secrets-管理)
4. [Workflow 权限设置](#workflow-权限设置)
5. [包管理权限](#包管理权限)
6. [Docker 镜像构建权限](#docker-镜像构建权限)
7. [组织权限配置](#组织权限配置)
8. [常见问题解决](#常见问题解决)
9. [最佳实践](#最佳实践)

---

## 🚀 从零开始完整流程

### 场景：新建仓库并配置 Docker 镜像构建

基于我们刚才的操作，以下是完整的权限配置流程：

#### 第一步：创建仓库
```bash
# 1. 在 GitHub 上创建新仓库
# 访问：https://github.com/new
# 填写仓库名称：smart-forward
# 选择：Public/Private
# 初始化：README, .gitignore, license
```

#### 第二步：配置仓库基本权限
```bash
# 2. 进入仓库设置
# 访问：https://github.com/用户名/smart-forward/settings

# 3. 配置 Actions 权限
# Settings → Actions → General
# 选择："Read and write permissions"
# 勾选："Allow GitHub Actions to create and approve pull requests"
```

#### 第三步：创建 Personal Access Token
```bash
# 4. 创建 PAT
# 访问：https://github.com/settings/tokens
# 点击："Generate new token (classic)"
# 权限选择：
#   ✅ write:packages
#   ✅ read:packages  
#   ✅ delete:packages
#   ✅ repo
# 复制生成的 token（只显示一次）
```

#### 第四步：配置仓库 Secrets
```bash
# 5. 添加 Secrets
# 访问：https://github.com/用户名/smart-forward/settings/secrets/actions
# 点击："New repository secret"
# Name: GHCR_TOKEN
# Value: 粘贴刚才复制的 token
# 点击："Add secret"
```

#### 第五步：配置包权限
```bash
# 6. 配置包权限
# 访问：https://github.com/用户名/smart-forward/packages
# 如果包不存在，会在首次推送时自动创建
# 包权限会自动继承仓库权限
```

#### 第六步：配置 Workflow 文件
```yaml
# 7. 创建 .github/workflows/build.yml
name: Build and Push Docker Image

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
      id-token: write
      actions: read
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GHCR_TOKEN }}
      
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: Dockerfile
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            ghcr.io/${{ github.repository }}:${{ github.ref_name }}
            ghcr.io/${{ github.repository }}:latest
```

#### 第七步：测试构建
```bash
# 8. 推送代码并创建标签
git add .
git commit -m "Initial commit"
git push origin main

# 创建版本标签
git tag v1.0.0
git push origin v1.0.0

# 9. 检查构建结果
# 访问：https://github.com/用户名/smart-forward/actions
# 查看构建进度和结果
```

### 权限需求总结

| 操作 | 所需权限 | 配置位置 |
|------|----------|----------|
| **创建仓库** | 无 | GitHub 网站 |
| **推送代码** | `contents: write` | 仓库权限 |
| **构建镜像** | `packages: write` | Workflow 权限 |
| **推送镜像** | `GHCR_TOKEN` | Secrets |
| **创建 Release** | `contents: write` | Workflow 权限 |

---

## 🔐 仓库权限配置

### 1. 基本仓库权限

**访问路径**：`仓库 Settings` → `General`

**权限级别**：
- **Read** - 只能查看代码
- **Write** - 可以推送代码，创建分支
- **Admin** - 完全控制权限
- **Maintain** - 可以管理 issues 和 PR

### 2. 分支保护规则

**访问路径**：`仓库 Settings` → `Branches`

**推荐配置**：
```yaml
# 主分支保护
main:
  - Require a pull request before merging
  - Require status checks to pass before merging
  - Require branches to be up to date before merging
  - Require linear history
  - Restrict pushes that create files
```

---

## 🔑 Secrets 管理

### 1. 仓库 Secrets

**访问路径**：`仓库 Settings` → `Secrets and variables` → `Actions`

**常用 Secrets**：

| Secret 名称 | 用途 | 示例值 |
|-------------|------|--------|
| `GHCR_TOKEN` | Docker 镜像推送 | `ghp_xxxxxxxxxxxx` |
| `GITHUB_TOKEN` | 默认权限 | 自动生成 |
| `NPM_TOKEN` | NPM 包发布 | `npm_xxxxxxxxxxxx` |
| `DOCKER_USERNAME` | Docker Hub 登录 | `your-username` |
| `DOCKER_PASSWORD` | Docker Hub 密码 | `your-password` |

### 2. 环境 Secrets

**访问路径**：`仓库 Settings` → `Environments`

**环境类型**：
- **Production** - 生产环境
- **Staging** - 测试环境
- **Development** - 开发环境

### 3. 组织 Secrets

**访问路径**：`组织 Settings` → `Secrets and variables` → `Actions`

**用途**：在多个仓库间共享 Secrets

---

## ⚙️ Workflow 权限设置

### 1. 全局权限配置

**访问路径**：`仓库 Settings` → `Actions` → `General`

**权限选项**：

#### 选项 1：Read and write permissions（推荐）
```yaml
permissions:
  contents: write
  packages: write
  actions: read
  security-events: write
```

#### 选项 2：Read repository contents and packages permissions
```yaml
permissions:
  contents: read
  packages: read
```

### 2. Workflow 文件权限配置

**在 `.github/workflows/*.yml` 中配置**：

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
      id-token: write
      actions: read
```

### 3. 权限说明

| 权限 | 说明 | 用途 |
|------|------|------|
| `contents: read` | 读取仓库内容 | 检出代码 |
| `contents: write` | 写入仓库内容 | 推送代码、创建 Release |
| `packages: read` | 读取包 | 拉取 Docker 镜像 |
| `packages: write` | 写入包 | 推送 Docker 镜像 |
| `id-token: write` | 写入 ID Token | OIDC 认证 |
| `actions: read` | 读取 Actions | 查看其他 workflow |
| `security-events: write` | 写入安全事件 | 安全扫描 |

---

## 📦 包管理权限

### 1. GitHub Container Registry (GHCR)

**访问路径**：`仓库 Settings` → `Packages`

**权限配置**：
- **Visibility**：Public / Private
- **Actions access**：允许 Actions 访问
- **API access**：允许 API 访问

### 2. 包权限设置

**访问路径**：`包页面` → `Package settings`

**权限级别**：
- **Read** - 拉取包
- **Write** - 推送包
- **Admin** - 管理包

### 3. 包可见性

| 可见性 | 说明 | 适用场景 |
|--------|------|----------|
| **Public** | 公开可见 | 开源项目 |
| **Private** | 私有 | 内部项目 |
| **Internal** | 组织内可见 | 企业项目 |

---

## 🐳 Docker 镜像构建权限

### 1. 构建权限需求

**基本权限**：
```yaml
permissions:
  contents: read      # 读取代码
  packages: write     # 推送镜像到 GHCR
  id-token: write     # OIDC 认证
  actions: read       # 读取其他 workflow
```

### 2. GHCR 权限配置

**访问路径**：`仓库 Settings` → `Packages`

**权限设置**：
- **Visibility**：Public（开源项目）或 Private（私有项目）
- **Actions access**：允许 Actions 访问包
- **API access**：允许 API 访问包

### 3. 镜像推送流程

```yaml
# 1. 登录到 GHCR
- name: Login to GHCR
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.actor }}
    password: ${{ secrets.GHCR_TOKEN }}

# 2. 构建并推送镜像
- name: Build and push
  uses: docker/build-push-action@v5
  with:
    context: .
    file: Dockerfile
    platforms: linux/amd64,linux/arm64
    push: true
    tags: |
      ghcr.io/${{ github.repository }}:${{ github.ref_name }}
      ghcr.io/${{ github.repository }}:latest
```

### 4. 镜像大小优化

**静态链接配置**：
```dockerfile
# Dockerfile.tiny
FROM rust:1.88-alpine AS builder
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-s"
# 构建静态链接程序

FROM scratch
COPY --from=builder /app/target/release/app /app
ENTRYPOINT ["/app"]
```

**镜像大小对比**：
| 方案 | 基础镜像 | 程序大小 | 总大小 |
|------|----------|----------|--------|
| 动态链接 | debian:bullseye-slim (80MB) | 3MB | ~83MB |
| 静态链接 | scratch (0MB) | 8MB | ~8MB |

### 5. 多架构构建

**支持的架构**：
- `linux/amd64` - Intel/AMD 64位
- `linux/arm64` - ARM 64位
- `linux/arm/v7` - ARM 32位

**配置示例**：
```yaml
platforms: linux/amd64,linux/arm64
```

### 6. 常见问题

**问题 1：403 Forbidden**
```bash
# 原因：GHCR_TOKEN 权限不足
# 解决：检查 token 权限，确保有 write:packages
```

**问题 2：镜像推送失败**
```bash
# 原因：包权限设置错误
# 解决：检查包可见性和 Actions 访问权限
```

**问题 3：构建超时**
```bash
# 原因：镜像太大或网络问题
# 解决：使用静态链接减小镜像大小
```

---

## 🏢 组织权限配置

### 1. 组织成员权限

**访问路径**：`组织 Settings` → `People`

**角色级别**：
- **Owner** - 完全控制
- **Member** - 基本权限
- **Billing manager** - 账单管理

### 2. 团队权限

**访问路径**：`组织 Settings` → `Teams`

**团队类型**：
- **Public** - 公开团队
- **Private** - 私有团队
- **Secret** - 秘密团队

### 3. 组织 Secrets

**访问路径**：`组织 Settings` → `Secrets and variables` → `Actions`

**用途**：在多个仓库间共享敏感信息

---

## 🚨 常见问题解决

### 1. 403 Forbidden 错误

**原因**：权限不足

**解决方案**：
```bash
# 检查权限配置
1. 确认仓库权限设置
2. 检查 Workflow 权限
3. 验证 Secrets 配置
4. 确认包权限设置
```

### 2. 401 Unauthorized 错误

**原因**：认证失败

**解决方案**：
```bash
# 检查认证
1. 验证 Personal Access Token
2. 确认 Token 权限
3. 检查用户名和 Token 匹配
4. 验证 Token 是否过期
```

### 3. 推送保护错误

**原因**：代码中包含敏感信息

**解决方案**：
```bash
# 清理敏感信息
1. 移除代码中的 Token
2. 使用 git reset 重置
3. 重新提交安全代码
4. 配置 .gitignore
```

### 4. 包推送失败

**原因**：包权限不足

**解决方案**：
```bash
# 配置包权限
1. 检查包可见性设置
2. 确认 Actions 访问权限
3. 验证 GHCR_TOKEN 权限
4. 检查包名称格式
```

---

## 📝 最佳实践

### 1. 权限最小化原则

```yaml
# 只给必要的权限
permissions:
  contents: read  # 只需要读取，不给写入
  packages: write # 需要推送包
```

### 2. Secrets 管理

```yaml
# 使用环境变量
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  GHCR_TOKEN: ${{ secrets.GHCR_TOKEN }}
```

### 3. 分支保护

```yaml
# 主分支保护
main:
  - Require pull request reviews
  - Require status checks
  - Require up-to-date branches
```

### 4. 定期轮换 Secrets

```bash
# 定期更新 Token
1. 创建新的 Personal Access Token
2. 更新仓库 Secrets
3. 删除旧的 Token
4. 测试新 Token 功能
```

---

## 🔧 配置检查清单

### 仓库配置
- [ ] 仓库权限设置为适当级别
- [ ] 分支保护规则已配置
- [ ] 包权限设置正确
- [ ] Workflow 权限配置完整

### Secrets 配置
- [ ] 必要的 Secrets 已添加
- [ ] Secrets 权限正确
- [ ] 环境 Secrets 已配置
- [ ] 定期轮换 Secrets

### Workflow 配置
- [ ] 权限配置最小化
- [ ] 使用最新版本的 Actions
- [ ] 错误处理完善
- [ ] 日志记录详细

### 安全配置
- [ ] 代码中无敏感信息
- [ ] .gitignore 配置完整
- [ ] 依赖项安全扫描
- [ ] 定期安全审计

---

## 📞 技术支持

### 相关链接
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [GitHub Packages 文档](https://docs.github.com/en/packages)
- [GitHub Security 文档](https://docs.github.com/en/security)
- [Personal Access Tokens](https://github.com/settings/tokens)

### 常见命令
```bash
# 检查仓库权限
gh repo view --json permissions

# 列出 Secrets
gh secret list

# 检查包权限
gh api repos/:owner/:repo/packages
```

---

## 🎯 总结

正确的权限配置是 GitHub 项目成功的关键：

1. **最小权限原则** - 只给必要的权限
2. **定期审计** - 定期检查权限配置
3. **安全第一** - 保护敏感信息
4. **文档记录** - 记录所有配置变更

遵循这些最佳实践，可以确保您的 GitHub 项目安全、高效地运行。
