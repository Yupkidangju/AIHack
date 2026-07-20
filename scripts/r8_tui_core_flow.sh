#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
SESSION="aihack-r8-core-flow-$$"
TMP_DIR=$(mktemp -d)
EXIT_FILE="$TMP_DIR/exit.status"

cleanup() {
    tmux kill-session -t "$SESSION" 2>/dev/null || true
    rm -rf -- "$TMP_DIR"
}
trap cleanup EXIT

wait_for_text() {
    local expected=$1
    for _ in $(seq 1 150); do
        if tmux capture-pane -p -t "$SESSION" | grep -Fq "$expected"; then
            return 0
        fi
        sleep 0.02
    done
    tmux capture-pane -p -t "$SESSION"
    return 1
}

command -v tmux >/dev/null
cargo build --manifest-path "$ROOT/Cargo.toml" -p aihack-tui --bin aihack --locked

tmux new-session -d -s "$SESSION" -x 120 -y 36 \
    "env AIHACK_LLM_ENABLED=false '$ROOT/target/debug/aihack' --seed 42 --high-contrast --reduced-motion; printf '%s\n' \"\$?\" > '$EXIT_FILE'; exec bash"

wait_for_text "Press Enter to Start"
tmux send-keys -t "$SESSION" Enter
wait_for_text "Character Creation"
tmux send-keys -t "$SESSION" Enter
wait_for_text "LLM: OFF"
tmux send-keys -t "$SESSION" i
wait_for_text "[i] Inventory"

game_over=false
for _ in $(seq 1 300); do
    tmux send-keys -t "$SESSION" .
    if tmux capture-pane -p -t "$SESSION" | grep -Fq "GAME OVER"; then
        game_over=true
        break
    fi
    sleep 0.01
done
[[ "$game_over" == true ]]

tmux send-keys -t "$SESSION" N
wait_for_text "Press Enter to Start"
tmux send-keys -t "$SESSION" q
for _ in $(seq 1 100); do
    [[ -s "$EXIT_FILE" ]] && break
    sleep 0.01
done
[[ "$(<"$EXIT_FILE")" -eq 0 ]]

tmux kill-session -t "$SESSION"
rm -f -- "$EXIT_FILE"
tmux new-session -d -s "$SESSION" -x 59 -y 23 \
    "env AIHACK_LLM_ENABLED=false '$ROOT/target/debug/aihack' --seed 42; printf '%s\n' \"\$?\" > '$EXIT_FILE'; exec bash"
wait_for_text "terminal requires 60x24"
tmux send-keys -t "$SESSION" q
for _ in $(seq 1 100); do
    [[ -s "$EXIT_FILE" ]] && break
    sleep 0.01
done
[[ "$(<"$EXIT_FILE")" -eq 0 ]]

printf '%s\n' 'PASS core-flow: Title -> Character Creation -> Playing -> Inventory -> Game Over -> New Run -> undersized clean exit'
