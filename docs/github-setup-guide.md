# GitHub 操作流程指南

## 📋 目录
1. [仓库权限配置](#仓库权限配置)
2. [Secrets 配置](#secrets-配置)
3. [Workflow 权限设置](#workflow-权限设置)
4. [Docker 镜像推送配置](#docker-镜像推送配置)
5. [常见问题解决](#常见问题解决)

---

## 🔐 仓库权限配置

### 1. 仓库包权限设置

**访问路径**：`仓库 Settings` → `Packages` 或 `Actions and packages`

**配置要求**：
- ✅ 包权限设置为 **Write** 或 **Admin**
- ✅ 允许 Actions 访问包
- ✅ 确保仓库有推送包的权限

### 2. Workflow 权限设置

**访问路径**：`仓库 Settings` → `Actions` → `General`

**配置要求**：
- ✅ 选择 **"Read and write permissions"**
- ✅ 勾选 **"Allow GitHub Actions to create and approve pull requests"**

---

## 🔑 Secrets 配置

### 1. 添加 Personal Access Token

**访问路径**：`仓库 Settings` → `Secrets and variables` → `Actions`

**创建步骤**：
1. 点击 **"New repository secret"**
2. 填写以下信息：
   - **Name**: `GHCR_TOKEN`
   - **Value**: `YOUR_PERSONAL_ACCESS_TOKEN`
3. 点击 **"Add secret"**

### 2. 创建 Personal Access Token（如果需要）

**访问路径**：https://github.com/settings/tokens

**权限设置**：
- ✅ `write:packages` - 推送包
- ✅ `read:packages` - 拉取包
- ✅ `delete:packages` - 删除包（可选）
- ✅ `repo` - 访问仓库

---

## ⚙️ Workflow 权限设置

### 1. 检查 Workflow 文件权限

确保 `.github/workflows/tag-release.yml` 包含正确的权限：

```yaml
permissions:
  contents: read
  packages: write
  id-token: write
  actions: read
```

### 2. Docker 登录配置

```yaml
- name: 登录到 GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.actor }}
    password: ${{ secrets.GHCR_TOKEN }}
    logout: true
```

---

## 🐳 Docker 镜像推送配置

### 1. 本地测试推送

```bash
# 登录 GHCR
echo "YOUR_TOKEN" | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# 构建镜像
docker build -t ghcr.io/YOUR_USERNAME/YOUR_REPO:test .

# 推送镜像
docker push ghcr.io/YOUR_USERNAME/YOUR_REPO:test
```

### 2. 多架构构建配置

```yaml
- name: 构建并推送多架构 Docker 镜像
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

---

## 🚨 常见问题解决

### 1. 403 Forbidden 错误

**原因**：权限不足或认证失败

**解决方案**：
- ✅ 检查包权限设置
- ✅ 确认 GHCR_TOKEN secret 已添加
- ✅ 验证 Personal Access Token 权限

### 2. 401 Unauthorized 错误

**原因**：认证失败

**解决方案**：
- ✅ 检查 Personal Access Token 是否有效
- ✅ 确认 token 有正确的权限
- ✅ 验证用户名和 token 匹配

### 3. Docker 构建失败

**原因**：网络问题或依赖安装失败

**解决方案**：
- ✅ 使用 `--fix-missing` 参数
- ✅ 添加 `--no-install-recommends` 减少依赖
- ✅ 配置代理（如需要）

### 4. 推送保护错误

**原因**：代码中包含敏感信息

**解决方案**：
- ✅ 移除代码中的 token 或密码
- ✅ 使用 git reset 重置到安全提交
- ✅ 重新提交不包含敏感信息的代码

---

## 📝 完整操作流程

### 1. 初始设置
```bash
# 1. 克隆仓库
git clone https://github.com/YOUR_USERNAME/YOUR_REPO.git
cd YOUR_REPO

# 2. 配置 Git
git config user.name "YOUR_USERNAME"
git config user.email "YOUR_EMAIL@example.com"
```

### 2. 权限配置
1. 访问仓库 Settings
2. 配置包权限为 Write
3. 配置 Workflow 权限为 Read and write
4. 添加 GHCR_TOKEN secret

### 3. 推送构建
```bash
# 1. 提交更改
git add .
git commit -m "fix: 修复构建问题"

# 2. 推送代码
git push origin main

# 3. 创建版本标签
git tag v1.0.0
git push origin v1.0.0
```

### 4. 验证构建
1. 访问 Actions 页面
2. 查看构建进度
3. 检查 Docker 镜像推送
4. 验证 GitHub Release 创建

---

## 🔍 验证清单

- [ ] 仓库包权限设置为 Write
- [ ] Workflow 权限设置为 Read and write
- [ ] GHCR_TOKEN secret 已添加
- [ ] Personal Access Token 有正确权限
- [ ] Docker 登录测试成功
- [ ] Workflow 文件权限配置正确
- [ ] 代码中不包含敏感信息

---

## 📞 技术支持

如果遇到问题，请检查：
1. GitHub Actions 日志
2. Docker 构建日志
3. 权限配置是否正确
4. Secrets 是否有效

**相关链接**：
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Docker 登录文档](https://docs.docker.com/engine/reference/commandline/login/)
- [GitHub Container Registry 文档](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
