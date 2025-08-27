@echo off
echo 停止智能转发器 (后台模式)...

REM 检查PID文件是否存在
if not exist "smart-forward.pid" (
    echo 错误: 找不到PID文件，程序可能未在运行
    pause
    exit /b 1
)

REM 读取PID
set /p pid=<smart-forward.pid
echo 正在停止进程 PID: %pid%

REM 尝试优雅停止
taskkill /PID %pid% /F
if %errorlevel% equ 0 (
    echo 程序已成功停止
    del smart-forward.pid
) else (
    echo 停止失败，进程可能已经不存在
    del smart-forward.pid
)

pause
