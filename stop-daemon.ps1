Write-Host "停止智能转发器 (后台模式)..." -ForegroundColor Green

# 检查PID文件是否存在
if (-not (Test-Path "smart-forward.pid")) {
    Write-Host "错误: 找不到PID文件，程序可能未在运行" -ForegroundColor Red
    Read-Host "按回车键退出"
    exit 1
}

# 读取PID
$pid = Get-Content "smart-forward.pid"
Write-Host "正在停止进程 PID: $pid" -ForegroundColor Cyan

# 尝试优雅停止
try {
    Stop-Process -Id $pid -Force
    Write-Host "程序已成功停止" -ForegroundColor Green
    Remove-Item "smart-forward.pid"
} catch {
    Write-Host "停止失败，进程可能已经不存在" -ForegroundColor Yellow
    Remove-Item "smart-forward.pid" -ErrorAction SilentlyContinue
}

Read-Host "按回车键退出"
