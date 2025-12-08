#!/bin/bash

# 编译 crypto-exchange 系统组件为 Ubuntu 24 (x86_64) 可执行文件
# 使用 musl 静态链接，无需额外依赖

set -e  # 遇到错误立即退出

# 定义要编译的服务列表 (package names)
SERVICES=(
    "exchange"
    "business"
    "agent"
    "manage"
    "job"
    "input"
)

echo "======================================"
echo "编译 Crypto Exchange for Ubuntu 24"
echo "======================================"

# 检查是否安装了 musl-cross
if ! command -v x86_64-linux-musl-gcc &> /dev/null; then
    echo "错误: 未找到 x86_64-linux-musl-gcc"
    echo "请先安装: brew install filosottile/musl-cross/musl-cross"
    exit 1
fi

# 检查是否添加了 musl 目标
if ! rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "添加 x86_64-unknown-linux-musl 目标..."
    rustup target add x86_64-unknown-linux-musl
fi

# 设置环境变量
export CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc

# 确保在项目根目录
cd "$(dirname "$0")"

echo ""
echo "目标平台: x86_64-unknown-linux-musl"
echo "准备编译以下服务:"
for SERVICE in "${SERVICES[@]}"; do
    echo " - $SERVICE"
done
echo ""
echo "开始编译..."
echo ""

# 循环编译每个服务
SUCCESS_COUNT=0
TOTAL_COUNT=${#SERVICES[@]}

for SERVICE in "${SERVICES[@]}"; do
    echo ">>> 正在编译 [$SERVICE] ..."
    
    if [[ "$SERVICE" == "block-eth" ]]; then
        if cargo build --release --target x86_64-unknown-linux-musl --manifest-path block/eth/Cargo.toml --target-dir target; then
             echo "✅ [$SERVICE] 编译成功"
            ((SUCCESS_COUNT++))
        else
            echo "❌ [$SERVICE] 编译失败"
            exit 1
        fi
    elif [[ "$SERVICE" == "block-sol" ]]; then
        if cargo build --release --target x86_64-unknown-linux-musl --manifest-path block/sol/Cargo.toml --target-dir target; then
             echo "✅ [$SERVICE] 编译成功"
            ((SUCCESS_COUNT++))
        else
            echo "❌ [$SERVICE] 编译失败"
            exit 1
        fi
    else
        if cargo build --release --target x86_64-unknown-linux-musl -p "$SERVICE"; then
            echo "✅ [$SERVICE] 编译成功"
            ((SUCCESS_COUNT++))
        else
            echo "❌ [$SERVICE] 编译失败"
            exit 1
        fi
    fi
    echo ""
done

# 汇总结果
echo "======================================"
echo "编译完成！ ($SUCCESS_COUNT/$TOTAL_COUNT)"
echo "======================================"
echo ""
echo "生成的二进制文件:"

TARGET_DIR="$(pwd)/target/x86_64-unknown-linux-musl/release"

for SERVICE in "${SERVICES[@]}"; do
    BINARY_PATH="$TARGET_DIR/$SERVICE"
    if [ -f "$BINARY_PATH" ]; then
        echo " - $SERVICE: $BINARY_PATH"
    fi
done

echo ""
echo "======================================"
echo "部署说明:"
echo "======================================"
echo "1. 将需要的文件复制到 Ubuntu 服务器:"
echo "   scp target/x86_64-unknown-linux-musl/release/<binary_name> user@server:/path/to/deploy/"
echo ""
echo "2. 在服务器上设置权限并运行:"
echo "   chmod +x <binary_name>"
echo "   ./<binary_name>"
echo ""
echo "注意: 这些是静态链接的二进制文件，可以在任何 x86_64 Linux 系统上运行"
echo "======================================"
