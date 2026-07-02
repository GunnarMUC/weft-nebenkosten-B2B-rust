# Changelog

## [0.2.0] - 2025-07-02

### P2: Weft-Integration & Live-Pipeline

- **Weft Docker Build**: Multi-Stage Image (orchestrator, api, node-runner) mit custom Catalog-Nodes
- **patch-weft.sh**: Kopiert unsere 5 Catalog-Nodes in Weft und fuehrt catalog-link.sh aus
- **docker-compose.yml** vollstaendig ueberarbeitet (Weft via custom Dockerfile, `host.docker.internal` fuer vLLM/CPU)
- **Operator-UI API-Client**: TypeScript-Client fuer Weft Extension API (fetchTasks, completeTask, cancelTask, validateToken)
- **Operator-UI** SvelteKit-App: Login, Task-Liste mit Severity, Decision-Panel (approve/reject/cancel)
- **Full-Pipeline-Integrationstest** (5 Stages):
  - Stage 1: JSONL Chunk Loading + Validation
  - Stage 2: BetrKV Classification (5 Kategorien)
  - Stage 3: Position Extraction from Tables
  - Stage 4: Compliance Checks (HeizkostenV, CO2KostAufG)
  - Stage 5: End-to-End Pipeline Simulation
- **P2-DEPLOYMENT.md**: Schritt-fuer-Schritt Deployment-Guide
- **Tests**: 15 Rust (unit + integration + full-pipeline) + 25 Shell = alle bestanden

## [0.1.0] - 2025-07-02

### P1: Nodes & Core-Engine

- **big-pdf-data-chunker** als Git Submodul eingebunden
- **VllmInference Node**: vollstaendig implementiert (vLLM OpenAI-compatible API via reqwest)
- **BetrkvClassifier Node**: Pattern-Matching (17 BetrKV-Kategorien + 1 Unbekannt) mit 4 Unit-Tests
- **ComplianceChecker Node**: 8 deterministische Regel-Checks (Kabelanschluss, HeizkostenV, CO2-Kosten, Fristen, Plausibilitaet)
- **JsonlIngestion Node**: Trigger mit Verzeichnis-Ueberwachung, Chunk-Parsing
- **ReportGenerator Node**: JSON + Markdown Report
- **vLLM Bridge Sidecar**: Axum HTTP-Interface (`/health`, `/outputs`, `/action`)
- **Test-Fixtures**: 6 realistische JSONL-Chunks (Gewerbe-Abrechnung 2024)

### P0: Foundation

- Projekt-Architektur (Dreistufig: Chunker -> Weft -> Output)
- Repository-Struktur mit catalog/, rule-engine/, operator-ui/, sidecars/
- Docker Compose fuer alle Services (PostgreSQL, Redis, Chunker, Weft, vLLM, Frontends)
- Rust Regel-Engine Grundgeruest (6 Module: betrkv, heizkostenv, co2_kostaufg, gewerbe, fristen, plausibilitaet)
- Weft Catalog Nodes definiert (VllmInference, BetrkvClassifier, ComplianceChecker, ReportGenerator, JsonlIngestion)
- PostgreSQL Schema (6 Tabellen: tenants, document_sets, chunks, analysis_results, reports, audit_log)
- Dokumentation: README.md, ARCHITECTURE.md, SETUP.md, RECHTSRAHMEN.md
- Setup- und Start-Skripte
- Rechtliche Referenz (BetrKV § 2, HeizkostenV § 7, CO2KostAufG, BGB § 556)

### Geplant fuer 0.3.0

- Weft Docker Image bauen und im CI/CD testen
- vLLM GPU-Profil auf Ziel-Hardware verifizieren
- Erste Live-Pipeline mit echter Abrechnung
- Performance-Benchmarks (Durchsatz pro GPU-Stunde)
- Operator-UI Authentifizierung via Weft Extension Tokens
