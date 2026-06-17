# RustDesk Windows 便携版 Docker 构建

## 快速开始

```bash
# 一键构建（推荐）
chmod +x docker/build-windows.sh && ./docker/build-windows.sh

# 或分步执行：
# 1. 构建镜像
docker build -t rustdesk-windows-builder -f docker/windows/Dockerfile .

# 2. 运行编译
docker run --rm -v $(pwd):/workspace rustdesk-windows-builder

# 3. 获取输出文件
ls -lh output/rustdesk.exe
```

## 内置配置

| 配置项 | 值 |
|--------|-----|
| ID服务器 | 192.168.1.28:21116 |
| 中继服务器 | 192.168.1.28:21117 |
| API服务器 | http://192.168.1.28:21114 |
| 连接密码 | yarou1994 |
| 公钥 | O18upa5wxCXadpN7hMCmAOnpdDnl88JpvWOt9hHQtg8= |

## 输出文件

编译完成后，Windows 可执行文件位于:
```
output/rustdesk.exe
```

## 自定义配置

如需修改服务器信息，编辑 `src/core_main.rs` 中的 `set_builtin_config()` 函数：

```rust
fn set_builtin_config() {
    set_option("custom-rendezvous-server".into(), "你的ID服务器".into());
    set_option("relay-server".into(), "你的中继服务器".into());
    set_option("api-server".into(), "你的API服务器".into());
    Config::set_permanent_password("你的密码".to_owned()).ok();
}
```

## 环境变量

可通过 `-e` 参数传递给 Docker 容器：

```bash
docker run --rm \
    -v $(pwd):/workspace \
    -e RENDEZVOUS_SERVER="你的ID服务器" \
    -e RS_PUB_KEY="你的公钥" \
    rustdesk-windows-builder
```
