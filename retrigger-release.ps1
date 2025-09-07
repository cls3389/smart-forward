# 重新触发发布脚本
param(
    [string]$Version = "v1.0.1"
)

Write-Host "重新触发发布: $Version" -ForegroundColor Cyan

# 1. 检查当前状态
Write-Host "`n检查当前状态..." -ForegroundColor Yellow
git status

# 2. 提交任何未提交的更改
Write-Host "`n提交更改..." -ForegroundColor Yellow
git add .
git commit -m "Update for release $Version" || Write-Host "没有需要提交的更改" -ForegroundColor Green

# 3. 推送代码
Write-Host "`n推送代码..." -ForegroundColor Yellow
git push origin main

# 4. 创建新标签
Write-Host "`n创建标签 $Version..." -ForegroundColor Yellow
git tag -a $Version -m "Release $Version - 修复 Windows 构建问题"
git push origin $Version

Write-Host "`n✅ 发布已触发！" -ForegroundColor Green
Write-Host "请访问 https://github.com/cls3389/smart-forward/actions 查看构建状态" -ForegroundColor Cyan
Write-Host "构建完成后，文件将在 https://github.com/cls3389/smart-forward/releases 中可见" -ForegroundColor Cyan
