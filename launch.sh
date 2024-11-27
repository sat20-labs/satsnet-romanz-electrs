#!/bin/bash

# 定义 electrs 可执行文件的路径
ELECTRS_PATH="/data/github/satsnet-romanz-electrs/target/release/electrs"
# 定义配置文件路径
CONFIG_PATH="/data/github/satsnet-romanz-electrs/config.toml"
# 定义日志文件路径
LOG_FILE="/var/log/electrs.log"
# 定义每次检查的间隔时间（秒）
WAIT_TIME=30

# 函数：启动 electrs 并记录日志
start_electrs() {
    echo "Starting electrs..."
    # 使用 nohup 启动 electrs，将输出重定向到日志文件，并在后台运行
    nohup $ELECTRS_PATH --conf $CONFIG_PATH >> $LOG_FILE 2>&1 &
    echo "electrs started with PID $!"
}

# 函数：获取 electrs 进程的 PID
get_electrs_pid() {
    pgrep -f "$ELECTRS_PATH"
}

# 无限循环，持续运行脚本
while true; do
    # 获取 electrs 进程的 PID
    PID=$(get_electrs_pid)

    # 检查是否找到了 electrs 进程
    if [ ! -z "$PID" ]; then
        echo "Found electrs process with PID: $PID"
        echo "Forcefully stopping electrs..."
        # 直接使用 KILL -9 强制终止进程
        kill -9 $PID
        sleep 1  # 短暂等待，确保系统有时间处理终止的进程

        # 再次检查进程是否真的被终止
        if [ -z "$(get_electrs_pid)" ]; then
            echo "electrs successfully terminated"
        else
            echo "Failed to terminate electrs, retrying..."
            kill -9 $(get_electrs_pid)
            sleep 1
        fi

        # 重新启动 electrs
        start_electrs
    else
        # 如果没有找到 electrs 进程，则启动它
        echo "electrs process not found, starting it..."
        start_electrs
    fi

    # 等待指定的时间再进行下一次检查
    echo "Waiting for $WAIT_TIME seconds before next check..."
    sleep $WAIT_TIME
done