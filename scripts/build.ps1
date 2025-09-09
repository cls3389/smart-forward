# Alpine 3.18 优化版 Docker 构建脚本 (目标: 8MB)
Write-Host "🐳 构建 Alpine 优化 Docker 镜像..." -ForegroundColor Green

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
    Write-Host "  docker run -d --name smart-forward --network host smart-forward:latest" -ForegroundColor White
    Write-Host "`n🎯 优化特性:" -ForegroundColor Cyan
    Write-Host "  - Alpine 3.18 基础镜像" -ForegroundColor White
    Write-Host "  - 极致编译优化 (opt-level=z)" -ForegroundColor White
    Write-Host "  - 预期大小: ~8MB" -ForegroundColor White
} else {
    Write-Host "❌ 构建失败!" -ForegroundColor Red
    exit 1
}
