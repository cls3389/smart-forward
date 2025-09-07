# GitHub Container Registry 诊断脚本
param(
    [string]$Repository = "cls3389/smart-forward",
    [string]$Tag = "v1.0.6"
)

Write-Host "GitHub Container Registry 诊断工具" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# 检查环境变量
Write-Host "`n环境检查:" -ForegroundColor Yellow
if ($env:GITHUB_TOKEN) {
    Write-Host "GITHUB_TOKEN 已设置" -ForegroundColor Green
} else {
    Write-Host "GITHUB_TOKEN 未设置" -ForegroundColor Red
}

# 检查 Docker
Write-Host "`nDocker 检查:" -ForegroundColor Yellow
try {
    $dockerVersion = docker --version
    Write-Host "Docker 已安装: $dockerVersion" -ForegroundColor Green
} catch {
    Write-Host "Docker 未安装" -ForegroundColor Red
}

# 测试 GHCR 连接
Write-Host "`nGHCR 连接测试:" -ForegroundColor Yellow
$packageUrl = "https://ghcr.io/v2/$Repository/manifests/$Tag"
Write-Host "测试 URL: $packageUrl" -ForegroundColor Gray

try {
    $response = Invoke-WebRequest -Uri $packageUrl -Method Head -ErrorAction Stop
    Write-Host "包访问成功 (状态码: $($response.StatusCode))" -ForegroundColor Green
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    Write-Host "包访问失败 (状态码: $statusCode)" -ForegroundColor Red
    
    if ($statusCode -eq 401) {
        Write-Host "认证失败，检查 GITHUB_TOKEN" -ForegroundColor Yellow
    } elseif ($statusCode -eq 403) {
        Write-Host "权限不足，检查包权限设置" -ForegroundColor Yellow
    } elseif ($statusCode -eq 404) {
        Write-Host "包不存在，需要先推送" -ForegroundColor Yellow
    } else {
        Write-Host "未知错误: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

Write-Host "`n解决建议:" -ForegroundColor Cyan
Write-Host "1. 检查仓库包权限设置" -ForegroundColor White
Write-Host "2. 确保 GITHUB_TOKEN 有 write:packages 权限" -ForegroundColor White
Write-Host "3. 尝试手动登录: docker login ghcr.io" -ForegroundColor White