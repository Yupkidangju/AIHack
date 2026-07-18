#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
SESSION="aihack-r6-exit-$$"
TMP_DIR=$(mktemp -d)
READY_FILE="$TMP_DIR/fixture.port"
EXIT_FILE="$TMP_DIR/exit.status"
STTY_FILE="$TMP_DIR/stty.txt"
FIXTURE_PID=""

cleanup() {
    tmux kill-session -t "$SESSION" 2>/dev/null || true
    if [[ -n "$FIXTURE_PID" ]]; then
        kill "$FIXTURE_PID" 2>/dev/null || true
        wait "$FIXTURE_PID" 2>/dev/null || true
    fi
    rm -rf -- "$TMP_DIR"
}
trap cleanup EXIT

monotonic_ms() {
    python3 -c 'import time; print(time.monotonic_ns() // 1_000_000)'
}

command -v tmux >/dev/null
command -v python3 >/dev/null

cargo build --manifest-path "$ROOT/Cargo.toml" -p aihack-tui --bin aihack --locked
python3 "$ROOT/scripts/r6_loopback_fixture.py" \
    --port 0 \
    --delay-ms 5000 \
    --max-requests 1 \
    --ready-file "$READY_FILE" &
FIXTURE_PID=$!

for _ in $(seq 1 100); do
    [[ -s "$READY_FILE" ]] && break
    sleep 0.02
done
[[ -s "$READY_FILE" ]]
PORT=$(<"$READY_FILE")

tmux new-session -d -s "$SESSION" -x 80 -y 24 \
    "env AIHACK_LLM_ENABLED=true AIHACK_LLM_ENDPOINT=http://127.0.0.1:$PORT/v1 AIHACK_LLM_MODEL=r6-fixture AIHACK_LLM_NARRATIVE_TIMEOUT_MS=10000 '$ROOT/target/debug/aihack' --seed 42; status=\$?; stty -a > '$STTY_FILE'; printf '%s\n' \"\$status\" > '$EXIT_FILE'; exec bash"

sleep 0.15
tmux send-keys -t "$SESSION" Enter
sleep 0.08
tmux send-keys -t "$SESSION" Enter
sleep 0.08
tmux send-keys -t "$SESSION" G

WAIT_SEEN=0
for _ in $(seq 1 100); do
    if tmux capture-pane -p -t "$SESSION" | grep -q 'LLM: WAIT'; then
        WAIT_SEEN=1
        break
    fi
    sleep 0.02
done
[[ "$WAIT_SEEN" -eq 1 ]]

START_MS=$(monotonic_ms)
tmux send-keys -t "$SESSION" q

RESTORED_BEFORE_EXIT=0
for _ in $(seq 1 100); do
    if ! tmux capture-pane -p -t "$SESSION" | grep -q 'LLM: WAIT'; then
        if [[ ! -e "$EXIT_FILE" ]]; then
            RESTORED_BEFORE_EXIT=1
        fi
        break
    fi
    sleep 0.005
done

for _ in $(seq 1 100); do
    [[ -s "$EXIT_FILE" ]] && break
    sleep 0.01
done
END_MS=$(monotonic_ms)

[[ "$RESTORED_BEFORE_EXIT" -eq 1 ]]
[[ "$(<"$EXIT_FILE")" -eq 0 ]]
grep -Eq '(^|[ ;])icanon([ ;]|$)' "$STTY_FILE"
grep -Eq '(^|[ ;])echo([ ;]|$)' "$STTY_FILE"
(( END_MS - START_MS < 1000 ))

printf 'PASS pending-exit: restore-before-worker-wait, elapsed=%dms\n' "$((END_MS - START_MS))"
