# 📁 项目结构说明

## 🎯 **目录概览**

```
smart-forward/
├── 📁 .github/workflows/     # GitHub Actions 工作流
│   ├── ci.yml               # 持续集成 (代码检查+测试)
│   └── release.yml          # 全平台发布 (二进制+Docker+Release)
├── 📁 docs/                 # 完整文档体系 (64000+字)
│   ├── README.md            # 📚 文档索引和导航
│   ├── configuration.md     # 📋 配置选项和API文档
│   ├── deployment.md        # 🚀 部署指南 (Docker/K8s/systemd)
│   ├── performance.md       # ⚡ 性能优化指南
│   ├── security.md          # 🛡️ 安全配置指南
│   ├── examples.md          # 📝 使用示例和案例 (5个场景)
│   ├── docker.md            # 🐳 Docker使用说明 (8MB Alpine)
│   ├── github-setup-guide.md    # 🔧 GitHub权限配置 (小白版)
│   ├── build-troubleshooting.md # 🚨 构建问题排查指南
│   └── openwrt-installation.md  # 📡 OpenWrt安装指南
├── 📁 scripts/              # 实用脚本
│   └── openwrt-install.sh   # OpenWrt自动安装脚本
├── 📁 src/                  # Rust源代码
│   ├── main.rs              # 程序入口
│   ├── config.rs            # 配置管理
│   ├── forwarder.rs         # 转发核心逻辑
│   ├── common.rs            # 公共模块
│   └── utils.rs             # 工具函数
├── 🐳 Dockerfile            # 8MB Alpine镜像构建
├── 📦 Cargo.toml            # Rust项目配置 (v1.0.0)
├── 🔒 Cargo.lock            # 依赖锁定文件
├── 🔧 config.yaml.example   # 配置文件模板
├── 🐳 docker-compose.yml    # Docker Compose配置
├── 🛡️ deny.toml             # 安全审计配置
├── 💻 build.ps1             # 本地Docker构建脚本
├── 📋 README.md             # 项目主说明文档
└── 🚫 .gitignore            # Git忽略规则
```

---

## 📂 **目录详细说明**

### **🔧 核心配置文件**

| 文件 | 用途 | 重要性 |
|------|------|--------|
| `Cargo.toml` | Rust项目配置、依赖管理 | ⭐⭐⭐⭐⭐ |
| `Cargo.lock` | 依赖版本锁定 | ⭐⭐⭐⭐⭐ |
| `config.yaml.example` | 配置文件模板 | ⭐⭐⭐⭐ |
| `deny.toml` | 安全审计规则 | ⭐⭐⭐ |
| `.gitignore` | Git忽略规则 | ⭐⭐⭐ |

### **🐳 容器化文件**

| 文件 | 用途 | 特色 |
|------|------|------|
| `Dockerfile` | 8MB Alpine镜像构建 | 极致优化 |
| `docker-compose.yml` | 容器编排配置 | 生产就绪 |
| `build.ps1` | 本地构建脚本 | Windows友好 |

### **📁 源代码结构 (`src/`)**

```rust
src/
├── main.rs          // 程序入口，CLI参数处理
├── config.rs        // 配置文件解析和验证
├── forwarder.rs     // 核心转发逻辑 (TCP/UDP/HTTP)
├── common.rs        // 公共类型定义和常量
└── utils.rs         // 工具函数 (日志、网络等)
```

### **📚 文档体系 (`docs/`)**

#### **🎯 按用户类型分类**

| 用户类型 | 推荐文档 | 说明 |
|----------|----------|------|
| **🆕 新手** | `github-setup-guide.md` | 零基础配置指南 |
| **👨‍💻 开发者** | `configuration.md` | API和配置详解 |
| **🚀 运维** | `deployment.md` | 生产环境部署 |
| **🏢 企业** | `security.md` | 安全合规配置 |
| **🎮 个人** | `examples.md` | 实际使用案例 |

#### **📊 文档统计**

| 文档类型 | 文件数 | 字数 | 覆盖范围 |
|----------|--------|------|----------|
| **配置指南** | 2个 | 15000+ | 完整API说明 |
| **部署指南** | 2个 | 15000+ | 多种部署方式 |
| **优化指南** | 2个 | 21000+ | 性能+安全 |
| **使用案例** | 1个 | 15000+ | 5个实际场景 |
| **问题排查** | 2个 | 8000+ | 错误解决方案 |
| **总计** | **10个** | **64000+** | **全面覆盖** |

### **🔄 CI/CD 工作流 (`.github/workflows/`)**

#### **工作流设计**

```yaml
CI流程 (ci.yml):
  触发: push到main/develop分支, PR
  内容: 代码检查 + 测试 + 安全审计
  时间: ~5-8分钟

Release流程 (release.yml):
  触发: 推送版本标签 (v*)
  内容: 
    1️⃣ 多平台二进制构建 (5个平台)
    2️⃣ Docker多架构构建 (AMD64/ARM64)
    3️⃣ 自动创建GitHub Release
  时间: ~20-25分钟
```

#### **构建产物**

| 平台 | 架构 | 文件名 |
|------|------|--------|
| Windows | x86_64 | `smart-forward-windows-x86_64.zip` |
| macOS | x86_64 | `smart-forward-macos-x86_64.tar.gz` |
| macOS | ARM64 | `smart-forward-macos-aarch64.tar.gz` |
| Linux | x86_64 | `smart-forward-linux-x86_64.tar.gz` |
| Linux | ARM64 | `smart-forward-linux-aarch64.tar.gz` |
| Docker | Multi-Arch | `ghcr.io/cls3389/smart-forward:latest` |

---

## 🎯 **项目特色**

### **📦 极致优化**
- ✅ **8MB Docker镜像** (Alpine 3.18 + 极致编译优化)
- ✅ **零重复构建** (智能缓存 + 并行构建)
- ✅ **5个平台支持** (Windows/macOS/Linux x64/ARM64)

### **📚 完整文档**
- ✅ **64000+字专业文档** (10个文档文件)
- ✅ **小白友好指南** (详细步骤 + 错误排查)
- ✅ **企业级配置** (安全 + 性能 + 部署)

### **🔧 开发友好**
- ✅ **现代化工具链** (Rust 2021 + GitHub Actions)
- ✅ **完整的CI/CD** (自动测试 + 多平台构建)
- ✅ **安全审计** (cargo-audit + deny.toml)

### **🚀 生产就绪**
- ✅ **容器化部署** (Docker + Kubernetes)
- ✅ **系统服务** (systemd + 自启动)
- ✅ **监控告警** (结构化日志 + 健康检查)

---

## 📋 **文件管理规则**

### **✅ 应该提交的文件**
- 所有源代码 (`src/`)
- 配置文件 (`Cargo.toml`, `Dockerfile`, etc.)
- 文档文件 (`docs/`, `README.md`)
- 工作流文件 (`.github/workflows/`)
- 脚本文件 (`scripts/`, `build.ps1`)
- 示例文件 (`config.yaml.example`)

### **❌ 不应该提交的文件**
- 编译产物 (`target/`, `*.exe`, `*.dll`)
- 本地配置 (`config.yaml`)
- 临时文件 (`*.tmp`, `*.log`)
- IDE文件 (`.vscode/`, `.idea/`)
- 构建缓存 (`logs/`, `dist/`)

### **🔄 自动忽略规则**
```gitignore
# 编译产物
/target/
*.exe, *.dll, *.so, *.dylib

# 本地配置
config.yaml

# 临时文件
*.tmp, *.log, logs/

# IDE和OS文件
.vscode/, .idea/, .DS_Store
```

---

## 🎯 **最佳实践**

### **📁 目录组织**
1. **按功能分类** - 源码、文档、脚本分离
2. **层次清晰** - 避免过深的目录嵌套
3. **命名规范** - 使用描述性的文件和目录名

### **📝 文档管理**
1. **集中管理** - 所有文档放在 `docs/` 目录
2. **索引导航** - 提供 `docs/README.md` 作为入口
3. **按需查找** - 支持按角色和场景查找

### **🔧 配置管理**
1. **示例优先** - 提供 `.example` 模板文件
2. **环境分离** - 本地配置不提交到仓库
3. **文档同步** - 配置变更及时更新文档

这个项目结构体现了**现代化Rust项目的最佳实践**，既适合个人使用，也满足企业级需求！🚀
