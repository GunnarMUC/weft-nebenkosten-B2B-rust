# NK-Check Industrie

**Automatisierte Nebenkosten-Prufung fur Industrieunternehmen -- On-Premises, DSGVO-konform, KI-gestutzt.**

NK-Check Industrie analysiert grosse, unstrukturierte Nebenkostenabrechnungen
(50-500+ Seiten), vergleicht sie mit den gesetzlichen Vorschriften
(BetrKV, HeizkostenV, BGB, CO2KostAufG) und erstellt einen strukturierten
Prufbericht mit Auffalligkeiten und Handlungsempfehlungen.

> **Abgrenzung zu [NebenkostenPro](https://nebenkostenpro.de):**
> Dieses System ist fur **Grosskunden** (Industrie, Gewerbe, Hausverwaltungen)
> ausgelegt -- nicht fur Privatkunden. Features: Mandantenfahigkeit,
> Gewerbemietvertragsprufung, Mischobjekt-Analyse, Batch-Verarbeitung.

## Kernfeatures

- **On-Premises LLM** -- Alle Daten bleiben auf eigener Hardware (vLLM + Qwen/Mixtral)
- **Stabile Regel-Engine** -- Rust-basierte, deterministische Compliance-Prufungen
- **Durable Execution** -- [Weft](https://github.com/WeaveMindAI/weft)-Framework: Workflows uberleben Crashes und Neustarts
- **Human-in-the-Loop** -- Experten-Prufung bei kritischen Auffalligkeiten
- **Multi-Tenant** -- Tenant-ID-basierte Mandantentrennung
- **Strukturierter Report** -- PDF + JSON mit allen Findings, Betragen und Normbezugen

## Architektur im Uberblick

```
PDF Upload -> big-pdf-data-chunker -> JSONL Chunks -> Weft Pipeline
                                                       |-- vLLM Analyse
                                                       |-- Regel-Engine (Rust)
                                                       |-- Human Review (SvelteKit)
                                                       +-- Report Generator
```

Detaillierte Architektur: [ARCHITECTURE.md](./ARCHITECTURE.md)

## Quickstart

```bash
git clone https://github.com/GunnarMUC/weft-nebenkosten-B2B-rust.git
cd weft-nebenkosten-B2B-rust
cp .env.example .env

./scripts/setup.sh    # Installiert alle Dependencies
./scripts/start.sh    # Startet alle Services

# -> Dashboard: http://localhost:5173
# -> Operator UI: http://localhost:5174
# -> Chunker UI:  http://localhost:5000
```

Detaillierte Installation: [SETUP.md](./SETUP.md)

## Technologie-Stack

| Layer | Technologie |
|-------|------------|
| Orchestrierung | [Weft](https://github.com/WeaveMindAI/weft) (Rust) + Restate |
| Daten-Aufbereitung | [big-pdf-data-chunker](https://github.com/GunnarMUC/big-pdf-data-chunker) (Python) |
| LLM-Inferenz | [vLLM](https://github.com/vllm-project/vllm) + Qwen 2.5 32B |
| Regel-Engine | Rust (Eigenentwicklung) |
| Human Interface | SvelteKit + Weft Extension-API |
| Datenbank | PostgreSQL |
| Queue | Redis / Restate |

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
