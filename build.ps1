# 简单的Docker构建脚本
Write-Host "🐳 构建 Docker 镜像..." -ForegroundColor Green

# 检查Docker
try {
    docker --version | Out-Null
    Write-Host "✅ Docker 可用" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker 不可用" -ForegroundColor Red
    exit 1
}

# 构建镜像
Write-Host "🔨 开始构建..." -ForegroundColor Yellow
docker build -t smart-forward:latest .

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ 构建成功!" -ForegroundColor Green
    
    # 显示镜像信息
    Write-Host "`n📊 镜像信息:" -ForegroundColor Cyan
    docker images smart-forward:latest
    
    Write-Host "`n💡 使用方法:" -ForegroundColor Cyan
    Write-Host "  docker run -d --name smart-forward -p 443:443 smart-forward:latest" -ForegroundColor White
} else {
    Write-Host "❌ 构建失败!" -ForegroundColor Red
    exit 1
}
