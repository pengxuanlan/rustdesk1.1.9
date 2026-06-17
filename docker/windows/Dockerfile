FROM debian:bullseye-slim

ENV DEBIAN_FRONTEND=noninteractive

# 安装基本工具和 mingw 交叉编译器
RUN apt-get update && apt-get install -y \
    g++-mingw-w64-x86-64 gcc-mingw-w64-x86-64 \
    git curl wget nasm yasm cmake unzip zip python3 \
    pkg-config autoconf automake libtool \
    clang llvm-dev libclang-dev libclang-cpp-dev \
    && rm -rf /var/lib/apt/lists/*

# 安装 Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# 添加 Windows 目标
RUN rustup target add x86_64-pc-windows-gnu

# 配置 cargo 交叉编译
RUN mkdir -p /root/.cargo && cat > /root/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
EOF

# 克隆并安装 vcpkg（用于 opus, libvpx, libyuv）
WORKDIR /
RUN git clone https://github.com/microsoft/vcpkg.git && cd vcpkg && \
    git checkout 134505003bb46e20fbace51ccfb69243fbbc5f82 && \
    ./bootstrap-vcpkg.sh -disableMetrics && \
    ./vcpkg --disable-metrics install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static

# 创建头文件大小写兼容链接
RUN cd /usr/x86_64-w64-mingw32/include && \
    ln -sf windows.h Windows.h && \
    ln -sf psapi.h Psapi.h && \
    ln -sf tlhelp32.h Tlhelp32.h

# 工作目录
WORKDIR /workspace

# 设置环境变量
ENV VCPKG_ROOT=/vcpkg
ENV RENDEZVOUS_SERVER=192.168.1.28:21116
ENV RS_PUB_KEY=O18upa5wxCXadpN7hMCmAOnpdDnl88JpvWOt9hHQtg8=
ENV BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/x86_64-w64-mingw32 --target=x86_64-pc-windows-gnu"

CMD ["bash", "/build.sh"]
