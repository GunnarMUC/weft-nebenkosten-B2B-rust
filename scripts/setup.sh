#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   NK-Check Industrie -- Setup${NC}"
echo -e "${BLUE}========================================${NC}"

cd "$PROJECT_DIR"

# ── .env Pruefung ──
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}Keine .env gefunden. Kopiere .env.example ...${NC}"
    cp .env.example .env
    echo -e "${GREEN}✓ .env erstellt. Bitte Werte anpassen:${NC}"
    echo -e "  vim .env"
    echo -e "${YELLOW}Danach erneut: ./scripts/setup.sh${NC}"
    exit 1
fi

source .env

# ── System-Abhaengigkeiten ──
echo -e "\n${BLUE}[1/6] Pruefe System-Abhaengigkeiten ...${NC}"

check_cmd() {
    if command -v "$1" &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} $1 ($($1 --version 2>&1 | head -1))"
    else
        echo -e "  ${RED}✗${NC} $1 nicht gefunden"
        MISSING_DEPS=true
    fi
}

MISSING_DEPS=false
check_cmd docker
check_cmd git
check_cmd curl

if [ "$MISSING_DEPS" = true ]; then
    echo -e "\n${RED}Fehlende Abhaengigkeiten. Bitte installieren:${NC}"
    echo "  Ubuntu: sudo apt install docker.io git curl"
    echo "  macOS:  brew install docker git curl"
    exit 1
fi

# ── Datenverzeichnisse ──
echo -e "\n${BLUE}[2/6] Erstelle Datenverzeichnisse ...${NC}"
mkdir -p data/uploads data/chunks data/output
echo -e "${GREEN}✓ Verzeichnisse erstellt${NC}"

# ── Weft Subtree ──
echo -e "\n${BLUE}[3/6] Pruefe Weft ...${NC}"
if [ ! -d "weft/Cargo.toml" ]; then
    echo -e "${YELLOW}Weft nicht gefunden. Klone als Git Subtree ...${NC}"
    git subtree add --prefix=weft \
        https://github.com/WeaveMindAI/weft.git main --squash
    echo -e "${GREEN}✓ Weft eingebunden${NC}"
else
    echo -e "${GREEN}✓ Weft bereits vorhanden${NC}"
fi

# ── Chunker ──
echo -e "\n${BLUE}[4/6] Pruefe big-pdf-data-chunker ...${NC}"
if [ ! -f "chunker/Dockerfile" ]; then
    echo -e "${YELLOW}Chunker nicht gefunden. Klone ...${NC}"
    git clone https://github.com/GunnarMUC/big-pdf-data-chunker.git chunker
    echo -e "${GREEN}✓ Chunker geklont${NC}"
else
    echo -e "${GREEN}✓ Chunker bereits vorhanden${NC}"
fi

# ── vLLM Modell (optional) ──
echo -e "\n${BLUE}[5/6] Pruefe LLM-Verfuegbarkeit ...${NC}"
if curl -s "http://localhost:${VLLM_PORT:-8000}/v1/models" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ vLLM laeuft auf Port ${VLLM_PORT:-8000}${NC}"
elif curl -s "http://localhost:11434/api/tags" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Ollama laeuft auf Port 11434${NC}"
else
    echo -e "${YELLOW}Kein LLM-Service erreichbar. Bitte vLLM oder Ollama starten.${NC}"
    echo -e "  vLLM: docker compose --profile gpu up -d vllm"
    echo -e "  Ollama: ollama serve"
fi

# ── Docker Compose Build ──
echo -e "\n${BLUE}[6/6] Docker Images bauen ...${NC}"
docker compose build --parallel

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}   Setup abgeschlossen${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e ""
echo -e "Starte alle Services:"
echo -e "  ${YELLOW}./scripts/start.sh${NC}"
echo -e ""
echo -e "Oder einzelne Services:"
echo -e "  docker compose up -d postgres redis restate"
echo -e "  docker compose --profile gpu up -d vllm"
echo -e "  ./scripts/start.sh chunker"
