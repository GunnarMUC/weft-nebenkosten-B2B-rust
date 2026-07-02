# Changelog

## [0.1.0] - 2025-07-02

### Added (P0: Foundation)
- Projekt-Architektur (Dreistufig: Chunker -> Weft -> Output)
- Repository-Struktur mit catalog/, rule-engine/, operator-ui/, sidecars/
- Docker Compose fur alle Services (PostgreSQL, Redis, Chunker, Weft, vLLM, Frontends)
- Rust Regel-Engine Grundgerust (5 Module: betrkv, heizkostenv, co2_kostaufg, gewerbe, fristen, plausibilitaet)
- vLLM Bridge Sidecar (HTTP-Interface fur Weft Infrastruktur-Nodes)
- Weft Catalog Nodes definiert (Platzhalter):
  - VllmInference (mit backend.rs + frontend.ts)
  - BetrkvClassifier
  - ComplianceChecker
  - ReportGenerator
  - JsonlIngestion
- PostgreSQL Schema (6 Tabellen: tenants, document_sets, chunks, analysis_results, reports, audit_log)
- Dokumentation: README.md, ARCHITECTURE.md, SETUP.md, RECHTSRAHMEN.md
- Setup-Skript (automatische Dependency-Installation)
- Start/Stop-Skripte fur alle Services
- Rechtliche Referenz (BetrKV § 2, HeizkostenV § 7, CO2KostAufG, BGB § 556)

### Geplant fur 0.2.0
- Weft als Git Subtree einbinden
- vLLM Docker Compose Profile testen
- Chunker-Integration (Shared Volume)
- Erste End-to-End-Testpipeline
