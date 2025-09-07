# GitHub Container Registry 权限问题解决方案

## 问题描述
Docker 镜像推送到 `ghcr.io` 时出现 403 Forbidden 错误。

## 解决步骤

### 1. 检查仓库包权限设置

1. 访问您的 GitHub 仓库：`https://github.com/cls3389/smart-forward`
2. 点击 **Settings** 标签
3. 在左侧菜单中找到 **Packages** 或 **Actions and packages**
4. 确保包权限设置为：
   - **Write** 或 **Admin** 权限
   - 允许 Actions 访问包

### 2. 检查 GITHUB_TOKEN 权限

在仓库设置中：
1. 进入 **Settings** → **Actions** → **General**
2. 滚动到 **Workflow permissions** 部分
3. 确保选择：
   - **Read and write permissions**
   - **Allow GitHub Actions to create and approve pull requests**

### 3. 验证包访问权限

1. 访问 `https://github.com/orgs/cls3389/packages`
2. 找到 `smart-forward` 包
3. 点击包名称进入包设置
4. 在 **Manage Actions access** 中确保：
   - 仓库 `cls3389/smart-forward` 有 **Write** 权限

### 4. 手动测试推送权限

在本地运行以下命令测试权限：

```bash
# 登录到 GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u cls3389 --password-stdin

# 构建测试镜像
docker build -t ghcr.io/cls3389/smart-forward:test .

# 推送测试镜像
docker push ghcr.io/cls3389/smart-forward:test
```

### 5. 如果仍然失败，尝试以下解决方案

#### 方案 A：使用 Personal Access Token
1. 创建 Personal Access Token (PAT)：
   - 访问 `https://github.com/settings/tokens`
   - 创建新 token，权限包括：`write:packages`, `read:packages`, `delete:packages`
2. 在仓库 Secrets 中添加 `GHCR_TOKEN`
3. 更新 workflow 使用 PAT：

```yaml
- name: 登录到 GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.actor }}
    password: ${{ secrets.GHCR_TOKEN }}
```

#### 方案 B：检查包可见性
确保包设置为 **Public** 或您的账户有访问权限。

#### 方案 C：清理并重新创建包
如果包损坏，可以：
1. 删除现有包
2. 重新推送创建新包

## 验证修复

运行以下命令验证修复：

```bash
# 检查包是否存在
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://ghcr.io/v2/cls3389/smart-forward/manifests/latest

# 应该返回 200 OK 而不是 403 Forbidden
```

## 常见问题

1. **包不存在**：首次推送需要手动创建包或确保有创建权限
2. **权限不足**：GITHUB_TOKEN 权限不够，需要 PAT
3. **包可见性**：包设置为私有但 token 无访问权限
4. **组织权限**：组织级别的包权限设置问题
