@echo off
echo 智能转发器启动脚本
echo ====================
echo 当前目录: %CD%
echo.

REM 检查配置文件是否存在
if not exist "config.yaml" (
    echo 错误: 找不到配置文件 config.yaml
    echo 请确保配置文件存在于当前目录
    pause
    exit /b 1
)

REM 检查程序文件是否存在
if not exist "target\release\smart-forward.exe" (
    echo 错误: 找不到程序文件 target\release\smart-forward.exe
    echo 请先运行 cargo build --release 编译程序
    pause
    exit /b 1
)

echo 启动程序...
echo.
target\release\smart-forward.exe

echo.
echo 程序已退出
pause
