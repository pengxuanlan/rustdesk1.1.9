#!/bin/bash
# RustDesk Windows 便携版构建脚本
# 在 Docker 容器内执行

set -e

echo "=========================================="
echo "  RustDesk Windows 便携版构建脚本"
echo "=========================================="
echo ""

# 显示配置信息
echo "[配置信息]"
echo "  ID服务器:    $RENDEZVOUS_SERVER"
echo "  中继服务器:  $RELAY_SERVER (未设置则使用默认)"
echo "  API服务器:   $API_SERVER (未设置则使用默认)"
echo "  公钥:        $RS_PUB_KEY"
echo "  连接密码:    yarou1994 (内置)"
echo ""

# 进入源码目录
cd /workspace

# 开始编译
echo "[开始编译]"
echo "目标: x86_64-pc-windows-gnu"
echo "特性: flutter"
echo ""

cargo build --release --target x86_64-pc-windows-gnu --features "flutter"

# 检查编译结果
if [ -f "target/x86_64-pc-windows-gnu/release/rustdesk.exe" ]; then
    echo ""
    echo "=========================================="
    echo "  编译成功!"
    echo "=========================================="
    echo ""
    echo "输出文件: target/x86_64-pc-windows-gnu/release/rustdesk.exe"
    
    # 复制到输出目录
    mkdir -p /workspace/output
    cp target/x86_64-pc-windows-gnu/release/rustdesk.exe /workspace/output/
    
    # 显示文件大小
    ls -lh /workspace/output/rustdesk.exe
    
    echo ""
    echo "文件已复制到: /workspace/output/rustdesk.exe"
else
    echo ""
    echo "=========================================="
    echo "  编译失败!"
    echo "=========================================="
    exit 1
fi
