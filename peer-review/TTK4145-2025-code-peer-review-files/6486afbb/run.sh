#!/bin/bash
if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <id> <port>"
    exit 1
fi

while true; do
    go build -o main heislab/main.go
    ./main -id="$1" -port="$2"
    exit_code=$?
    if [ $exit_code -eq 42 ]; then
        echo "Restarting to  (exit code 42)"
        sleep 1
    else
        echo "Exiting due to exit code $exit_code"
        break
    fi
done