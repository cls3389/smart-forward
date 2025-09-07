# GitHub Container Registry 权限修复脚本
# 解决 403 Forbidden 错误

Write-Host "🔧 GitHub Container Registry 权限诊断和修复" -ForegroundColor Green
Write-Host ""

# 1. 检查仓库设置
Write-Host "📋 第一步: 检查仓库设置" -ForegroundColor Cyan
Write-Host "请在GitHub仓库中检查以下设置:"
Write-Host ""
Write-Host "1. 进入仓库 Settings → Actions → General" -ForegroundColor Yellow
Write-Host "2. 找到 'Workflow permissions' 部分" -ForegroundColor Yellow
Write-Host "3. 选择 'Read and write permissions'" -ForegroundColor Green
Write-Host "4. 勾选 'Allow GitHub Actions to create and approve pull requests'" -ForegroundColor Green
Write-Host ""

# 2. 检查包权限
Write-Host "📦 第二步: 检查包权限" -ForegroundColor Cyan
Write-Host "1. 进入 GitHub 个人资料 → Packages" -ForegroundColor Yellow
Write-Host "2. 找到 'smart-forward' 包 (如果存在)" -ForegroundColor Yellow
Write-Host "3. 点击包名 → Package settings" -ForegroundColor Yellow
Write-Host "4. 在 'Manage Actions access' 中添加仓库权限:" -ForegroundColor Yellow
Write-Host "   - Repository: cls3389/smart-forward" -ForegroundColor Green
Write-Host "   - Role: Write" -ForegroundColor Green
Write-Host ""

# 3. 检查工作流配置
Write-Host "⚙️ 第三步: 检查工作流配置" -ForegroundColor Cyan
$workflowFile = ".github/workflows/release.yml"
if (Test-Path $workflowFile) {
    Write-Host "✅ 找到工作流文件: $workflowFile" -ForegroundColor Green
    
    # 检查权限配置
    $content = Get-Content $workflowFile -Raw
    if ($content -match "packages:\s*write") {
        Write-Host "✅ 工作流已配置 packages: write 权限" -ForegroundColor Green
    } else {
        Write-Host "❌ 工作流缺少 packages: write 权限" -ForegroundColor Red
        Write-Host "需要在 release.yml 中添加:" -ForegroundColor Yellow
        Write-Host "permissions:" -ForegroundColor White
        Write-Host "  contents: write" -ForegroundColor White
        Write-Host "  packages: write" -ForegroundColor White
    }
    
    # 检查登录配置
    if ($content -match "docker/login-action") {
        Write-Host "✅ 找到 Docker 登录配置" -ForegroundColor Green
        if ($content -match "secrets\.GITHUB_TOKEN") {
            Write-Host "✅ 使用 GITHUB_TOKEN (推荐)" -ForegroundColor Green
        } elseif ($content -match "secrets\.GHCR_TOKEN") {
            Write-Host "⚠️  使用 GHCR_TOKEN (需要检查是否配置)" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "❌ 找不到工作流文件: $workflowFile" -ForegroundColor Red
}
Write-Host ""

# 4. 生成修复建议
Write-Host "🛠️ 修复建议" -ForegroundColor Cyan
Write-Host ""
Write-Host "如果仍然出现 403 错误，请尝试以下步骤:" -ForegroundColor Yellow
Write-Host ""
Write-Host "方案1: 删除现有包 (推荐)" -ForegroundColor Green
Write-Host "1. 进入 GitHub → Your profile → Packages" -ForegroundColor White
Write-Host "2. 找到 'smart-forward' 包" -ForegroundColor White
Write-Host "3. 点击包名 → Package settings → Delete package" -ForegroundColor White
Write-Host "4. 重新运行 GitHub Actions 构建" -ForegroundColor White
Write-Host ""

Write-Host "方案2: 手动配置包权限" -ForegroundColor Green
Write-Host "1. 进入包设置页面" -ForegroundColor White
Write-Host "2. 在 'Manage Actions access' 中:" -ForegroundColor White
Write-Host "   - 添加仓库: cls3389/smart-forward" -ForegroundColor White
Write-Host "   - 设置权限: Write" -ForegroundColor White
Write-Host "3. 保存设置并重新构建" -ForegroundColor White
Write-Host ""

Write-Host "方案3: 使用个人访问令牌 (高级)" -ForegroundColor Green
Write-Host "1. 创建 Personal Access Token:" -ForegroundColor White
Write-Host "   - 进入 Settings → Developer settings → Personal access tokens" -ForegroundColor White
Write-Host "   - 权限: write:packages, read:packages" -ForegroundColor White
Write-Host "2. 在仓库 Secrets 中添加 GHCR_TOKEN" -ForegroundColor White
Write-Host "3. 修改工作流使用 secrets.GHCR_TOKEN" -ForegroundColor White
Write-Host ""

# 5. 快速测试
Write-Host "🧪 快速测试" -ForegroundColor Cyan
Write-Host "修复后，可以通过以下方式测试:" -ForegroundColor Yellow
Write-Host ""
Write-Host "1. 推送一个小的更改触发 CI" -ForegroundColor White
Write-Host "2. 创建新的版本标签触发 Release" -ForegroundColor White
Write-Host "3. 观察 GitHub Actions 日志" -ForegroundColor White
Write-Host ""

Write-Host "📞 需要帮助?" -ForegroundColor Cyan
Write-Host "如果问题仍然存在，请:" -ForegroundColor Yellow
Write-Host "1. 检查 GitHub Actions 的详细日志" -ForegroundColor White
Write-Host "2. 确认仓库名称是小写 (cls3389/smart-forward)" -ForegroundColor White
Write-Host "3. 验证网络连接和 GitHub 服务状态" -ForegroundColor White
Write-Host ""

Write-Host "✅ 诊断完成！请按照上述步骤修复权限问题。" -ForegroundColor Green
