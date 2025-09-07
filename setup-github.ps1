# ================================
# GitHub 仓库设置脚本
# ================================
# 功能：自动化设置 GitHub 仓库和 Actions

param(
    [string]$RepoName = "smart-forward",
    [string]$Description = "智能网络转发器 - 支持TCP/UDP/HTTP协议转发",
    [switch]$CreateRepo = $false,
    [switch]$SetupActions = $false
)

# 颜色输出
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "🚀 GitHub 仓库设置向导" "Cyan"
Write-ColorOutput "仓库名称: $RepoName" "Yellow"
Write-ColorOutput "描述: $Description" "Yellow"

# 1. 检查 Git 状态
Write-ColorOutput "`n📋 检查 Git 状态..." "Cyan"

if (-not (Test-Path ".git")) {
    Write-ColorOutput "初始化 Git 仓库..." "Yellow"
    git init
    git add .
    git commit -m "Initial commit: Smart Forward project setup"
} else {
    Write-ColorOutput "✅ Git 仓库已存在" "Green"
    git status
}

# 2. 创建 .gitignore
Write-ColorOutput "`n📝 创建 .gitignore..." "Cyan"

$gitignore = @"
# Rust
/target/
**/*.rs.bk
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Build artifacts
dist/
*.exe
*.dll
*.so
*.dylib

# Temporary files
*.tmp
*.temp
"@

$gitignore | Out-File -FilePath ".gitignore" -Encoding UTF8
Write-ColorOutput ".gitignore created" "Green"

# 3. 创建 README 徽章
Write-ColorOutput "`n🏷️  更新 README 徽章..." "Cyan"

$readmePath = "README.md"
if (Test-Path $readmePath) {
    $readme = Get-Content $readmePath -Raw
    
    # 添加徽章（如果不存在）
    if ($readme -notmatch "!\[CI\]") {
        $badges = @"

![CI](https://github.com/$env:USERNAME/$RepoName/workflows/CI%20Pipeline/badge.svg)
![Release](https://github.com/$env:USERNAME/$RepoName/workflows/Release/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

"@
        $readme = $badges + "`n" + $readme
        $readme | Out-File -FilePath $readmePath -Encoding UTF8
        Write-ColorOutput "README badges added" "Green"
    }
} else {
    Write-ColorOutput "README.md not found, skipping badges" "Yellow"
}

# 4. 验证 Actions 配置
Write-ColorOutput "`n🔍 验证 GitHub Actions 配置..." "Cyan"

$actionsDir = ".github/workflows"
if (Test-Path $actionsDir) {
    $workflows = Get-ChildItem $actionsDir -Filter "*.yml"
    Write-ColorOutput "找到工作流文件:" "Green"
    foreach ($workflow in $workflows) {
        Write-ColorOutput "  - $($workflow.Name)" "White"
    }
} else {
    Write-ColorOutput "❌ .github/workflows 目录不存在" "Red"
    Write-ColorOutput "请确保已创建 GitHub Actions 配置文件" "Yellow"
}

# 5. 创建发布说明模板
Write-ColorOutput "`n📋 创建发布说明模板..." "Cyan"

$releaseTemplate = @"
# 发布说明

## 版本 $version

### 新增功能
- 

### 修复问题
- 

### 性能优化
- 

### 依赖更新
- 

## 下载

- **Linux x86_64**: [smart-forward-linux-x86_64.tar.gz](https://github.com/$env:USERNAME/$RepoName/releases/download/$version/smart-forward-linux-x86_64.tar.gz)
- **Windows x86_64**: [smart-forward-windows-x86_64.zip](https://github.com/$env:USERNAME/$RepoName/releases/download/$version/smart-forward-windows-x86_64.zip)

## 安装说明

### Linux
```bash
# 下载并解压
wget https://github.com/$env:USERNAME/$RepoName/releases/download/$version/smart-forward-linux-x86_64.tar.gz
tar -xzf smart-forward-linux-x86_64.tar.gz
chmod +x smart-forward-linux-x86_64

# 运行
./smart-forward-linux-x86_64
```

### Windows
```powershell
# 下载并解压
Invoke-WebRequest -Uri "https://github.com/$env:USERNAME/$RepoName/releases/download/$version/smart-forward-windows-x86_64.zip" -OutFile "smart-forward-windows-x86_64.zip"
Expand-Archive -Path "smart-forward-windows-x86_64.zip" -DestinationPath "."

# 运行
.\smart-forward-windows-x86_64.exe
```
"@

$releaseTemplate | Out-File -FilePath "RELEASE-TEMPLATE.md" -Encoding UTF8
Write-ColorOutput "✅ 发布说明模板已创建" "Green"

# 6. 生成 GitHub 命令
Write-ColorOutput "`n📋 生成 GitHub 设置命令..." "Cyan"

$githubCommands = @"

# ================================
# GitHub 仓库设置命令
# ================================

# 1. 创建远程仓库（如果尚未创建）
gh repo create $RepoName --public --description "$Description"

# 2. 添加远程仓库
git remote add origin https://github.com/$env:USERNAME/$RepoName.git

# 3. 推送代码
git branch -M main
git push -u origin main

# 4. 创建第一个标签（触发发布）
git tag v1.0.0
git push origin v1.0.0

# 5. 查看 Actions 状态
gh run list

# 6. 查看仓库
gh repo view $RepoName

"@

$githubCommands | Out-File -FilePath "github-commands.txt" -Encoding UTF8
Write-ColorOutput "GitHub commands saved to github-commands.txt" "Green"

# 7. 显示下一步操作
Write-ColorOutput "`n🎯 下一步操作:" "Cyan"
Write-ColorOutput "1. 安装 GitHub CLI: winget install GitHub.cli" "Yellow"
Write-ColorOutput "2. 登录 GitHub: gh auth login" "Yellow"
Write-ColorOutput "3. 运行生成的命令创建仓库" "Yellow"
Write-ColorOutput "4. 推送代码到 GitHub" "Yellow"
Write-ColorOutput "5. 查看 Actions 运行状态" "Yellow"

Write-ColorOutput "`n📁 生成的文件:" "Cyan"
Write-ColorOutput "- .gitignore" "White"
Write-ColorOutput "- RELEASE-TEMPLATE.md" "White"
Write-ColorOutput "- github-commands.txt" "White"

Write-ColorOutput "`nSetup completed! Now you can enjoy free GitHub Actions!" "Green"
