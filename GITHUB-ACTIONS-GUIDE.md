# 🚀 GitHub Actions 使用指南

## 📋 概述

本项目已配置了完整的 GitHub Actions CI/CD 流程，包括：
- ✅ 代码质量检查（格式化、Clippy）
- ✅ 自动化测试
- ✅ 多平台构建（Linux、Windows）
- ✅ 安全扫描
- ✅ 自动发布

## 🔧 工作流配置

### 1. CI 流程 (`ci.yml`)
**触发条件：**
- 推送到 `main` 或 `develop` 分支
- 创建 Pull Request 到 `main` 分支
- 手动触发

**优化特性：**
- 🚀 快速失败机制（格式化检查优先）
- 💾 智能缓存（Rust 依赖、构建产物）
- ⏱️ 严格超时控制（5-15分钟）
- 🎯 路径过滤（忽略文档文件变更）

### 2. 发布流程 (`release.yml`)
**触发条件：**
- 推送版本标签（如 `v1.0.0`）
- 手动触发（可指定版本）

**功能：**
- 📦 多平台构建
- 🏷️ 自动创建 GitHub Release
- 📁 生成发布包（tar.gz、zip）

## 💰 计费说明

### 公开仓库优势
- ✅ **无限免费使用** GitHub Actions
- ✅ 无时间限制
- ✅ 无并发限制
- ✅ 包含所有功能

### 私有仓库限制
- ❌ 每月仅 2,000 分钟免费
- ❌ 超出后按使用量计费
- ❌ Linux: $0.008/分钟
- ❌ Windows: $0.016/分钟

## 🛠️ 本地开发

### 使用本地 CI 脚本
```powershell
# 完整 CI 流程
.\local-ci.ps1

# 跳过测试
.\local-ci.ps1 -SkipTests

# 只构建 Windows 版本
.\local-ci.ps1 -Target windows

# 详细输出
.\local-ci.ps1 -Verbose
```

### 手动检查命令
```bash
# 代码格式化
cargo fmt -- --check

# Clippy 检查
cargo clippy -- -D warnings

# 运行测试
cargo test

# 安全扫描
cargo audit

# 构建发布版本
cargo build --release
```

## 📊 工作流状态

### 查看运行状态
1. 访问 GitHub 仓库页面
2. 点击 "Actions" 标签
3. 查看工作流运行历史

### 工作流徽章
在 README.md 中添加状态徽章：

```markdown
![CI](https://github.com/用户名/仓库名/workflows/CI%20Pipeline/badge.svg)
![Release](https://github.com/用户名/仓库名/workflows/Release/badge.svg)
```

## 🔍 故障排除

### 常见问题

1. **工作流未触发**
   - 检查文件路径是否在 `.github/workflows/` 目录
   - 确认 YAML 语法正确
   - 检查触发条件是否匹配

2. **构建失败**
   - 查看 Actions 日志
   - 检查 Rust 工具链版本
   - 确认依赖项配置

3. **缓存问题**
   - 清除 Actions 缓存
   - 更新 `Cargo.lock` 文件
   - 重新触发工作流

### 调试技巧

1. **启用详细日志**
   ```yaml
   - name: Debug
     run: |
       echo "Debug info"
       cargo --version
       rustc --version
   ```

2. **本地复现问题**
   ```bash
   # 使用与 CI 相同的环境
   docker run --rm -v $(pwd):/workspace -w /workspace rust:latest bash
   ```

## 🚀 最佳实践

### 1. 提交规范
- 使用清晰的提交信息
- 避免频繁的小提交
- 使用 Pull Request 进行代码审查

### 2. 分支策略
- `main`: 生产环境代码
- `develop`: 开发环境代码
- `feature/*`: 功能分支
- `hotfix/*`: 紧急修复

### 3. 标签管理
- 使用语义化版本号（如 `v1.0.0`）
- 为重要版本创建标签
- 标签会自动触发发布流程

## 📈 性能优化

### 已实现的优化
- ✅ 依赖缓存（减少构建时间）
- ✅ 条件触发（避免不必要的运行）
- ✅ 并行作业（提高效率）
- ✅ 超时控制（防止资源浪费）

### 进一步优化建议
- 使用 `cargo-chef` 进行更精细的缓存
- 考虑使用 `sccache` 加速编译
- 分离测试和构建作业

## 🔐 安全考虑

### 已配置的安全措施
- ✅ 依赖漏洞扫描
- ✅ 最小权限原则
- ✅ 敏感信息保护

### 安全建议
- 定期更新依赖项
- 使用 `cargo audit` 检查漏洞
- 避免在代码中硬编码密钥

## 📞 支持

如果遇到问题：
1. 查看 GitHub Actions 日志
2. 检查本指南的故障排除部分
3. 使用本地 CI 脚本进行调试
4. 提交 Issue 获取帮助

---

**注意：** 由于项目已设为公开仓库，所有 GitHub Actions 功能均可免费使用，无需担心计费问题。
