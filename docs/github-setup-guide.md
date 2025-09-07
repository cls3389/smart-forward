# 🚀 GitHub Actions 构建配置指南 (小白版)

## 📋 **概述**

本指南将帮助您配置GitHub仓库，让自动构建和发布功能正常工作。

## 🎯 **需要配置的权限**

### **1. GitHub Actions 权限**
### **2. GitHub Container Registry (GHCR) 权限**  
### **3. Release 发布权限**

---

## 🔧 **第一步: 启用 GitHub Actions**

### 1.1 进入仓库设置
1. 打开您的GitHub仓库页面
2. 点击 **Settings** (设置) 标签
3. 在左侧菜单找到 **Actions** → **General**

### 1.2 启用Actions权限
```
✅ Allow all actions and reusable workflows
```
或者选择：
```
✅ Allow [your organization] actions and reusable workflows
```

### 1.3 工作流权限设置
在同一页面找到 **Workflow permissions**：
```
✅ Read and write permissions
✅ Allow GitHub Actions to create and approve pull requests
```

---

## 🐳 **第二步: 配置 Container Registry 权限**

### 2.1 包权限设置
1. 进入 **Settings** → **Actions** → **General**
2. 找到 **Workflow permissions** 部分
3. 确保选择了：
   ```
   ✅ Read and write permissions
   ```

### 2.2 个人访问令牌 (可选，推荐用内置TOKEN)
如果使用内置的 `GITHUB_TOKEN` 不够，可以创建个人令牌：

1. 点击右上角头像 → **Settings**
2. 左侧菜单 → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
3. 点击 **Generate new token (classic)**
4. 设置权限：
   ```
   ✅ repo (完整仓库权限)
   ✅ write:packages (包写入权限)
   ✅ read:packages (包读取权限)
   ```
5. 复制生成的令牌

### 2.3 添加Secret (如果使用个人令牌)
1. 回到仓库 → **Settings** → **Secrets and variables** → **Actions**
2. 点击 **New repository secret**
3. 名称: `GHCR_TOKEN`
4. 值: 粘贴刚才复制的令牌

---

## 📦 **第三步: 验证配置**

### 3.1 检查权限
在仓库设置中确认：
- ✅ Actions 已启用
- ✅ 工作流有读写权限
- ✅ 可以创建和批准PR

### 3.2 测试构建
1. 修改任意源代码文件
2. 提交并推送到 `main` 分支
3. 查看 **Actions** 标签页，应该看到CI工作流运行

### 3.3 测试发布
1. 创建版本标签：
   ```bash
   git tag v1.0.0
   git push --tags
   ```
2. 查看 **Actions** 标签页，应该看到发布工作流运行
3. 构建完成后，检查：
   - **Releases** 页面有新版本
   - **Packages** 页面有Docker镜像

---

## 🚨 **常见问题排查**

### 问题1: "Permission denied" 错误
**症状**: Actions运行失败，显示权限被拒绝
**解决**: 
1. 检查 **Workflow permissions** 是否设置为 "Read and write"
2. 确保Actions已启用

### 问题2: Docker推送失败
**症状**: Docker镜像构建成功但推送失败
**解决**:
1. 检查是否有 `write:packages` 权限
2. 确认仓库名称正确 (小写)

### 问题3: Release创建失败
**症状**: 二进制文件构建成功但Release创建失败
**解决**:
1. 检查是否有 `contents: write` 权限
2. 确认标签格式正确 (v开头)

### 问题4: 构建超时
**症状**: 构建运行很长时间后超时
**解决**:
1. 检查网络连接
2. 查看是否有依赖下载问题
3. 考虑增加超时时间

---

## 🎯 **快速检查清单**

在开始构建前，请确认：

- [ ] ✅ GitHub Actions 已启用
- [ ] ✅ 工作流权限设置为 "Read and write"  
- [ ] ✅ 允许Actions创建和批准PR
- [ ] ✅ 仓库名称是小写 (Docker要求)
- [ ] ✅ 有 `.github/workflows/` 目录和工作流文件
- [ ] ✅ `Cargo.toml` 和 `Dockerfile` 存在

---

## 📞 **获取帮助**

如果遇到问题：

1. **查看Actions日志**: 点击失败的工作流，查看详细错误信息
2. **检查权限**: 按照本指南重新检查所有权限设置
3. **查看示例**: 参考成功的开源项目配置
4. **社区求助**: 在GitHub Discussions或相关社区提问

---

## 🎉 **成功标志**

配置成功后，您应该看到：

1. **CI工作流**: 每次推送代码时自动运行
2. **发布工作流**: 推送标签时自动构建所有平台
3. **Docker镜像**: 在Packages页面可以看到
4. **Release页面**: 有二进制文件下载链接

恭喜！您的自动化构建和发布系统已经配置完成！🎊
