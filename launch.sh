#!/bin/bash

ELECTRS_PATH="/data/github/satsnet-romanz-electrs/target/release/electrs"

while true; do
    # 获取 electrs 进程的 PID
    PID=$(ps aux | grep "[${ELECTRS_PATH}]" | awk '{print $2}')

    if [ ! -z "$PID" ]; then
        echo "Found electrs process with PID: $PID"
        echo "Stopping electrs..."
        kill $PID

        # 等待进程完全停止
        while kill -0 $PID 2>/dev/null; do
            sleep 1
        done
        echo "electrs stopped"

        # 重新启动 electrs
        echo "Starting electrs..."
        $ELECTRS_PATH --conf ./config.toml &
        echo "electrs started"
    else
        echo "electrs process not found, starting it..."
        $ELECTRS_PATH --conf ./config.toml &
        echo "electrs started"
    fi

    # 等待 30 秒
    sleep 30
done