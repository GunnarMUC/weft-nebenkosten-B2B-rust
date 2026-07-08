# NK-Check Industrie

[![License](https://img.shields.io/badge/License-Apache%202.0-blue)](LICENSE) [![CI](https://github.com/GunnarMUC/weft-nebenkosten-B2B-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/GunnarMUC/weft-nebenkosten-B2B-rust/actions)

**Automatisierte Nebenkosten-Prufung fur Industrieunternehmen -- On-Premises, DSGVO-konform, KI-gestutzt.**

NK-Check Industrie verarbeitet grosse, unstrukturierte Nebenkostenabrechnungen
(50-500+ Seiten) in Minuten. Auf eigener Hardware, mit lokalen Open-Source-LLMs,
gepruft gegen alle relevanten Vorschriften -- BetrKV, HeizkostenV, BGB, CO2KostAufG.
Das Ergebnis: ein strukturierter Prufbericht mit normbezogenen Auffalligkeiten,
konkreten Betragen und vorbereiteten Handlungsempfehlungen.

> **Fur Grosskunden gebaut.** Mandantenfahigkeit, Gewerbemietvertragsprufung,
> Mischobjekt-Analyse, Batch-Verarbeitung -- alles auf Industriemassstab ausgelegt.
> Keine Cloud-Abhangigkeit, keine Datenabflusse, keine Kompromisse bei der
> Kontrolle uber sensible Finanzdokumente.

## Warum NK-Check Industrie

| Starken | Details |
|---------|---------|
| **Grosse Dateien** | Verarbeitet Abrechnungen mit 500+ Seiten und Dutzenden Anhangen. Der integrierte Chunker extrahiert OCR, Tabellen und handschriftliche Notizen automatisch. |
| **Industrie-Level** | Ausgelegt fur Gewerbeimmobilien, Mischobjekte, Center-Management und komplexe Umlageschlussel -- nicht fur die private 3-Zimmer-Wohnung. |
| **Lokale LLMs** | Keine Cloud-API. vLLM mit Qwen 2.5 32B und Mixtral 8x7B laufen on-premises. Alle Finanzdaten bleiben im eigenen Netzwerk. |
| **Stabiles Rust-Fundament** | Regel-Engine und Orchestrierung in Rust gebaut. Keine Laufzeitfehler durch deterministische Compliance-Checks. Typgepruft vom Compiler. |
| **Durable Execution** | Workflows uberleben Server-Neustarts. Eine dreiwochige Ruckfrage beim Vermieter ist derselbe Code wie eine dreisekundige API-Antwort. |
| **Human-in-the-Loop** | Kritische Findings gehen an einen menschlichen Prufer. Erst nach Freigabe geht der Report raus. Kein automatischer Blindflug. |

## Kernfeatures

- **On-Premises LLM** -- vLLM + Qwen 2.5 32B / Mixtral 8x7B auf eigener GPU oder CPU
- **Stabile Regel-Engine** -- Rust: 6 Prf-Module, 15+ deterministische Checks, 0 Clippy-Warnings
- **Durable Execution** -- [Weft](https://github.com/WeaveMindAI/weft)-Framework: Zustand uberlebt Crashes und Neustarts
- **Human-in-the-Loop** -- Operator-UI (SvelteKit) fur Experten-Prufung bei kritischen Auffalligkeiten
- **Multi-Tenant** -- Tenant-ID-basierte Mandantentrennung auf Datenbank-Ebene
- **Strukturierter Report** -- PDF + JSON + Markdown mit Seitenreferenzen, Normbezugen und Betragen

## Architektur im Uberblick

```
PDF Upload -> big-pdf-data-chunker -> JSONL Chunks -> Weft Pipeline
                                                       |-- vLLM Analyse (lokal, on-prem)
                                                       |-- Regel-Engine (Rust, deterministisch)
                                                       |-- Human Review (SvelteKit Operator-UI)
                                                       +-- Report Generator (PDF + JSON)
```

Detaillierte Architektur: [ARCHITECTURE.md](./ARCHITECTURE.md)

## Quickstart

```bash
git clone --recurse-submodules https://github.com/GunnarMUC/weft-nebenkosten-B2B-rust.git
cd weft-nebenkosten-B2B-rust
cp .env.example .env

./scripts/setup.sh    # Installiert alle Dependencies
./scripts/start.sh    # Startet alle Services

# -> Operator UI: http://localhost:5174
# -> Chunker UI:  http://localhost:5000
```

Detaillierte Installation: [SETUP.md](./SETUP.md)

## Technologie-Stack

| Layer | Technologie | Eigenschaft |
|-------|------------|------------|
| Orchestrierung | [Weft](https://github.com/WeaveMindAI/weft) (Rust) + Restate | Durable Execution, typgepruft |
| Daten-Aufbereitung | [big-pdf-data-chunker](https://github.com/GunnarMUC/big-pdf-data-chunker) (Python) | OCR, Tabellen, Handschrift |
| LLM-Inferenz | [vLLM](https://github.com/vllm-project/vllm) + Qwen 2.5 32B | Open-Source, on-premises |
| Regel-Engine | Rust (Eigenentwicklung) | 15+ deterministische Checks |
| Human Interface | SvelteKit + Weft Extension-API | Approve/Reject/Cancel |
| Datenbank | PostgreSQL 17 | JSONB, Mandantenfahigkeit |

## Geprufte Vorschriften

- **BetrKV SS 2** -- 17 Betriebskostenarten (Katalog)
- **HeizkostenV SS 7** -- 50/70-Verteilung
- **BGB SS 556** -- Abrechnungsfristen, Formelle Anforderungen
- **CO2KostAufG** -- 10-Stufen-Modell
- **Gewerbemietrecht** -- Vertragsklauseln, Centerkosten, Vorwegabzuge

## Lizenz & Haftung

Dieses Projekt liefert schematische Hinweise und ersetzt keine
Rechtsberatung (SS 1 RDG). Nutzung auf eigene Verantwortung.

Apache 2.0 -- siehe [LICENSE](./LICENSE).
