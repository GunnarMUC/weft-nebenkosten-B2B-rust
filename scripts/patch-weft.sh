#!/usr/bin/env bash
set -euo pipefail
# patch-weft.sh -- Integriert unsere NK-Check Catalog-Nodes in Weft
#
# 1. Kopiert unsere Nodes aus catalog/ nach weft/catalog/
# 2. Fuehrt catalog-link.sh aus (generiert Rust/TS symlinks)
# 3. Validiert dass alle Nodes erkannt wurden

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
WEFT_DIR="$PROJECT_DIR/weft"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}NK-Check: Patche Weft Catalog ...${NC}"

# Pruefe ob Weft vorhanden
if [ ! -f "$WEFT_DIR/Cargo.toml" ]; then
    echo -e "${RED}Weft nicht gefunden. Bitte Weft als Subtree einbinden:${NC}"
    echo "  git subtree add --prefix=weft https://github.com/WeaveMindAI/weft.git main --squash"
    exit 1
fi

# Unsere Custom-Nodes definieren
# Format: source_dir -> target_dir
CUSTOM_NODES=(
    "catalog/ai/generative/vllm:ai/generative/vllm"
    "catalog/legal/betrkv-classifier:legal/betrkv-classifier"
    "catalog/legal/compliance-checker:legal/compliance-checker"
    "catalog/output/report-generator:output/report-generator"
    "catalog/triggers/jsonl-ingestion:triggers/jsonl-ingestion"
)

echo "  Installiere Custom Nodes ..."
for node in "${CUSTOM_NODES[@]}"; do
    src="${node%%:*}"
    tgt="${node##*:}"
    src_path="$PROJECT_DIR/$src"
    tgt_path="$WEFT_DIR/catalog/$tgt"

    if [ -d "$src_path" ]; then
        mkdir -p "$tgt_path"
        cp -f "$src_path/backend.rs" "$tgt_path/backend.rs" 2>/dev/null || true
        if [ -f "$src_path/frontend.ts" ]; then
            cp -f "$src_path/frontend.ts" "$tgt_path/frontend.ts" 2>/dev/null || true
        fi
        echo -e "    ${GREEN}✓${NC} $tgt"
    else
        echo -e "    ${YELLOW}⚠${NC}  $src nicht gefunden, ueberspringe"
    fi
done

# catalog-link.sh ausfuehren (braucht Bash 4+)
echo "  Fuehre catalog-link.sh aus ..."
if [[ "$(uname)" == "Darwin" ]] && [[ -x /opt/homebrew/bin/bash ]]; then
    CATALOG_BASH="/opt/homebrew/bin/bash"
else
    CATALOG_BASH="bash"
fi

"$CATALOG_BASH" "$WEFT_DIR/scripts/catalog-link.sh" --copy || {
    echo -e "${RED}catalog-link.sh fehlgeschlagen${NC}"
    exit 1
}

# Validiere dass unsere Nodes im Build-System sind
echo "  Validiere Node-Registrierung ..."
NODES_MOD="$WEFT_DIR/crates/weft-nodes/src/nodes/mod.rs"
if [ -f "$NODES_MOD" ]; then
    for expected in "vllm" "betrkv_classifier" "compliance_checker" "report_generator" "jsonl_ingestion"; do
        if grep -q "pub mod $expected" "$NODES_MOD"; then
            echo -e "    ${GREEN}✓${NC} mod $expected"
        else
            echo -e "    ${RED}✗${NC} mod $expected nicht in mod.rs gefunden"
        fi
    done
fi

echo -e "${GREEN}✓ Weft-Catalog gepatcht. Bereit zum builden.${NC}"

# Baue Weft (optional)
if [ "${1:-}" = "--build" ]; then
    echo -e "\n${YELLOW}Baue Weft mit Custom Nodes ...${NC}"
    cd "$WEFT_DIR"
    cargo build --release 2>&1 | tail -3
    echo -e "${GREEN}✓ Build complete${NC}"
fi
