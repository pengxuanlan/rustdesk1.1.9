# RustDesk Windows 客户端 Docker 编译指南

本指南说明如何使用 Docker 在 Linux 环境中交叉编译 Windows 客户端。

## 环境要求

- Docker 已安装并运行
- 至少 10GB 可用磁盘空间
- 至少 4GB 内存

## 快速开始

### 1. 构建 Docker 镜像

```bash
cd /workspace
docker build -t rustdesk-win-builder -f Dockerfile.windows .
```

这将创建一个包含所有必要工具的 Docker 镜像，包括：
- Rust 编译器
- MinGW-W64 交叉编译工具链
- Windows 编译目标

### 2. 编译 Windows 客户端

#### 使用默认服务器配置编译

```bash
docker run --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  rustdesk-win-builder \
  cargo build --release --features inline --target x86_64-pc-windows-gnu
```

#### 使用自定义服务器配置编译

设置环境变量来指定内置服务器：

```bash
# 设置服务器IP和密钥
export RENDEZVOUS_SERVER="21.8.249.108:21116"
export RS_PUB_KEY="YOUR_SERVER_KEY_HERE"

# 编译
docker run --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  -e RENDEZVOUS_SERVER="$RENDEZVOUS_SERVER" \
  -e RS_PUB_KEY="$RS_PUB_KEY" \
  rustdesk-win-builder \
  bash -c 'echo "pub const RENDEZVOUS_SERVERS: &[&str] = &[\"$RENDEZVOUS_SERVER\"];" > /tmp/config.txt && \
           cat libs/hbb_common/src/config.rs | sed "s/pub const RENDEZVOUS_SERVERS.*/pub const RENDEZVOUS_SERVERS: \&[\&str] = \&[\"$RENDEZVOUS_SERVER\",];/" > /tmp/config_new.rs && \
           mv /tmp/config_new.rs libs/hbb_common/src/config.rs && \
           cargo build --release --features inline --target x86_64-pc-windows-gnu'
```

#### 使用编译脚本

更简单的方式是使用提供的脚本：

```bash
# 构建镜像
./build-docker.sh --build-image

# 编译（使用默认配置）
./build-docker.sh --compile

# 编译（使用自定义配置）
./build-docker.sh --compile --server-ip "21.8.249.108:21116" --server-key "YOUR_KEY"
```

## 编译输出

编译成功后，可执行文件位于：

```
target/x86_64-pc-windows-gnu/release/rustdesk.exe
```

## 常见问题

### 1. 编译失败：找不到 libsodium

确保在 Docker 镜像中安装了 mingw-w64 工具链：

```dockerfile
RUN apt-get install -y mingw-w64
```

### 2. 编译失败：找不到库文件

确保设置了正确的链接器配置：

```bash
cat >> ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
rustflags = ["-C", "target-feature=+crt-static"]
EOF
```

### 3. 编译速度慢

- 使用 `--release` 标志进行优化编译
- 确保 Docker 有足够的内存分配（至少 4GB）
- 考虑使用本地 Rust 安装而不是 Docker

## 手动编译步骤

如果您想要手动执行每一步：

1. **启动容器并进入 shell**：
```bash
docker run -it --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  rustdesk-win-builder \
  bash
```

2. **在容器内执行编译**：
```bash
# 设置环境变量
export RENDEZVOUS_SERVER="21.8.249.108:21116"
export RS_PUB_KEY="YOUR_SERVER_KEY"

# 修改配置（如果需要）
sed -i 's/pub const RENDEZVOUS_SERVERS.*/pub const RENDEZVOUS_SERVERS: \&[\&str] = \&["21.8.249.108:21116",];/' \
  libs/hbb_common/src/config.rs

# 编译
cargo build --release --features inline --target x86_64-pc-windows-gnu
```

## 清理

清理 Docker 镜像：

```bash
docker rmi rustdesk-win-builder
```

清理编译产物：

```bash
cargo clean
rm -rf target/x86_64-pc-windows-gnu
```
