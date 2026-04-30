#!/usr/bin/env bash
# start.sh — one-command launcher for PDE Agent
#
# Starts three services in the background and tails their logs:
#   - Knowledge Base  (Rust)   → http://localhost:3001
#   - Solver API      (Rust)   → http://localhost:3000
#   - Frontend        (Vite)   → http://localhost:5173
#
# Usage:
#   ./start.sh            # start all services
#   ./start.sh --stop     # kill all previously started services
#   ./start.sh --logs     # tail logs without restarting

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/.logs"
PID_FILE="$SCRIPT_DIR/.pde-agent.pids"

KB_PORT="${KB_PORT:-3001}"
SOLVER_PORT="${SOLVER_PORT:-3000}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

log()  { echo -e "${BOLD}[pde-agent]${RESET} $*"; }
ok()   { echo -e "${GREEN}[pde-agent]${RESET} $*"; }
warn() { echo -e "${YELLOW}[pde-agent]${RESET} $*"; }
err()  { echo -e "${RED}[pde-agent]${RESET} $*" >&2; }

# ── Stop ──────────────────────────────────────────────────────────────────────
stop_all() {
    if [[ ! -f "$PID_FILE" ]]; then
        warn "No PID file found; nothing to stop."
        return
    fi
    log "Stopping services..."
    while IFS= read -r pid; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" && ok "  killed PID $pid"
        fi
    done < "$PID_FILE"
    rm -f "$PID_FILE"
    ok "All services stopped."
}

# ── Wait for port ─────────────────────────────────────────────────────────────
wait_for_port() {
    local name="$1" port="$2" timeout="${3:-30}"
    local elapsed=0
    printf "  Waiting for %s on :%s " "$name" "$port"
    while ! (echo >/dev/tcp/localhost/"$port") 2>/dev/null; do
        sleep 1
        elapsed=$((elapsed + 1))
        printf "."
        if [[ $elapsed -ge $timeout ]]; then
            echo
            err "  $name did not become ready within ${timeout}s — check .logs/${name}.log"
            return 1
        fi
    done
    echo
    ok "  $name is up on :$port"
}

# ── Build helpers ─────────────────────────────────────────────────────────────
build_rust() {
    local name="$1" dir="$2"
    log "Building $name (cargo build --release)..."
    if ! cargo build --release --manifest-path "$dir/Cargo.toml" 2>&1 | \
            tee "$LOG_DIR/${name}-build.log" | grep -E "^error|Compiling|Finished"; then
        err "Build failed — see $LOG_DIR/${name}-build.log"
        exit 1
    fi
}

# ── Start ─────────────────────────────────────────────────────────────────────
start_all() {
    mkdir -p "$LOG_DIR"
    > "$PID_FILE"

    # ── 1. Knowledge Base ─────────────────────────────────────────────────────
    log "Starting Knowledge Base service..."
    build_rust "knowledge-base" "$SCRIPT_DIR/knowledge_base"

    KB_BIN="$SCRIPT_DIR/knowledge_base/target/release/knowledge-base"
    KB_BIND_ADDR="0.0.0.0:${KB_PORT}" \
    KB_DB_PATH="${KB_DB_PATH:-$SCRIPT_DIR/knowledge_base/knowledge_base.db}" \
    KB_INDEX_PATH="${KB_INDEX_PATH:-$SCRIPT_DIR/knowledge_base/vector_index.bin}" \
        "$KB_BIN" > "$LOG_DIR/knowledge-base.log" 2>&1 &
    echo $! >> "$PID_FILE"
    wait_for_port "knowledge-base" "$KB_PORT"

    # ── 2. Solver API ─────────────────────────────────────────────────────────
    log "Starting Solver API service..."
    build_rust "pde-solver-api" "$SCRIPT_DIR/solvers/api"

    SOLVER_BIN="$SCRIPT_DIR/solvers/api/target/release/pde-solver-api"
    LISTEN_ADDR="0.0.0.0:${SOLVER_PORT}" \
        "$SOLVER_BIN" > "$LOG_DIR/solver-api.log" 2>&1 &
    echo $! >> "$PID_FILE"
    wait_for_port "solver-api" "$SOLVER_PORT"

    # ── 3. Frontend ───────────────────────────────────────────────────────────
    log "Starting Frontend (Vite dev server)..."
    (cd "$SCRIPT_DIR/frontend" && \
        npm run dev -- --port "$FRONTEND_PORT" --host) \
        > "$LOG_DIR/frontend.log" 2>&1 &
    echo $! >> "$PID_FILE"
    wait_for_port "frontend" "$FRONTEND_PORT"

    echo
    ok "${BOLD}All services are running.${RESET}"
    echo
    echo -e "  ${CYAN}Knowledge Base${RESET}  →  http://localhost:${KB_PORT}"
    echo -e "  ${CYAN}Solver API${RESET}      →  http://localhost:${SOLVER_PORT}"
    echo -e "  ${CYAN}Frontend${RESET}        →  http://localhost:${FRONTEND_PORT}"
    echo
    echo -e "  Logs: ${LOG_DIR}/"
    echo -e "  Stop: ${BOLD}./start.sh --stop${RESET}"
    echo
    log "Tailing logs (Ctrl-C to detach — services keep running)..."
    tail -f "$LOG_DIR/knowledge-base.log" "$LOG_DIR/solver-api.log" "$LOG_DIR/frontend.log"
}

# ── Entry point ───────────────────────────────────────────────────────────────
case "${1:-}" in
    --stop)
        stop_all
        ;;
    --logs)
        if [[ ! -d "$LOG_DIR" ]]; then
            err "No log directory found. Have you run ./start.sh yet?"
            exit 1
        fi
        tail -f "$LOG_DIR/knowledge-base.log" "$LOG_DIR/solver-api.log" "$LOG_DIR/frontend.log"
        ;;
    "")
        # If services from a previous run are still alive, stop them first
        if [[ -f "$PID_FILE" ]]; then
            warn "Found existing PID file — stopping previous run first."
            stop_all
        fi
        start_all
        ;;
    *)
        echo "Usage: $0 [--stop | --logs]"
        exit 1
        ;;
esac
