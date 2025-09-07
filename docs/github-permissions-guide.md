# 🔧 GitHub Container Registry 权限配置指南

## 🚨 **403 Forbidden 错误解决方案**

如果您在GitHub Actions中遇到以下错误：
```
ERROR: failed to push ghcr.io/cls3389/smart-forward:v1.0.0: 
unexpected status from HEAD request: 403 Forbidden
```

这是GitHub Container Registry (GHCR) 权限问题，请按以下步骤解决。

---

## 🎯 **快速解决方案 (推荐)**

### **方案1: 删除现有包并重新构建**

1. **删除现有包**
   - 进入 GitHub → 您的头像 → **Your profile** → **Packages**
   - 找到 `smart-forward` 包 (如果存在)
   - 点击包名 → **Package settings** → **Delete package**
   - 确认删除

2. **重新触发构建**
   ```bash
   # 创建新标签重新构建
   git tag v1.0.1
   git push --tags
   ```

### **方案2: 配置包权限**

1. **进入包设置**
   - GitHub → Your profile → **Packages**
   - 点击 `smart-forward` 包名
   - 点击 **Package settings**

2. **添加仓库权限**
   - 找到 **Manage Actions access** 部分
   - 点击 **Add Repository**
   - 输入: `cls3389/smart-forward`
   - 选择权限: **Write**
   - 点击 **Add**

---

## 🔧 **详细诊断步骤**

### **第一步: 检查仓库权限**

1. **进入仓库设置**
   - 仓库页面 → **Settings** → **Actions** → **General**

2. **配置工作流权限**
   ```
   Workflow permissions:
   ✅ Read and write permissions
   ✅ Allow GitHub Actions to create and approve pull requests
   ```

### **第二步: 检查包可见性**

1. **进入包设置**
   - GitHub → Your profile → **Packages** → `smart-forward`

2. **设置包可见性**
   ```
   Package visibility:
   ✅ Public (推荐)
   或
   ✅ Private (需要配置访问权限)
   ```

### **第三步: 验证工作流配置**

检查 `.github/workflows/release.yml` 中的配置：

```yaml
# ✅ 正确的权限配置
build-docker:
  permissions:
    contents: read
    packages: write  # 必需！

# ✅ 正确的登录配置
- name: 登录 GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}  # 使用内置token
```

---

## 🛠️ **高级解决方案**

### **方案3: 使用个人访问令牌**

如果内置 `GITHUB_TOKEN` 不工作，可以创建个人令牌：

1. **创建 Personal Access Token**
   - GitHub → Settings → **Developer settings**
   - **Personal access tokens** → **Tokens (classic)**
   - **Generate new token (classic)**

2. **设置权限**
   ```
   ✅ repo (Full control of private repositories)
   ✅ write:packages (Write packages to GitHub Package Registry)
   ✅ read:packages (Read packages from GitHub Package Registry)
   ✅ delete:packages (Delete packages from GitHub Package Registry)
   ```

3. **添加到仓库 Secrets**
   - 仓库 → **Settings** → **Secrets and variables** → **Actions**
   - **New repository secret**
   - Name: `GHCR_TOKEN`
   - Value: 粘贴生成的令牌

4. **修改工作流**
   ```yaml
   - name: 登录 GitHub Container Registry
     uses: docker/login-action@v3
     with:
       registry: ghcr.io
       username: ${{ github.actor }}
       password: ${{ secrets.GHCR_TOKEN }}  # 使用个人令牌
   ```

---

## 🧪 **测试和验证**

### **测试权限配置**

1. **手动测试登录**
   ```bash
   # 使用 GitHub CLI 测试
   echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
   ```

2. **检查包权限**
   ```bash
   # 尝试推送测试镜像
   docker tag hello-world ghcr.io/cls3389/test:latest
   docker push ghcr.io/cls3389/test:latest
   ```

### **验证构建成功**

构建成功后，您应该看到：

1. **GitHub Actions** 显示绿色 ✅
2. **Packages** 页面出现 `smart-forward` 包
3. **Release** 页面有新的发布版本

---

## 🚨 **常见问题排查**

### **问题1: 仓库名称大小写**
```
错误: ghcr.io/CLS3389/smart-forward  ❌
正确: ghcr.io/cls3389/smart-forward  ✅
```

### **问题2: 网络连接问题**
```bash
# 检查网络连接
curl -I https://ghcr.io/v2/

# 检查 GitHub 服务状态
curl -I https://www.githubstatus.com/
```

### **问题3: 令牌过期**
- 检查个人访问令牌是否过期
- 重新生成令牌并更新 Secrets

### **问题4: 组织权限**
如果仓库属于组织：
- 确保组织允许 GitHub Packages
- 检查组织的包权限设置

---

## 📋 **检查清单**

在重新构建前，请确认：

- [ ] ✅ 仓库工作流权限设置为 "Read and write"
- [ ] ✅ 包不存在或已正确配置权限
- [ ] ✅ 工作流文件包含 `packages: write` 权限
- [ ] ✅ 使用正确的登录凭据 (GITHUB_TOKEN 或 GHCR_TOKEN)
- [ ] ✅ 仓库名称全部小写
- [ ] ✅ 网络连接正常

---

## 🎯 **成功标志**

权限配置成功后，您将看到：

1. **GitHub Actions** 构建成功 ✅
2. **Docker镜像** 推送到 GHCR ✅
3. **Release页面** 包含所有构建产物 ✅
4. **可以正常拉取镜像**:
   ```bash
   docker pull ghcr.io/cls3389/smart-forward:latest
   ```

如果仍有问题，请检查 GitHub Actions 的详细日志获取更多信息。
