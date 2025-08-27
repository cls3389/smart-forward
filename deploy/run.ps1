# 智能转发器启动脚本
Write-Host "智能转发器启动脚本" -ForegroundColor Green
Write-Host "====================" -ForegroundColor Green
Write-Host "当前目录: $(Get-Location)" -ForegroundColor Yellow
Write-Host ""

# 检查配置文件是否存在
if (-not (Test-Path "config.yaml")) {
    Write-Host "错误: 找不到配置文件 config.yaml" -ForegroundColor Red
    Write-Host "请确保配置文件存在于当前目录" -ForegroundColor Red
    Read-Host "按任意键退出"
    exit 1
}

# 检查程序文件是否存在
if (-not (Test-Path "smart-forward.exe")) {
    Write-Host "错误: 找不到程序文件 smart-forward.exe" -ForegroundColor Red
    Write-Host "请确保程序文件存在于当前目录" -ForegroundColor Red
    Read-Host "按任意键退出"
    exit 1
}

Write-Host "启动程序..." -ForegroundColor Green
Write-Host ""

# 启动程序
try {
    & ".\smart-forward.exe"
} catch {
    Write-Host "程序启动失败: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "程序已退出" -ForegroundColor Yellow
Read-Host "按任意键退出"
