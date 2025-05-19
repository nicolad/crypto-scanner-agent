#!/usr/bin/env bash
set -e
clear
RUST_LOG=debug shuttle run --secrets Secrets.toml &
PID=$!

URL="http://localhost:8000/"
# Wait until the server is reachable
for _ in {1..30}; do
    if nc -z localhost 8000; then
        break
    fi
    sleep 1
done

if ! nc -z localhost 8000; then
    echo "Server failed to start at $URL"
    kill $PID
    wait $PID
    exit 1
fi

if command -v xdg-open >/dev/null; then
    xdg-open "$URL" >/dev/null 2>&1 &
elif command -v open >/dev/null; then
    open "$URL" >/dev/null 2>&1 &
else
    echo "Server running at $URL"
fi

wait $PID
