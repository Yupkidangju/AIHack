#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
PREFIX="aihack-r6-matrix-$$"
TMP_DIR=$(mktemp -d)
ACTIVE_SESSION=""
FIXTURE_PID=""

stop_case() {
    if [[ -n "$ACTIVE_SESSION" ]]; then
        tmux kill-session -t "$ACTIVE_SESSION" 2>/dev/null || true
        ACTIVE_SESSION=""
    fi
    if [[ -n "$FIXTURE_PID" ]]; then
        kill "$FIXTURE_PID" 2>/dev/null || true
        wait "$FIXTURE_PID" 2>/dev/null || true
        FIXTURE_PID=""
    fi
}

cleanup() {
    stop_case
    rm -rf -- "$TMP_DIR"
}
trap cleanup EXIT

command -v tmux >/dev/null
command -v python3 >/dev/null
cargo build --manifest-path "$ROOT/Cargo.toml" -p aihack-tui --bin aihack --locked

wait_for_text() {
    local session=$1
    local expected=$2
    for _ in $(seq 1 150); do
        if tmux capture-pane -p -t "$session" | grep -Fq "$expected"; then
            return 0
        fi
        sleep 0.02
    done
    tmux capture-pane -p -t "$session"
    return 1
}

start_tui() {
    local name=$1 width=$2 height=$3 port=$4 narrative_timeout=$5 decision_timeout=$6
    local exit_file="$TMP_DIR/$name.exit"
    ACTIVE_SESSION="$PREFIX-$name"
    tmux new-session -d -s "$ACTIVE_SESSION" -x "$width" -y "$height" \
        "env AIHACK_LLM_ENABLED=true AIHACK_LLM_ENDPOINT=http://127.0.0.1:$port/v1 AIHACK_LLM_MODEL=r6-fixture AIHACK_LLM_NARRATIVE_TIMEOUT_MS=$narrative_timeout AIHACK_LLM_DECISION_TIMEOUT_MS=$decision_timeout '$ROOT/target/debug/aihack' --seed 42; printf '%s\n' \"\$?\" > '$exit_file'; exec bash"
    sleep 0.12
    tmux send-keys -t "$ACTIVE_SESSION" Enter
    sleep 0.06
    tmux send-keys -t "$ACTIVE_SESSION" Enter
    sleep 0.06
}

finish_case() {
    local name=$1
    local exit_file="$TMP_DIR/$name.exit"
    tmux send-keys -t "$ACTIVE_SESSION" q
    for _ in $(seq 1 100); do
        [[ -s "$exit_file" ]] && break
        sleep 0.01
    done
    [[ "$(<"$exit_file")" -eq 0 ]]
    stop_case
}

run_fixture_case() {
    local name=$1 width=$2 height=$3 delay_ms=$4 timeout_ms=$5 request_key=$6 expected=$7
    local advance_key=${8:-}
    local ready_file="$TMP_DIR/$name.port"
    python3 "$ROOT/scripts/r6_loopback_fixture.py" \
        --port 0 --delay-ms "$delay_ms" --max-requests 1 --ready-file "$ready_file" &
    FIXTURE_PID=$!
    for _ in $(seq 1 100); do
        [[ -s "$ready_file" ]] && break
        sleep 0.02
    done
    [[ -s "$ready_file" ]]
    start_tui "$name" "$width" "$height" "$(<"$ready_file")" "$timeout_ms" "$timeout_ms"
    tmux send-keys -t "$ACTIVE_SESSION" "$request_key"
    wait_for_text "$ACTIVE_SESSION" 'LLM: WAIT'
    if [[ -n "$advance_key" ]]; then
        sleep 0.03
        tmux send-keys -t "$ACTIVE_SESSION" "$advance_key"
    fi
    wait_for_text "$ACTIVE_SESSION" "$expected"
    finish_case "$name"
    printf 'PASS %s: %s\n' "$name" "$expected"
}

run_fixture_case success 80 24 100 1000 G '[N] Dismiss'
run_fixture_case timeout 60 24 400 100 G 'LLM: TIMEOUT'
run_fixture_case stale 120 36 300 1000 A 'LLM: STALE' .

CLOSED_PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
start_tui down 60 24 "$CLOSED_PORT" 500 500
tmux send-keys -t "$ACTIVE_SESSION" G
wait_for_text "$ACTIVE_SESSION" 'LLM: DOWN'
finish_case down
printf 'PASS down: LLM: DOWN\n'
