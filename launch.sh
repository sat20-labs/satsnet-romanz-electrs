#!/bin/bash

# 定义 electrs 可执行文件的路径
ELECTRS_PATH="/data/github/satsnet-romanz-electrs/target/release/electrs"
# 定义配置文件路径
CONFIG_PATH="/data/github/satsnet-romanz-electrs/config.toml"
# 定义日志文件路径
LOG_FILE="/var/log/electrs.log"
# 定义每次检查的间隔时间（秒）
WAIT_TIME=30
# 定义等待进程正常退出的最长时间（秒）
KILL_TIMEOUT=10

# 函数：启动 electrs 并记录日志
start_electrs() {
    echo "Starting electrs..."
    # 使用 nohup 启动 electrs，将输出重定向到日志文件，并在后台运行
    nohup $ELECTRS_PATH --conf $CONFIG_PATH >> $LOG_FILE 2>&1 &
    echo "electrs started with PID $!"
}

# 无限循环，持续运行脚本
while true; do
    # 获取 electrs 进程的 PID
    PID=$(ps aux | grep "[${ELECTRS_PATH}]" | awk '{print $2}')

    # 检查是否找到了 electrs 进程
    if [ ! -z "$PID" ]; then
        echo "Found electrs process with PID: $PID"
        echo "Stopping electrs..."
        # 尝试正常终止进程
        kill $PID

        # 等待进程停止，最多等待 KILL_TIMEOUT 秒
        for ((i=0; i<KILL_TIMEOUT; i++)); do
            if ! kill -0 $PID 2>/dev/null; then
                echo "electrs stopped"
                break
            fi
            sleep 1
        done

        # 如果进程仍然存在，强制 KILL
        if kill -0 $PID 2>/dev/null; then
            echo "electrs did not stop gracefully. Force killing..."
            kill -9 $PID
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