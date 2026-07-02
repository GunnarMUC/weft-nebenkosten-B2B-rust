#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="/Users/gunnar/weft-nebenkosten-B2B-rust"
PASS=0
FAIL=0
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}   NK-Check Integration Tests${NC}"
echo -e "${YELLOW}========================================${NC}"

check() {
    local name="$1"
    local cmd="$2"
    shift 2
    echo -n "  $name ... "
    if "$@" > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        FAIL=$((FAIL + 1))
    fi
}

# ── Test 1: Rule Engine ──
echo -e "\n${YELLOW}[1] Regel-Engine${NC}"
cargo build --manifest-path "$PROJECT_DIR/rule-engine/Cargo.toml" > /tmp/build-out.txt 2>&1
if [ $? -eq 0 ]; then
    echo -e "  Compile ... ${GREEN}PASS${NC}" && PASS=$((PASS+1))
else
    echo -e "  Compile ... ${RED}FAIL${NC}" && FAIL=$((FAIL+1))
fi

cargo test --manifest-path "$PROJECT_DIR/rule-engine/Cargo.toml" > /tmp/test-out.txt 2>&1
if grep -q '0 failed' /tmp/test-out.txt; then
    echo -e "  Tests ... ${GREEN}PASS${NC}" && PASS=$((PASS+1))
else
    echo -e "  Tests ... ${RED}FAIL${NC}" && FAIL=$((FAIL+1))
fi

# ── Test 2: Chunker Submodul ──
echo -e "\n${YELLOW}[2] Chunker Submodul${NC}"
test -f "$PROJECT_DIR/chunker/Dockerfile" && \
    echo -e "  Dockerfile ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  Dockerfile ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }

test -f "$PROJECT_DIR/chunker/chunker.py" && \
    echo -e "  chunker.py ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  chunker.py ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }

test -f "$PROJECT_DIR/chunker/worker.py" && \
    echo -e "  worker.py ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  worker.py ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }

# ── Test 3: Projektstruktur ──
echo -e "\n${YELLOW}[3] Projektstruktur${NC}"
for f in README.md ARCHITECTURE.md SETUP.md docker-compose.yml .env.example; do
    name="${f%.*}" && name="${name//-/ }"
    test -f "$PROJECT_DIR/$f" && \
        echo -e "  $f ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  $f ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }
done
test -f "$PROJECT_DIR/operator-ui/package.json" && \
    echo -e "  operator-ui ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  operator-ui ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }
test -x "$PROJECT_DIR/scripts/setup.sh" && \
    echo -e "  scripts/setup.sh ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  scripts/setup.sh ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }

# ── Test 4: Catalog Nodes ──
echo -e "\n${YELLOW}[4] Weft Catalog Nodes${NC}"
for node in vllm betrkv-classifier compliance-checker report-generator jsonl-ingestion; do
    backend=$(find "$PROJECT_DIR/catalog" -name backend.rs 2>/dev/null | grep "$node" | head -1)
    frontend=$(find "$PROJECT_DIR/catalog" -name frontend.ts 2>/dev/null | grep "$node" | head -1)
    test -n "$backend" && \
        echo -e "  $node:backend.rs ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  $node:backend.rs ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }
    test -n "$frontend" && \
        echo -e "  $node:frontend.ts ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  $node:frontend.ts ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }
done

# ── Test 5: vLLM Bridge Sidecar ──
echo -e "\n${YELLOW}[5] vLLM Bridge Sidecar${NC}"
test -f "$PROJECT_DIR/sidecars/vllm-bridge/Cargo.toml" && \
    echo -e "  Cargo.toml ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  Cargo.toml ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }
test -f "$PROJECT_DIR/sidecars/vllm-bridge/src/main.rs" && \
    echo -e "  main.rs ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  main.rs ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }
cargo build --manifest-path "$PROJECT_DIR/sidecars/vllm-bridge/Cargo.toml" 2>&1 && \
    echo -e "  Compile ... ${GREEN}PASS${NC}" && PASS=$((PASS+1)) || { echo -e "  Compile ... ${RED}FAIL${NC}"; FAIL=$((FAIL+1)); }

# ── Summary ──
echo ""
echo -e "${YELLOW}========================================${NC}"
echo -e "${GREEN}  Passed: $PASS${NC}"
if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}  Failed: $FAIL${NC}"
fi
echo -e "${YELLOW}========================================${NC}"

exit $FAIL
