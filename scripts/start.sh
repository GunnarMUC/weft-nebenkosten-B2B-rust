#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

source .env 2>/dev/null || true

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   NK-Check Industrie -- Start${NC}"
echo -e "${BLUE}========================================${NC}"

case "${1:-all}" in
    all)
        echo -e "\n${YELLOW}Starte alle Services ...${NC}"
        docker compose up -d postgres redis restate vllm
        sleep 5
        docker compose up -d chunker-web chunker-worker \
            weft-orchestrator weft-api weft-node-runner \
            weft-dashboard operator-ui
        ;;

    database)
        echo -e "\n${YELLOW}Starte Datenbanken ...${NC}"
        docker compose up -d postgres redis
        ;;

    llm)
        echo -e "\n${YELLOW}Starte vLLM ...${NC}"
        docker compose --profile gpu up -d vllm
        ;;

    chunker)
        echo -e "\n${YELLOW}Starte Chunker ...${NC}"
        docker compose up -d chunker-web chunker-worker
        ;;

    weft)
        echo -e "\n${YELLOW}Starte Weft Backend ...${NC}"
        docker compose up -d restate weft-orchestrator weft-api weft-node-runner
        ;;

    frontend)
        echo -e "\n${YELLOW}Starte Frontends ...${NC}"
        docker compose up -d weft-dashboard operator-ui
        ;;

    stop)
        echo -e "\n${YELLOW}Stoppe alle Services ...${NC}"
        docker compose down
        ;;

    logs)
        docker compose logs -f "${2:-}"
        ;;

    status)
        docker compose ps
        ;;

    *)
        echo "Usage: ./scripts/start.sh [all|database|llm|chunker|weft|frontend|stop|logs|status]"
        exit 1
        ;;
esac

if [ "${1:-all}" != "stop" ] && [ "${1:-all}" != "logs" ] && [ "${1:-all}" != "status" ]; then
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}   Services gestartet${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo -e "  Chunker UI:      http://localhost:${CHUNKER_PORT:-5000}"
    echo -e "  Weft Dashboard:  http://localhost:${DASHBOARD_PORT:-5173}"
    echo -e "  Operator UI:     http://localhost:${OPERATOR_UI_PORT:-5174}"
    echo -e "  vLLM API:        http://localhost:${VLLM_PORT:-8000}"
    echo -e "  Restate:         http://localhost:${RESTATE_PORT:-8080}"
    echo ""
    echo -e "${YELLOW}Status: ./scripts/start.sh status${NC}"
    echo -e "${YELLOW}Logs:   ./scripts/start.sh logs [service]${NC}"
    echo -e "${YELLOW}Stop:   ./scripts/start.sh stop${NC}"
fi
