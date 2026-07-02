# P2 Deployment Guide — Live-Pipeline

> **Ziel**: Chunker -> JSONL -> Weft (VllmInference -> BetrkvClassifier -> ComplianceChecker -> HumanQuery -> ReportGenerator) -> Pruefbericht

## Voraussetzungen

- Mac Studio / Linux Server (64+ GB RAM)
- Docker + Docker Compose v2
- NVIDIA GPU (optional, fuer vLLM GPU-Beschleunigung)
- Rust 1.85+ (falls lokal gebaut wird)

## Schritt 1: Repo + Submodule

```bash
git clone --recurse-submodules https://github.com/GunnarMUC/weft-nebenkosten-B2B-rust.git
cd weft-nebenkosten-B2B-rust
cp .env.example .env
# -> .env anpassen: LLM_HOST, LLM_MODEL_PATH, etc.
```

## Schritt 2: Weft einbinden

```bash
# Option A: Git Subtree (empfohlen)
git subtree add --prefix=weft https://github.com/WeaveMindAI/weft.git main --squash

# Option B: Manuell klonen
git clone https://github.com/WeaveMindAI/weft.git weft

# Weft-Catalog mit unseren Nodes patchen
chmod +x scripts/patch-weft.sh
./scripts/patch-weft.sh
```

## Schritt 3: LLM Engine starten

### vLLM (GPU)
```bash
docker compose --profile gpu up -d vllm
# Warte bis Modell geladen ist (kann 2-5 Min dauern):
docker compose logs -f vllm | grep -q "Uvicorn running"
```

### Ollama (CPU, Fallback)
```bash
ollama serve &
ollama pull qwen2.5:32b
ollama pull mixtral:8x7b
# API unter: http://localhost:11434
# .env: LLM_HOST=http://host.docker.internal:11434
```

## Schritt 4: Alle Services starten

```bash
# Basis-Services
./scripts/start.sh database

# Chunker
./scripts/start.sh chunker

# Backend (Restate + Weft)
docker compose up -d restate weft-orchestrator weft-api weft-node-runner

# Operator UI
./scripts/start.sh frontend

# Status
./scripts/start.sh status
```

## Schritt 5: Test-PDF verarbeiten

```bash
# PDF in den Chunker-Upload-Ordner kopieren
cp tests/fixtures/test_gewerbe_2024.pdf data/uploads/

# Oder via Chunker UI hochladen:
# -> http://localhost:5000
# -> Test-PDF per Drag & Drop

# JSONL-Chunks erscheinen in data/chunks/
ls data/chunks/
```

## Schritt 6: Pipeline ausloesen

```bash
# Weft-Projekt anlegen (via Dashboard oder API)
curl -X POST http://localhost:3000/api/projects \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test-Pruefung 2024",
    "weftCode": "...",   # Siehe catalog/test-project.weft
    "tenantId": "00000000-0000-0000-0000-000000000001"
  }'

# Projekt starten
curl -X POST http://localhost:3000/api/projects/{id}/execute

# Status verfolgen
curl http://localhost:3000/api/executions/{executionId}
```

## Port-Tabelle

| Service | Port | URL |
|---------|------|-----|
| Chunker UI | 5000 | http://localhost:5000 |
| vLLM API | 8000 | http://localhost:8000/v1/chat/completions |
| Ollama (alt) | 11434 | http://localhost:11434 |
| Operator UI | 5174 | http://localhost:5174 |
| Weft Dashboard | 5173 | http://localhost:5173 (mit `--profile full`) |
| Restate | 8080 | http://localhost:8080 |
| Restate Admin | 9070 | http://localhost:9070 |
| Weft Orchestrator | 9080 | http://localhost:9080 |
| Weft API | 3000 | http://localhost:3000 |
| PostgreSQL | 5432 | localhost:5432 |

## Troubleshooting

| Problem | Loesung |
|---------|---------|
| Weft Dockerfile "catalog/ not found" | `./scripts/patch-weft.sh` ausfuehren |
| vLLM "CUDA out of memory" | `VLLM_MAX_MODEL_LEN=16384` in .env setzen |
| Restate "port busy" | `docker compose down -v && docker compose up -d` |
| Operator-UI "Invalid token" | Token im Weft-Dashboard unter Einstellungen generieren |
