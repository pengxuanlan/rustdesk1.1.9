#!/bin/bash
# RustDesk Windows 便携版 Docker 构建脚本
# 在宿主机执行，自动构建 Docker 镜像并编译

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=========================================="
echo "  RustDesk Windows Docker 构建"
echo "=========================================="
echo ""

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "错误: Docker 未安装"
    exit 1
fi

# 显示配置
echo "[构建配置]"
echo "  项目目录: $PROJECT_ROOT"
echo "  ID服务器:    192.168.1.28:21116"
echo "  中继服务器:  192.168.1.28:21117"
echo "  API服务器:   http://192.168.1.28:21114"
echo "  连接密码:    yarou1994"
echo "  公钥:        O18upa5wxCXadpN7hMCmAOnpdDnl88JpvWOt9hHQtg8="
echo ""

# 构建 Docker 镜像
echo "[步骤 1/3] 构建 Docker 镜像..."
docker build -t rustdesk-windows-builder \
    -f "$SCRIPT_DIR/Dockerfile" \
    "$PROJECT_ROOT"

# 运行编译容器
echo ""
echo "[步骤 2/3] 运行编译..."
docker run --rm \
    -v "$PROJECT_ROOT:/workspace" \
    -e RELAY_SERVER="192.168.1.28:21117" \
    -e API_SERVER="http://192.168.1.28:21114" \
    rustdesk-windows-builder

# 复制输出文件
echo ""
echo "[步骤 3/3] 复制输出文件..."
OUTPUT_DIR="$PROJECT_ROOT/output/windows"
mkdir -p "$OUTPUT_DIR"
cp "$PROJECT_ROOT/output/rustdesk.exe" "$OUTPUT_DIR/" 2>/dev/null || true

echo ""
echo "=========================================="
echo "  构建完成!"
echo "=========================================="
echo ""
echo "输出文件: $OUTPUT_DIR/rustdesk.exe"

if [ -f "$OUTPUT_DIR/rustdesk.exe" ]; then
    ls -lh "$OUTPUT_DIR/rustdesk.exe"
else
    echo "警告: 输出文件未找到，请检查编译日志"
fi
