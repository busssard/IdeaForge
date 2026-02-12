#!/usr/bin/env bash
set -euo pipefail

# IdeaForge Local Development Runner
# Starts PostgreSQL, runs migrations, seeds data, starts backend + frontend.
# Usage: ./run_dev.sh

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$REPO_DIR/src"
FRONTEND_DIR="$SRC_DIR/crates/ideaforge-frontend"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down...${NC}"
    # Kill background processes
    [ -n "${BACKEND_PID:-}" ] && kill "$BACKEND_PID" 2>/dev/null && echo "  Stopped backend (PID $BACKEND_PID)"
    [ -n "${FRONTEND_PID:-}" ] && kill "$FRONTEND_PID" 2>/dev/null && echo "  Stopped frontend (PID $FRONTEND_PID)"
    echo -e "${GREEN}Done. Run 'docker compose down' to stop PostgreSQL.${NC}"
    exit 0
}
trap cleanup SIGINT SIGTERM

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

# --- 3. Build workspace ---
echo -e "${GREEN}[3/6] Building workspace...${NC}"
cd "$SRC_DIR"
cargo build --bin ideaforge --bin migrate --bin seed 2>&1

# --- 4. Migrate + Seed ---
echo -e "${GREEN}[4/6] Running migrations...${NC}"
cargo run --bin migrate

echo -e "${GREEN}[5/6] Seeding database...${NC}"
cargo run --bin seed

# --- 6. Start servers ---
echo -e "${GREEN}[6/6] Starting servers...${NC}"

# Backend on :3000
cargo run --bin ideaforge &
BACKEND_PID=$!
echo "  Backend starting (PID $BACKEND_PID) on http://localhost:3000"

# Wait for backend to be ready
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

# Wait for either process to exit
wait
