@echo off
chcp 65001 > nul
title 智能转发器 - 后台运行

echo ================================
echo    智能网络转发器 (Smart Forward)
echo ================================
echo.

:: 检查配置文件是否存在
if not exist "config.yaml" (
    echo [错误] 配置文件 config.yaml 不存在！
    echo 请确保 config.yaml 文件在当前目录下。
    pause
    exit /b 1
)

:: 检查可执行文件是否存在
if not exist "target\release\smart-forward.exe" (
    echo [信息] Release版本不存在，正在编译...
    cargo build --release
    if errorlevel 1 (
        echo [错误] 编译失败！
        pause
        exit /b 1
    )
)

:: 检查是否已经运行
if exist "smart-forward.pid" (
    set /p PID=<smart-forward.pid
    tasklist /FI "PID eq %PID%" 2>nul | find /I "smart-forward.exe" >nul
    if not errorlevel 1 (
        echo [警告] 智能转发器可能已在运行 (PID: %PID%)
        echo 如需重启，请先运行 stop.bat 停止服务
        pause
        exit /b 0
    )
)

:: 显示启动信息
echo [信息] 启动模式：后台运行
echo [信息] 配置文件：config.yaml
echo [信息] PID文件：smart-forward.pid
echo.

:: 后台启动程序
start /B "" target\release\smart-forward.exe --daemon --config config.yaml --pid-file smart-forward.pid

:: 等待一秒确保启动
timeout /t 2 /nobreak >nul

:: 检查是否成功启动
if exist "smart-forward.pid" (
    set /p PID=<smart-forward.pid
    echo [成功] 智能转发器已启动 (PID: %PID%)
    echo [信息] 使用 stop.bat 停止服务
    echo [信息] 查看日志：RUST_LOG=info 环境变量控制日志级别
) else (
    echo [错误] 启动失败，未找到PID文件
)

echo.
pause
