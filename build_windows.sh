#!/bin/bash
# Build script for RustDesk Windows portable client with built-in server configuration

set -e

echo "=== Building RustDesk Windows Portable Client ==="

# Server configuration
ID_SERVER="192.168.1.28:21116"
RELAY_SERVER="192.168.1.28:21117"
API_SERVER="http://192.168.1.28:21114"
RS_PUB_KEY="O18upa5wxCXadpN7hMCmAOnpdDnl88JpvWOt9hHQtg8="
PASSWORD="yarou1994"

echo "Server Configuration:"
echo "  ID Server: $ID_SERVER"
echo "  Relay Server: $RELAY_SERVER"
echo "  API Server: $API_SERVER"
echo "  Key: $RS_PUB_KEY"
echo "  Password: $PASSWORD"
echo ""

# Set environment variables for compilation
export RENDEZVOUS_SERVER="$ID_SERVER"
export RS_PUB_KEY="$RS_PUB_KEY"

# Check if Windows target is installed
if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "Installing Windows target..."
    rustup target add x86_64-pc-windows-gnu
fi

# Configure cargo for cross-compilation
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"

[target.aarch64-pc-windows-gnullvm]
linker = "aarch64-w64-mingw32-gcc"
ar = "aarch64-w64-mingw32-ar"
EOF

echo "Building RustDesk for Windows..."

# Build with flutter feature for Windows
cargo build \
    --release \
    --target x86_64-pc-windows-gnu \
    --features "flutter,hwcodec,flutter_text_style" \
    2>&1 | tee build.log

if [ $? -eq 0 ]; then
    echo ""
    echo "=== Build Successful ==="
    echo "Output: target/x86_64-pc-windows-gnu/release/rustdesk.exe"
    
    # Copy to output directory
    mkdir -p output
    cp target/x86_64-pc-windows-gnu/release/rustdesk.exe output/
    
    echo ""
    echo "Executable copied to: output/rustdesk.exe"
else
    echo ""
    echo "=== Build Failed ==="
    exit 1
fi
