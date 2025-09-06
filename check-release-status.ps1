# ================================
# 检查发布状态脚本
# ================================

Write-Host "🔍 检查 Smart Forward v1.0.0 发布状态..." -ForegroundColor Cyan

# 检查本地状态
Write-Host "`n📋 本地状态:" -ForegroundColor Yellow
Write-Host "当前分支: $(git branch --show-current)" -ForegroundColor Green
Write-Host "最新提交: $(git log -1 --oneline)" -ForegroundColor Green
Write-Host "版本标签: $(git tag --list | Select-String 'v1.0.0')" -ForegroundColor Green

# 检查 GitHub Actions 状态
Write-Host "`n🚀 GitHub Actions 状态:" -ForegroundColor Yellow
Write-Host "请访问以下链接查看构建进度:" -ForegroundColor Cyan
Write-Host "https://github.com/cls3389/smart-forward/actions" -ForegroundColor Blue

# 检查发布页面
Write-Host "`n📦 发布页面:" -ForegroundColor Yellow
Write-Host "请访问以下链接查看发布状态:" -ForegroundColor Cyan
Write-Host "https://github.com/cls3389/smart-forward/releases" -ForegroundColor Blue

# 检查 Docker 镜像
Write-Host "`n🐳 Docker 镜像:" -ForegroundColor Yellow
Write-Host "多架构镜像将发布到:" -ForegroundColor Cyan
Write-Host "ghcr.io/cls3389/smart-forward:1.0.0" -ForegroundColor Blue
Write-Host "ghcr.io/cls3389/smart-forward:latest" -ForegroundColor Blue

Write-Host "`n✅ 发布流程已启动，请等待 GitHub Actions 完成构建！" -ForegroundColor Green
