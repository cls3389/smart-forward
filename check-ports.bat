@echo off
echo ==============================
echo 检查智能转发器端口占用情况
echo ==============================

echo.
echo 检查配置中的端口：
findstr "listen_port:" config.yaml

echo.
echo 检查端口占用情况：
echo 57111端口:
netstat -ano | findstr ":57111" || echo "  端口空闲"

echo.
echo 7909端口:
netstat -ano | findstr ":7909" || echo "  端口空闲"

echo.
echo 7697端口:
netstat -ano | findstr ":7697" || echo "  端口空闲"

echo.
echo 7853端口:
netstat -ano | findstr ":7853" || echo "  端口空闲"

echo.
echo 7857端口:
netstat -ano | findstr ":7857" || echo "  端口空闲"

echo.
echo 检查是否有smart-forward进程运行：
tasklist | findstr "smart-forward" || echo "  没有smart-forward进程运行"

echo.
echo ==============================
echo 检查完成
echo ==============================
pause
