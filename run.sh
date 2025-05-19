#!/usr/bin/env bash
clear
RUST_LOG=debug shuttle run --secrets Secrets.toml &
PID=$!
# Wait briefly for the server to start listening
sleep 2

URL="http://localhost:8000/"
if command -v xdg-open >/dev/null; then
    xdg-open "$URL" >/dev/null 2>&1 &
elif command -v open >/dev/null; then
    open "$URL" >/dev/null 2>&1 &
else
    echo "Server running at $URL"
fi

wait $PID
