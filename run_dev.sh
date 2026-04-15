#!/usr/bin/env bash
set -euo pipefail

# IdeaForge Local Development Runner
# Starts PostgreSQL, runs migrations, seeds data, starts backend + frontend.
#
# Default usage:      ./run_dev.sh              (start everything)
# Useful flags:
#   --kill            stop any IdeaForge processes on :3000 / :8080 and exit
#   --migrate         run migrations (no server start)
#   --truncate-mls    wipe all MLS messaging state (keystores, groups, messages,
#                     welcomes, keypackages), then continue with normal startup
#   --truncate-all    wipe ALL user data (keeps schema), re-seed, then continue
#   --fresh           alias for --truncate-all
#   --restart-backend kill + rebuild + restart the backend only, then exit
#   --no-start        run maintenance ops but don't start servers at the end
#   --help            show this help and exit
#
# Flags compose: e.g. `./run_dev.sh --truncate-mls` wipes MLS tables then
# starts the stack. `./run_dev.sh --truncate-all --no-start` wipes everything
# and leaves you at a shell prompt.

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$REPO_DIR/src"
FRONTEND_DIR="$SRC_DIR/crates/ideaforge-frontend"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# --- Parse flags ---
WANT_START=1
WANT_MIGRATE=0
WANT_SEED=1
WANT_TRUNCATE_MLS=0
WANT_TRUNCATE_ALL=0
WANT_KILL_ONLY=0
WANT_RESTART_BACKEND=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --kill)             WANT_KILL_ONLY=1 ;;
        --migrate)          WANT_MIGRATE=1; WANT_START=0; WANT_SEED=0 ;;
        --truncate-mls)     WANT_TRUNCATE_MLS=1 ;;
        --truncate-all|--fresh) WANT_TRUNCATE_ALL=1 ;;
        --restart-backend)  WANT_RESTART_BACKEND=1 ;;
        --no-start)         WANT_START=0 ;;
        --help|-h)
            sed -n '3,22p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown flag: $1${NC}"
            echo "Try --help."
            exit 1
            ;;
    esac
    shift
done

cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down...${NC}"
    [ -n "${BACKEND_PID:-}" ] && kill "$BACKEND_PID" 2>/dev/null && echo "  Stopped backend (PID $BACKEND_PID)"
    [ -n "${FRONTEND_PID:-}" ] && kill "$FRONTEND_PID" 2>/dev/null && echo "  Stopped frontend (PID $FRONTEND_PID)"
    echo -e "${GREEN}Done. Run 'docker compose down' to stop PostgreSQL.${NC}"
    exit 0
}
trap cleanup SIGINT SIGTERM

kill_port() {
    local port="$1"
    if lsof -ti:"$port" >/dev/null 2>&1; then
        echo -e "${YELLOW}  Killing process on port ${port}...${NC}"
        kill $(lsof -ti:"$port") 2>/dev/null || true
        sleep 1
    fi
}

# --- --kill: stop ideaforge processes and exit ---
if [[ $WANT_KILL_ONLY -eq 1 ]]; then
    echo -e "${GREEN}Stopping IdeaForge processes...${NC}"
    kill_port 3000
    kill_port 8080
    # `trunk serve` can spawn a watcher too — kill any matching process.
    pkill -f "trunk serve" 2>/dev/null || true
    pkill -f "target/debug/ideaforge" 2>/dev/null || true
    echo -e "${GREEN}Done. Run 'docker compose down' to stop PostgreSQL too.${NC}"
    exit 0
fi

# --- --restart-backend: kill + rebuild + restart backend only ---
if [[ $WANT_RESTART_BACKEND -eq 1 ]]; then
    echo -e "${GREEN}Restarting backend only...${NC}"
    kill_port 3000
    pkill -f "target/debug/ideaforge" 2>/dev/null || true
    cd "$SRC_DIR"
    cargo build --bin ideaforge
    # Start detached; logs go to /tmp so the caller's terminal stays free.
    nohup cargo run --bin ideaforge > /tmp/ideaforge-backend.log 2>&1 &
    NEW_PID=$!
    echo "  Backend restarting (PID $NEW_PID). Logs: tail -f /tmp/ideaforge-backend.log"
    for i in $(seq 1 30); do
        if curl -sf http://localhost:3000/health >/dev/null 2>&1; then
            echo -e "${GREEN}  Backend is ready.${NC}"
            exit 0
        fi
        sleep 1
    done
    echo -e "${RED}  Backend did not become healthy in 30s. Check /tmp/ideaforge-backend.log${NC}"
    exit 1
fi

# --- 1. Check prerequisites ---
echo -e "${GREEN}[1/6] Checking prerequisites...${NC}"

if ! command -v docker &>/dev/null; then
    echo -e "${RED}Error: docker is not installed.${NC}"
    exit 1
fi
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}Error: cargo is not installed. Install Rust via rustup.${NC}"
    exit 1
fi
if ! command -v trunk &>/dev/null; then
    echo -e "${YELLOW}trunk is not installed. Installing...${NC}"
    cargo install trunk
fi
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo -e "${YELLOW}Adding wasm32-unknown-unknown target...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# --- 2. Start PostgreSQL ---
echo -e "${GREEN}[2/6] Starting PostgreSQL...${NC}"
cd "$REPO_DIR"
docker compose up -d postgres
echo "  Waiting for PostgreSQL to be ready..."
until docker compose exec postgres pg_isready -U ideaforge >/dev/null 2>&1; do
    sleep 1
done
echo "  PostgreSQL is ready (port 5433)."

# Helper: run SQL against the dev DB.
run_sql() {
    docker compose exec -T postgres psql -U ideaforge -d ideaforge "$@"
}

# --- Maintenance ops (before building/migrating) ---
if [[ $WANT_TRUNCATE_ALL -eq 1 ]]; then
    echo -e "${BLUE}Wiping ALL user data (schema preserved)...${NC}"
    # Lists every table that the migrations create, in FK-safe order.
    run_sql <<'SQL'
TRUNCATE TABLE
    mls_messages,
    mls_welcomes,
    mls_group_members,
    mls_groups,
    mls_keypackages,
    mls_keystore,
    notifications,
    board_tasks,
    nda_signatures,
    nda_templates,
    flags,
    invite_links,
    bot_endorsements,
    subscriptions,
    team_applications,
    team_members,
    contributions,
    stokes,
    ideas,
    categories,
    users
RESTART IDENTITY CASCADE;
SQL
    echo -e "${GREEN}  Done. Schema intact; all rows gone.${NC}"
    # Re-seed after a full wipe so the test accounts exist.
    WANT_SEED=1
elif [[ $WANT_TRUNCATE_MLS -eq 1 ]]; then
    echo -e "${BLUE}Wiping MLS messaging state (keeping users, ideas, etc)...${NC}"
    run_sql <<'SQL'
TRUNCATE TABLE
    mls_messages,
    mls_welcomes,
    mls_group_members,
    mls_groups,
    mls_keypackages,
    mls_keystore
RESTART IDENTITY CASCADE;
-- Orphaned notifications for messages whose groups we just wiped. Safe to
-- drop; the inbox UX will regenerate fresh ones as new messages arrive.
DELETE FROM notifications WHERE kind = 'message';
SQL
    echo -e "${GREEN}  Done. Users and ideas preserved.${NC}"
fi

# --- 3. Build workspace ---
echo -e "${GREEN}[3/6] Building workspace...${NC}"
cd "$SRC_DIR"
cargo build --bin ideaforge --bin migrate --bin seed 2>&1

# --- 4. Migrate ---
echo -e "${GREEN}[4/6] Running migrations...${NC}"
cargo run --bin migrate
if [[ $WANT_MIGRATE -eq 1 && $WANT_START -eq 0 ]]; then
    echo -e "${GREEN}Migration complete (no server start requested).${NC}"
    exit 0
fi

# --- 5. Seed ---
if [[ $WANT_SEED -eq 1 ]]; then
    echo -e "${GREEN}[5/6] Seeding database...${NC}"
    cargo run --bin seed
else
    echo -e "${YELLOW}[5/6] Skipping seed.${NC}"
fi

# --- 6. Start servers ---
if [[ $WANT_START -eq 0 ]]; then
    echo -e "${GREEN}Setup complete. Not starting servers (--no-start).${NC}"
    exit 0
fi

echo -e "${GREEN}[6/6] Starting servers...${NC}"
kill_port 3000
kill_port 8080

# Backend on :3000
cargo run --bin ideaforge &
BACKEND_PID=$!
echo "  Backend starting (PID $BACKEND_PID) on http://localhost:3000"

echo "  Waiting for backend..."
for i in $(seq 1 30); do
    if curl -sf http://localhost:3000/health >/dev/null 2>&1; then
        echo "  Backend is ready."
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo -e "${RED}  Backend failed to start within 30s.${NC}"
        cleanup
    fi
    sleep 1
done

# Frontend on :8080
cd "$FRONTEND_DIR"
trunk serve &
FRONTEND_PID=$!
echo "  Frontend starting (PID $FRONTEND_PID) on http://localhost:8080"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  IdeaForge is running!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "  Backend API:  http://localhost:3000"
echo "  Frontend:     http://localhost:8080"
echo "  Health check: http://localhost:3000/health"
echo ""
echo "  Test accounts:"
echo "    alice@example.com / Test1234!  (Entrepreneur)"
echo "    bob@example.com   / Test1234!  (Maker)"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop everything.${NC}"
echo ""

wait
