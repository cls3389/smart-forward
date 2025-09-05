@echo off
chcp 65001 > nul
title 智能转发器 - 停止服务

echo ================================
echo    智能网络转发器 (Smart Forward)
echo ================================
echo.

:: 检查PID文件是否存在
if not exist "smart-forward.pid" (
    echo [信息] 未找到PID文件，服务可能未运行
    goto :manual_stop
)

:: 读取PID
set /p PID=<smart-forward.pid
echo [信息] 发现PID文件：%PID%

:: 检查进程是否存在
tasklist /FI "PID eq %PID%" 2>nul | find /I "smart-forward.exe" >nul
if errorlevel 1 (
    echo [信息] 进程 %PID% 不存在，可能已停止
    del smart-forward.pid 2>nul
    goto :end
)

:: 终止进程
echo [信息] 正在停止智能转发器 (PID: %PID%)...
taskkill /PID %PID% /F >nul 2>&1
if not errorlevel 1 (
    echo [成功] 智能转发器已停止
    del smart-forward.pid 2>nul
) else (
    echo [错误] 无法停止进程 %PID%
)
goto :end

:manual_stop
echo [信息] 尝试手动停止所有智能转发器进程...
taskkill /IM smart-forward.exe /F >nul 2>&1
if not errorlevel 1 (
    echo [成功] 已停止所有智能转发器进程
) else (
    echo [信息] 未找到运行中的智能转发器进程
)

:end
echo.
pause
