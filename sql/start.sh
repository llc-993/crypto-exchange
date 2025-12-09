#!/bin/bash

# input 服务管理脚本
# 用于在 Ubuntu 上启动、停止、重启服务

# 配置
APP_NAME="exchange"
APP_DIR="/home/exchange/exchange"  # 修改为实际部署路径
APP_BIN="${APP_DIR}/${APP_NAME}"
PID_FILE="${APP_DIR}/${APP_NAME}.pid"
LOG_FILE="${APP_DIR}/out.log"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查是否以 root 运行（可选）
# if [ "$EUID" -ne 0 ]; then
#   echo -e "${RED}请使用 root 或 sudo 运行此脚本${NC}"
#   exit 1
# fi

# 获取进程 PID
get_pid() {
    if [ -f "$PID_FILE" ]; then
        cat "$PID_FILE"
    fi
}

# 检查进程是否运行
is_running() {
    local pid=$(get_pid)
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# 启动服务
start() {
    if is_running; then
        local pid=$(get_pid)
        echo -e "${YELLOW}服务已在运行中 (PID: $pid)${NC}"
        return 0
    fi

    echo -e "${GREEN}正在启动 $APP_NAME...${NC}"

    # 检查二进制文件是否存在
    if [ ! -f "$APP_BIN" ]; then
        echo -e "${RED}错误: 找不到可执行文件 $APP_BIN${NC}"
        exit 1
    fi

    # 确保有执行权限
    chmod +x "$APP_BIN"

    # 启动服务
    cd "$APP_DIR"
    nohup "$APP_BIN" --production > "$LOG_FILE" 2>&1 &
    local pid=$!

    # 保存 PID
    echo $pid > "$PID_FILE"

    # 等待一秒检查是否启动成功
    sleep 1

    if is_running; then
        echo -e "${GREEN}✓ $APP_NAME 启动成功 (PID: $pid)${NC}"
        echo -e "日志文件: $LOG_FILE"
    else
        echo -e "${RED}✗ $APP_NAME 启动失败${NC}"
        echo -e "请查看日志: tail -f $LOG_FILE"
        rm -f "$PID_FILE"
        exit 1
    fi
}

# 停止服务
stop() {
    echo -e "${YELLOW}正在停止 $APP_NAME...${NC}"

    # 查找占用 8080 端口的进程
    local pid=$(lsof -ti:8086 2>/dev/null)

    # 如果 lsof 不可用，尝试使用 netstat
    if [ -z "$pid" ]; then
        pid=$(netstat -tlnp 2>/dev/null | grep ':8086 ' | awk '{print $7}' | cut -d'/' -f1)
    fi

    # 如果还是找不到，尝试从 PID 文件读取
    if [ -z "$pid" ] && [ -f "$PID_FILE" ]; then
        pid=$(cat "$PID_FILE")
    fi

    if [ -z "$pid" ]; then
        echo -e "${YELLOW}未找到运行中的服务进程${NC}"
        rm -f "$PID_FILE"
        return 0
    fi

    echo -e "${YELLOW}找到进程 PID: $pid，正在终止...${NC}"

    # 发送 TERM 信号
    kill -TERM "$pid" 2>/dev/null

    # 等待进程结束（最多等待 5 秒）
    local count=0
    while kill -0 "$pid" 2>/dev/null && [ $count -lt 5 ]; do
        sleep 1
        count=$((count + 1))
    done

    # 如果还在运行，强制杀死
    if kill -0 "$pid" 2>/dev/null; then
        echo -e "${RED}进程未响应，强制终止...${NC}"
        kill -9 "$pid" 2>/dev/null
        sleep 1
    fi

    # 清理 PID 文件
    rm -f "$PID_FILE"

    # 验证是否成功停止
    if kill -0 "$pid" 2>/dev/null; then
        echo -e "${RED}✗ 停止失败${NC}"
        exit 1
    else
        echo -e "${GREEN}✓ $APP_NAME 已停止${NC}"
    fi
}

# 重启服务
restart() {
    echo -e "${GREEN}正在重启 $APP_NAME...${NC}"
    stop
    sleep 2
    start
}

# 查看状态
status() {
    if is_running; then
        local pid=$(get_pid)
        echo -e "${GREEN}✓ $APP_NAME 正在运行${NC}"
        echo -e "PID: $pid"
        echo -e "日志: $LOG_FILE"
        echo ""
        echo "进程信息:"
        ps -p "$pid" -o pid,ppid,cmd,%mem,%cpu,etime
    else
        echo -e "${RED}✗ $APP_NAME 未运行${NC}"
        if [ -f "$PID_FILE" ]; then
            echo -e "${YELLOW}发现残留的 PID 文件，正在清理...${NC}"
            rm -f "$PID_FILE"
        fi
    fi
}

# 查看日志
logs() {
    if [ ! -f "$LOG_FILE" ]; then
        echo -e "${RED}日志文件不存在: $LOG_FILE${NC}"
        exit 1
    fi

    if [ "$1" == "-f" ]; then
        echo -e "${GREEN}实时查看日志 (Ctrl+C 退出):${NC}"
        tail -f "$LOG_FILE"
    else
        echo -e "${GREEN}最近 50 行日志:${NC}"
        tail -n 50 "$LOG_FILE"
    fi
}

# 显示帮助
usage() {
    echo "用法: $0 {start|stop|restart|status|logs|logs -f}"
    echo ""
    echo "命令:"
    echo "  start    - 启动服务"
    echo "  stop     - 停止服务"
    echo "  restart  - 重启服务"
    echo "  status   - 查看服务状态"
    echo "  logs     - 查看最近日志"
    echo "  logs -f  - 实时查看日志"
    echo ""
    exit 1
}

# 主逻辑
case "$1" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    status)
        status
        ;;
    logs)
        logs "$2"
        ;;
    *)
        usage
        ;;
esac

exit 0
