Write-Host "启动智能转发器 (后台模式)..." -ForegroundColor Green

# 检查配置文件是否存在
if (-not (Test-Path "config.yaml")) {
    Write-Host "错误: 找不到配置文件 config.yaml" -ForegroundColor Red
    Write-Host "请确保配置文件与程序在同一目录" -ForegroundColor Yellow
    Read-Host "按回车键退出"
    exit 1
}

# 检查程序是否存在
if (-not (Test-Path "smart-forward.exe")) {
    Write-Host "错误: 找不到程序 smart-forward.exe" -ForegroundColor Red
    Write-Host "请确保程序与脚本在同一目录" -ForegroundColor Yellow
    Read-Host "按回车键退出"
    exit 1
}

# 检查是否已经在运行
if (Test-Path "smart-forward.pid") {
    Write-Host "警告: 发现PID文件，程序可能已在运行" -ForegroundColor Yellow
    Write-Host "PID: $(Get-Content smart-forward.pid)" -ForegroundColor Cyan
    Write-Host ""
    $choice = Read-Host "是否继续启动? (y/N)"
    if ($choice -ne "y" -and $choice -ne "Y") {
        Write-Host "取消启动" -ForegroundColor Yellow
        Read-Host "按回车键退出"
        exit 0
    }
}

# 启动程序到后台
Write-Host "正在启动后台服务..." -ForegroundColor Green
Start-Process -FilePath "smart-forward.exe" -ArgumentList "-d" -WindowStyle Hidden

# 等待一下让程序启动
Start-Sleep -Seconds 2

# 检查是否启动成功
if (Test-Path "smart-forward.pid") {
    Write-Host "程序已成功启动到后台" -ForegroundColor Green
    Write-Host "PID: $(Get-Content smart-forward.pid)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "程序正在后台运行，可以关闭此窗口" -ForegroundColor Green
} else {
    Write-Host "启动失败，请检查错误信息" -ForegroundColor Red
}

Read-Host "按回车键退出"
