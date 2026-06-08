#!/bin/bash
# RustDesk Windows Cross-Compilation Script
# 使用Docker进行Windows交叉编译

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

IMAGE_NAME="rustdesk-win-builder"
CONTAINER_NAME="rustdesk-win-build"
DOCKERFILE="Dockerfile.windows"

echo -e "${GREEN}=== RustDesk Windows Cross-Compilation ===${NC}"

# 检查Docker是否安装
if ! command -v docker &> /dev/null; then
    echo -e "${RED}错误: Docker未安装${NC}"
    echo "请先安装Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

# 检查Dockerfile是否存在
if [ ! -f "$DOCKERFILE" ]; then
    echo -e "${RED}错误: $DOCKERFILE 不存在${NC}"
    exit 1
fi

# 解析命令行参数
BUILD_IMAGE=false
RUN_BUILD=false
SERVER_IP=""
SERVER_KEY=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-image)
            BUILD_IMAGE=true
            shift
            ;;
        --compile)
            RUN_BUILD=true
            shift
            ;;
        --server-ip)
            SERVER_IP="$2"
            shift 2
            ;;
        --server-key)
            SERVER_KEY="$2"
            shift 2
            ;;
        --help)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --build-image    构建Docker镜像"
            echo "  --compile       运行编译"
            echo "  --server-ip     设置服务器IP (例如: 21.8.249.108:21116)"
            echo "  --server-key    设置服务器密钥"
            echo "  --help          显示此帮助信息"
            echo ""
            echo "示例:"
            echo "  $0 --build-image                    # 构建Docker镜像"
            echo "  $0 --compile --server-ip 21.8.249.108:21116 --server-key YOUR_KEY"
            exit 0
            ;;
        *)
            echo -e "${RED}未知选项: $1${NC}"
            exit 1
            ;;
    esac
done

# 构建Docker镜像
if [ "$BUILD_IMAGE" = true ]; then
    echo -e "${GREEN}构建Docker镜像...${NC}"
    docker build -t "$IMAGE_NAME" -f "$DOCKERFILE" .
    echo -e "${GREEN}镜像构建完成!${NC}"
fi

# 运行编译
if [ "$RUN_BUILD" = true ]; then
    echo -e "${GREEN}开始Windows交叉编译...${NC}"

    # 检查服务器配置
    if [ -z "$SERVER_IP" ] || [ -z "$SERVER_KEY" ]; then
        echo -e "${YELLOW}警告: 未提供服务器配置，将使用默认配置${NC}"
        SERVER_IP="21.8.249.108:21116"
        SERVER_KEY="O18upa5wxCXadpN7hMCmAOnpdDnl88JpvWOt9hHQtg8="
    fi

    echo "服务器IP: $SERVER_IP"
    echo "服务器密钥: $SERVER_KEY"

    # 修改源代码中的服务器配置
    echo -e "${GREEN}修改服务器配置...${NC}"

    # 备份并修改配置文件
    sed -i "s/pub const RENDEZVOUS_SERVERS.*/pub const RENDEZVOUS_SERVERS: \&[\&str] = \&[\"$SERVER_IP\",];/" \
        libs/hbb_common/src/config.rs 2>/dev/null || true

    sed -i "s/pub const RS_PUB_KEY.*/pub const RS_PUB_KEY: \&str = \"$SERVER_KEY\";/" \
        libs/hbb_common/src/config.rs 2>/dev/null || true

    # 创建并运行编译容器
    docker run --rm \
        --name "$CONTAINER_NAME" \
        -v "$(pwd)":/workspace \
        -w /workspace \
        "$IMAGE_NAME" \
        bash -c "export RENDEZVOUS_SERVER='$SERVER_IP' && \
                 export RS_PUB_KEY='$SERVER_KEY' && \
                 cargo build --release --features inline --target x86_64-pc-windows-gnu"

    echo -e "${GREEN}编译完成!${NC}"
    echo "输出文件: target/x86_64-pc-windows-gnu/release/rustdesk.exe"
fi

# 如果没有执行任何操作，显示帮助
if [ "$BUILD_IMAGE" = false ] && [ "$RUN_BUILD" = false ]; then
    echo ""
    echo -e "${YELLOW}请指定要执行的操作${NC}"
    echo "使用 --help 查看帮助信息"
fi
