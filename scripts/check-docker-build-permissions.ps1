# Docker 构建权限检查脚本
# 专门检查 Docker 镜像构建所需的权限

param(
    [string]$Repository = "cls3389/smart-forward"
)

Write-Host "🐳 Docker 构建权限检查工具" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan

# 检查 GitHub CLI
Write-Host "`n📋 环境检查:" -ForegroundColor Yellow
try {
    $ghVersion = gh --version
    Write-Host "✅ GitHub CLI 已安装" -ForegroundColor Green
} catch {
    Write-Host "❌ GitHub CLI 未安装" -ForegroundColor Red
    Write-Host "请安装: https://cli.github.com/" -ForegroundColor Yellow
    exit 1
}

# 检查认证
Write-Host "`n🔐 认证检查:" -ForegroundColor Yellow
try {
    gh auth status | Out-Null
    Write-Host "✅ GitHub 认证成功" -ForegroundColor Green
} catch {
    Write-Host "❌ GitHub 未认证" -ForegroundColor Red
    Write-Host "请运行: gh auth login" -ForegroundColor Yellow
    exit 1
}

# 检查仓库权限
Write-Host "`n🏠 仓库权限检查:" -ForegroundColor Yellow
try {
    $repoInfo = gh repo view $Repository --json permissions
    $permissions = $repoInfo.permissions
    
    Write-Host "仓库: $Repository" -ForegroundColor White
    Write-Host "管理员权限: $($permissions.admin)" -ForegroundColor $(if($permissions.admin) {"Green"} else {"Red"})
    Write-Host "推送权限: $($permissions.push)" -ForegroundColor $(if($permissions.push) {"Green"} else {"Red"})
    Write-Host "拉取权限: $($permissions.pull)" -ForegroundColor $(if($permissions.pull) {"Green"} else {"Red"})
} catch {
    Write-Host "❌ 无法访问仓库: $Repository" -ForegroundColor Red
    exit 1
}

# 检查 Actions 权限
Write-Host "`n⚙️ Actions 权限检查:" -ForegroundColor Yellow
try {
    $workflows = gh api repos/$Repository/actions/workflows
    if ($workflows.total_count -gt 0) {
        Write-Host "✅ 找到 $($workflows.total_count) 个 workflow" -ForegroundColor Green
    } else {
        Write-Host "⚠️  没有找到 workflow" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ 无法访问 Actions" -ForegroundColor Red
}

# 检查 Secrets
Write-Host "`n🔑 Secrets 检查:" -ForegroundColor Yellow
try {
    $secrets = gh secret list --repo $Repository
    $hasGHCRToken = $secrets -match "GHCR_TOKEN"
    
    if ($hasGHCRToken) {
        Write-Host "✅ 找到 GHCR_TOKEN" -ForegroundColor Green
    } else {
        Write-Host "❌ 缺少 GHCR_TOKEN" -ForegroundColor Red
        Write-Host "请添加: https://github.com/$Repository/settings/secrets/actions" -ForegroundColor Yellow
    }
    
    if ($secrets) {
        Write-Host "所有 Secrets:" -ForegroundColor Gray
        $secrets | ForEach-Object { Write-Host "  - $_" -ForegroundColor Gray }
    }
} catch {
    Write-Host "❌ 无法访问 Secrets" -ForegroundColor Red
}

# 检查包权限
Write-Host "`n📦 包权限检查:" -ForegroundColor Yellow
try {
    $packages = gh api repos/$Repository/packages
    if ($packages) {
        Write-Host "✅ 找到 $($packages.Count) 个包" -ForegroundColor Green
        $packages | ForEach-Object { 
            Write-Host "  - $($_.name): $($_.package_type)" -ForegroundColor Gray
        }
    } else {
        Write-Host "⚠️  没有找到包（首次推送时会自动创建）" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ 无法访问包信息" -ForegroundColor Red
}

# 检查 Dockerfile
Write-Host "`n🐳 Dockerfile 检查:" -ForegroundColor Yellow
try {
    $dockerfiles = gh api repos/$Repository/contents --jq '.[] | select(.name | startswith("Dockerfile")) | .name'
    if ($dockerfiles) {
        Write-Host "✅ 找到 Dockerfile:" -ForegroundColor Green
        $dockerfiles | ForEach-Object { Write-Host "  - $_" -ForegroundColor Gray }
    } else {
        Write-Host "❌ 没有找到 Dockerfile" -ForegroundColor Red
        Write-Host "请添加 Dockerfile 到仓库根目录" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ 无法检查 Dockerfile" -ForegroundColor Red
}

# 提供配置建议
Write-Host "`n💡 Docker 构建配置建议:" -ForegroundColor Cyan
Write-Host "1. 确保仓库 Actions 权限设置为 'Read and write permissions'" -ForegroundColor White
Write-Host "2. 添加 GHCR_TOKEN secret 用于镜像推送" -ForegroundColor White
Write-Host "3. 配置 workflow 权限包含 packages: write" -ForegroundColor White
Write-Host "4. 使用静态链接减小镜像大小" -ForegroundColor White
Write-Host "5. 配置多架构构建支持" -ForegroundColor White

Write-Host "`n🔗 相关链接:" -ForegroundColor Cyan
Write-Host "仓库设置: https://github.com/$Repository/settings" -ForegroundColor Blue
Write-Host "Actions: https://github.com/$Repository/actions" -ForegroundColor Blue
Write-Host "包管理: https://github.com/$Repository/packages" -ForegroundColor Blue
Write-Host "Secrets: https://github.com/$Repository/settings/secrets/actions" -ForegroundColor Blue
Write-Host "详细文档: docs/github-permissions-guide.md" -ForegroundColor Blue

# 检查结果总结
Write-Host "`n📊 检查结果总结:" -ForegroundColor Cyan
$issues = @()

if (-not $permissions.push) { $issues += "缺少推送权限" }
if (-not $hasGHCRToken) { $issues += "缺少 GHCR_TOKEN" }
if (-not $dockerfiles) { $issues += "缺少 Dockerfile" }

if ($issues.Count -eq 0) {
    Write-Host "✅ 所有权限配置正确，可以开始构建！" -ForegroundColor Green
} else {
    Write-Host "❌ 发现 $($issues.Count) 个问题需要解决:" -ForegroundColor Red
    $issues | ForEach-Object { Write-Host "  - $_" -ForegroundColor Yellow }
}
