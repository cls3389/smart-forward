#!/bin/bash
# Docker 构建脚本 - 支持 WSL2 代理环境

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 智能网络转发器 Docker 构建脚本${NC}"
echo "======================================"

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Docker 未运行，请启动 Docker${NC}"
    exit 1
fi

# 设置代理（如果在 WSL2 环境中）
if [[ -n "$WSL_DISTRO_NAME" ]]; then
    echo -e "${YELLOW}📡 检测到 WSL2 环境，配置代理...${NC}"
    
    # WSL2 镜像网络模式下的代理配置
    PROXY_HOST="127.0.0.1"
    PROXY_PORT="7897"
    
    export HTTP_PROXY="http://${PROXY_HOST}:${PROXY_PORT}"
    export HTTPS_PROXY="http://${PROXY_HOST}:${PROXY_PORT}"
    export NO_PROXY="localhost,127.0.0.1,::1"
    
    echo "代理设置: $HTTP_PROXY"
fi

# 镜像信息
IMAGE_NAME="smart-forward"
IMAGE_TAG="latest"
FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"

echo -e "${GREEN}📦 开始构建镜像: ${FULL_IMAGE_NAME}${NC}"

# 构建 Docker 镜像
docker build \
    --build-arg HTTP_PROXY="${HTTP_PROXY:-}" \
    --build-arg HTTPS_PROXY="${HTTPS_PROXY:-}" \
    --build-arg NO_PROXY="${NO_PROXY:-}" \
    -t "${FULL_IMAGE_NAME}" \
    .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ 镜像构建成功!${NC}"
    echo ""
    echo "📋 镜像信息:"
    docker images | grep "${IMAGE_NAME}"
    echo ""
    echo -e "${GREEN}🚀 运行命令:${NC}"
    echo "docker run -d --name smart-forward-container \\"
    echo "  -p 443:443 \\"
    echo "  -p 99:99 \\"
    echo "  -p 6690:6690 \\"
    echo "  -p 999:999 \\"
    echo "  -v \$(pwd)/config.yaml:/app/config.yaml \\"
    echo "  ${FULL_IMAGE_NAME}"
    echo ""
    echo -e "${GREEN}🔍 查看日志:${NC}"
    echo "docker logs -f smart-forward-container"
    echo ""
    echo -e "${GREEN}🛑 停止容器:${NC}"
    echo "docker stop smart-forward-container && docker rm smart-forward-container"
else
    echo -e "${RED}❌ 镜像构建失败!${NC}"
    exit 1
fi
