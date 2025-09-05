@echo off
chcp 65001 > nul
title 智能转发器 - 前台运行

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

:: 显示启动信息
echo [信息] 启动模式：前台运行
echo [信息] 配置文件：config.yaml
echo [信息] 按 Ctrl+C 停止服务
echo.

:: 启动程序
target\release\smart-forward.exe --config config.yaml

:: 程序结束后的处理
echo.
echo [信息] 智能转发器已停止
pause
