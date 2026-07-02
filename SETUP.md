# Setup: NK-Check Industrie (P0)

## Hardware-Voraussetzungen

| Komponente | Minimum | Empfohlen |
|-----------|---------|-----------|
| CPU | 8 Kerne | 16+ Kerne |
| RAM | 64 GB | 128 GB |
| GPU | NVIDIA RTX 4090 (24 GB VRAM) | NVIDIA A6000 (48 GB VRAM) |
| Storage | 500 GB NVMe SSD | 1 TB NVMe SSD |
| OS | Ubuntu 22.04+ | Ubuntu 24.04 LTS |

Fur Apple Silicon: Mac Studio M2 Ultra (96 GB unified memory) -- vLLM wird dann
durch Ollama ersetzt (siehe Schritt 3b).

## Schritt 1: System-Abhangigkeiten

### Ubuntu/Debian
```bash
sudo apt update && sudo apt install -y \
    docker.io docker-compose-v2 \
    build-essential curl git \
    pkg-config libssl-dev \
    tesseract-ocr tesseract-ocr-deu \
    poppler-utils nvidia-container-toolkit

sudo systemctl enable --now docker
sudo usermod -aG docker $USER
# -> Abmelden und neu anmelden fur Docker-Gruppe
```

### macOS (Apple Silicon)
```bash
brew install docker git openssl@3 tesseract tesseract-lang poppler
brew install --cask ollama
```

## Schritt 2: Repository klonen

```bash
git clone https://github.com/GunnarMUC/weft-nebenkosten-B2B-rust.git
cd weft-nebenkosten-B2B-rust
cp .env.example .env
# -> .env bearbeiten (LLM-Modell, Ports, etc.)
```

## Schritt 3a: vLLM installieren (NVIDIA GPU, empfohlen)

```bash
# NVIDIA Container Toolkit (falls nicht in Schritt 1)
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | \
    sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
curl -s -L https://nvidia.github.io/libnvidia-container/$distribution/libnvidia-container.list | \
    sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
    sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
sudo apt update && sudo apt install -y nvidia-container-toolkit
sudo nvidia-ctk runtime configure --runtime=docker
sudo systemctl restart docker

# Modell herunterladen und testen
docker pull vllm/vllm-openai:latest
docker run --gpus all -p 8000:8000 \
    -v ~/.cache/huggingface:/root/.cache/huggingface \
    vllm/vllm-openai:latest \
    --model Qwen/Qwen2.5-32B-Instruct-AWQ \
    --max-model-len 32768 \
    --gpu-memory-utilization 0.90
```

## Schritt 3b: Ollama installieren (ohne GPU, Fallback)

```bash
# macOS
brew install --cask ollama
ollama serve &

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Modelle laden
ollama pull qwen2.5:32b
ollama pull mixtral:8x7b
ollama pull qwen2.5:14b

# API verfugbar unter http://localhost:11434
```

## Schritt 4: big-pdf-data-chunker klonen

```bash
git clone https://github.com/GunnarMUC/big-pdf-data-chunker.git chunker
# Wird uber docker-compose gestartet
```

## Schritt 5: Weft als Git Subtree einbinden

```bash
git subtree add --prefix=weft \
    https://github.com/WeaveMindAI/weft.git main --squash
```

## Schritt 6: Docker Compose starten

```bash
docker compose up --build -d

# Status prufen
docker compose ps

# Logs verfolgen
docker compose logs -f

# Alle Services stoppen
docker compose down
```

## Schritt 7: Datenbank initialisieren

```bash
# Schema anlegen (einmalig)
docker compose exec postgres psql -U nkcheck -d nkcheck \
    -c "SELECT 1"  # Verbindung testen

# Das Schema wird automatisch beim ersten Start uber
# docker-entrypoint-initdb.d/01-schema.sql angelegt
```

## Schritt 8: Verifikation

```bash
# 1. Chunker-UI erreichbar?
curl http://localhost:5000

# 2. Weft API erreichbar?
curl http://localhost:3000/health

# 3. Weft Dashboard erreichbar?
curl http://localhost:5173

# 4. Operator-UI erreichbar?
curl http://localhost:5174

# 5. vLLM erreichbar?
curl http://localhost:8000/v1/models
```

## Schritt 9: Test-Pipeline ausfuhren

```bash
# Test-PDF kopieren (falls vorhanden)
cp tests/fixtures/test_abrechnung.pdf /data/uploads/

# Alternative: Uber Chunker-UI hochladen
# -> http://localhost:5000
# -> Drag & Drop der Test-PDF

# Erwartet:
# 1. Chunker erstellt JSONL in /data/chunks/
# 2. Weft-Pipeline startet automatisch (JsonlWatcher)
# 3. Prufbericht erscheint in /data/output/
```

## Port-Ubersicht

| Service | Port | URL |
|---------|------|-----|
| Chunker UI | 5000 | http://localhost:5000 |
| Chunker Health | 5000 | http://localhost:5000/health |
| vLLM API | 8000 | http://localhost:8000/v1/chat/completions |
| Ollama API (Fallback) | 11434 | http://localhost:11434 |
| Weft Dashboard | 5173 | http://localhost:5173 |
| Operator UI | 5174 | http://localhost:5174 |
| Restate Ingress | 8080 | http://localhost:8080 |
| Restate Admin | 9070 | http://localhost:9070 |
| Weft Orchestrator | 9080 | http://localhost:9080 |
| Weft API | 3000 | http://localhost:3000 |
| Weft Node Runner | 9082 | http://localhost:9082 |
| PostgreSQL | 5432 | localhost:5432 |
| Redis | 6379 | localhost:6379 |

## Troubleshooting

| Problem | Losung |
|---------|--------|
| vLLM "CUDA out of memory" | `--max-model-len 16384` reduzieren, oder Q4-Quantisierung (AWQ) verwenden |
| Chunker "OCR fehlgeschlagen" | `docker compose exec chunker-web tesseract --list-langs` -- `deu` muss erscheinen |
| Weft "no API key" | `.env` prufen: `DEPLOYMENT_MODE=local` muss gesetzt sein |
| Restate startet nicht | `docker compose down -v && docker compose up -d` (Volumes zurucksetzen) |
| PostgreSQL "connection refused" | `docker compose logs postgres` -- auf Fehler beim Start prufen |
| Port-Konflikte | `lsof -i :<port>` -- Prozess killen oder Port in `.env` andern |
