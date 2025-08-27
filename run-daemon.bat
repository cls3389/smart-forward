@echo off
echo 启动智能转发器 (后台模式)...

REM 检查配置文件是否存在
if not exist "config.yaml" (
    echo 错误: 找不到配置文件 config.yaml
    echo 请确保配置文件与程序在同一目录
    pause
    exit /b 1
)

REM 检查程序是否存在
if not exist "smart-forward.exe" (
    echo 错误: 找不到程序 smart-forward.exe
    echo 请确保程序与脚本在同一目录
    pause
    exit /b 1
)

REM 检查是否已经在运行
if exist "smart-forward.pid" (
    echo 警告: 发现PID文件，程序可能已在运行
    echo PID: 
    type smart-forward.pid
    echo.
    set /p choice="是否继续启动? (y/N): "
    if /i not "%choice%"=="y" (
        echo 取消启动
        pause
        exit /b 0
    )
)

REM 启动程序到后台
echo 正在启动后台服务...
start /b smart-forward.exe -d

REM 等待一下让程序启动
timeout /t 2 /nobreak > nul

REM 检查是否启动成功
if exist "smart-forward.pid" (
    echo 程序已成功启动到后台
    echo PID: 
    type smart-forward.pid
    echo.
    echo 程序正在后台运行，可以关闭此窗口
) else (
    echo 启动失败，请检查错误信息
)

pause
