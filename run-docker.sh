#!/bin/bash
# Docker 运行脚本

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

CONTAINER_NAME="smart-forward-container"
IMAGE_NAME="smart-forward:latest"

echo -e "${GREEN}🚀 启动智能网络转发器容器${NC}"

# 检查容器是否已存在
if docker ps -a --format 'table {{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}⚠️  容器已存在，正在重启...${NC}"
    docker stop "${CONTAINER_NAME}" 2>/dev/null || true
    docker rm "${CONTAINER_NAME}" 2>/dev/null || true
fi

# 启动容器
echo -e "${GREEN}🔄 启动新容器...${NC}"
docker run -d \
    --name "${CONTAINER_NAME}" \
    --restart unless-stopped \
    -p 443:443 \
    -p 99:99 \
    -p 6690:6690 \
    -p 999:999 \
    -v "$(pwd)/config.yaml:/app/config.yaml:ro" \
    "${IMAGE_NAME}"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ 容器启动成功!${NC}"
    echo ""
    echo "📋 容器状态:"
    docker ps --filter "name=${CONTAINER_NAME}"
    echo ""
    echo -e "${GREEN}🔍 实时日志 (Ctrl+C 退出):${NC}"
    docker logs -f "${CONTAINER_NAME}"
else
    echo -e "${RED}❌ 容器启动失败!${NC}"
    exit 1
fi
