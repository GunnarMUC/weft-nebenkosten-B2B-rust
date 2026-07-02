# Architektur: NK-Check Industrie

## Gesamtsystem (Makro-Ebene)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     EXTERNE QUELLEN (Industriekunden)                    │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ NK-Abrech│  │ Mietvertrag  │  │ Heizkosten-  │  │ Belege/Rechnung│  │
│  │ nung PDF │  │ + Nachträge  │  │ abrechnung   │  │ (Scan/PDF)     │  │
│  │ 50-500 S.│  │ (DOCX/PDF)   │  │              │  │                │  │
│  └────┬─────┘  └──────┬───────┘  └──────┬───────┘  └───────┬────────┘  │
│       │               │                 │                  │           │
├───────┴───────────────┴─────────────────┴──────────────────┴───────────┤
│                     STUFE 1: DATEN-AUFBEREITUNG                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  big-pdf-data-chunker (Docker/Python)                            │  │
│  │  - Format-Erkennung (PDF/DOCX/XLSX/Bild)                        │  │
│  │  - OCR (Tesseract 5 + ocrmypdf, deutsches Worterbuch)           │  │
│  │  - Handschrift-Erkennung mit Confidence-Markern                  │  │
│  │  - 19+ Heuristik-Marker fur dt. Nebenkosten                     │  │
│  │  - LLM-Fallback (vLLM/Ollama) fur uneindeutige Abschnitte       │  │
│  │  -> Output: document.jsonl + document.md                         │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                              │                                           │
│                              ▼ JSONL-Stream / Shared Volume              │
├──────────────────────────────────────────────────────────────────────────┤
│                     STUFE 2: ORCHESTRIERUNG (Weft)                       │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Weft Execution Pipeline                                          │   │
│  │                                                                   │   │
│  │  ┌─────────┐  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌────────┐ │   │
│  │  │ JSONL-  │  │ Text-   │  │ vLLM     │  │ BetrKV │  │Compli- │ │   │
│  │  │ Ingestion│->│ Normali-│->│ Analyse  │->│Classi- │->│ance    │ │   │
│  │  │ (ForEach)│  │ sierung │  │ (Qwen32B)│  │fier    │  │Checker │ │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └────────┘  └────┬───┘ │   │
│  │                                                          │      │   │
│  │  ┌──────────────────────────────────────────────────────┘      │   │
│  │  │                                                              │   │
│  │  │  ┌──────────┐  ┌────────────┐  ┌──────────────────────┐    │   │
│  │  │  │ Scoring  │->│ Routing    │->│ Human Review (Opt.)  │    │   │
│  │  │  │ (1-5)    │  │ (Gate)     │  │ (SvelteKit Dashboard)│    │   │
│  │  │  └──────────┘  └────────────┘  └──────────────────────┘    │   │
│  │  │                                                              │   │
│  │  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  │  Report Generator -> PDF + JSON + Widerspruchsentwurf│   │   │
│  │  │  └──────────────────────────────────────────────────────┘   │   │
│  │  │                                                              │   │
│  │  └──────────────────────────────────────────────────────────────┘   │
│  │                                                                   │   │
│  │  Durable Execution (Restate):                                      │   │
│  │  - Workflow uberlebt Crashes/Neustarts                            │   │
│  │  - Human Review kann Stunden/Tage dauern                           │   │
│  │  - Parallele Verarbeitung mehrerer Dokumente via ForEach           │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              │                                           │
│                              ▼                                           │
├──────────────────────────────────────────────────────────────────────────┤
│                     STUFE 3: AUSGABE                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐          │
│  │ Prufbericht  │  │ Dashboard    │  │ Widerspruchsentwurf  │          │
│  │ (PDF/JSON)   │  │ (Statistiken)│  │ (vorausgefullte      │          │
│  │              │  │              │  │  Klarungsvorlage)    │          │
│  └──────────────┘  └──────────────┘  └──────────────────────┘          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Datenfluss (End-to-End)

```
Phase 1: INGESTION
  PDF/DOCX/XLSX/Scan -> big-pdf-data-chunker -> document.jsonl (strukturierte Chunks)
  Laufzeit: 30-120s pro 200-Seiten-Dokument
  Ausgabe: ~50-200 Chunks mit Metadaten (Seitenbereich, Tabellen, Handschrift-Notizen)

Phase 2: ANALYSE (Weft Pipeline)
  JSONL-Ingestion-Node -> ForEach(Chunk) -> vLLM-Inference-Node
  |
  BetrKV-Classifier-Node: Ordnet Positionen den 17 BetrKV-Kategorien zu
  |
  Contract-Check-Node: Gleicht mit Gewerbemietvertrag ab (optionaler Input)
  |
  Compliance-Checker-Node: Deterministische Rust-Regel-Engine
  |
  Scoring-Node: Aggregiert Findings -> Severity 1-5 + Handlungsempfehlung

Phase 3: COMPLIANCE (Rust Regel-Engine)
  Deterministische Prufungen (kein LLM):
  |-- Fristen-Prufung (12-Monats-Frist SS 556 BGB)
  |-- Verteilerschlussel-Prufung (50/70 SS 7 HeizkostenV)
  |-- CO2-Stufenmodell (SS 5-8 CO2KostAufG)
  |-- Kabelanschluss-Stichtag (01.07.2024)
  |-- Vorwegabzuge (Mischobjekte)
  +-- Plausibilitats-Checks (Vorjahresvergleich, Branchen-Benchmarks)

Phase 4: ENTSCHEIDUNG
  Routing-Gate:
  |-- Severity >= 4 -> Human Review (Operator-UI)
  |-- Severity 2-3 -> Automatischer Report mit Warnhinweisen
  +-- Severity 1   -> Automatischer Report ("Unauffallig")

Phase 5: AUSGABE
  Report-Generator:
  |-- PDF-Prufbericht (mit Seitenreferenzen, Normbezugen, Betragen)
  |-- JSON (maschinenlesbar fur ERP-Integration)
  |-- Widerspruchsentwurf (vorausgefullte Vorlage)
  +-- Dashboard-Visualisierung (Statistiken, Trends)
```

## Technologie-Stack

| Layer | Technologie | Begrundung |
|-------|------------|------------|
| **Orchestrierung** | Weft (Rust) + Restate | Durable Execution, Typ-Sicherheit, Graph-Visualisierung |
| **Daten-Aufbereitung** | Python (Docker), PyMuPDF, pdfplumber, Tesseract | Existierendes Projekt (big-pdf-data-chunker) |
| **LLM-Inferenz** | vLLM + Qwen 2.5 32B (AWQ) | On-Premises, DSGVO-konform, kein Cloud-Datentransfer |
| **LLM-Fallback** | Mixtral 8x7B (via Ollama) | Besseres Reasoning bei juristischen Texten |
| **Regel-Engine** | Rust (Eigenentwicklung) | Deterministisch, stabil, typgepruft, schnell |
| **Datenbank** | PostgreSQL | Persistenz, Audit-Trail, JSONB fur flexible Schemas |
| **Frontend** | SvelteKit (Weft Dashboard-Basis) | Gleicher Stack wie Weft, erweiterbar |
| **Human Interface** | SvelteKit (eigenes Operator-Dashboard) | Nutzt Weft Extension-API fur Task-Management |
| **Deployment** | Docker Compose | Einheitliche Umgebung, einfache Skalierung |

## Schnittstelle: Chunker -> Weft

Der Chunker liefert pro Dokument eine **JSONL-Datei** (ein JSON-Objekt pro logischem Abschnitt):

```json
{
  "chunk_id": "abc123_003",
  "doc_id": "abc123",
  "title": "Heizkostenabrechnung 2024",
  "level": 1,
  "pages": [4, 7],
  "content": "Die Heizkosten verteilen sich wie folgt...",
  "tables": [{"headers": ["Einheit", "Verbrauch", "Kosten"], "rows": [...]}],
  "annotations": [{"type": "handwriting", "text": "falscher Zahler", "confidence": 0.42}],
  "confidence": 0.93,
  "source_file": "abrechnung_q1_2025.pdf"
}
```

Ubergabe via **Shared Docker Volume** `/data/chunks/`. Ein Weft `JsonlWatcher`-Trigger
uberwacht das Verzeichnis und startet die Pipeline automatisch fur neue JSONL-Dateien.

## Weft Pipeline (konzeptioneller Weft-Code)

```weft
trigger = JsonlWatcher {
  directory: "/data/chunks"
  pattern: "*.jsonl"
}

pipeline = ForEach(trigger.chunks) -> (result: JsonDict) {

  preprocessor = TextNormalizer { label: "Bereinigung" }

  extractor = VllmInference {
    label: "Positionen erkennen"
    model: "qwen2.5:32b"
    parseJson: true
  }

  classifier = BetrkvClassifier {
    label: "BetrKV Kategorisierung"
  }

  compliance = ComplianceChecker {
    label: "Gesetzliche Prufung"
    checks: ["fristen", "verteilschluessel", "co2", "kabelanschluss", "vorwegabzuege"]
  }

  scorer = FindingAggregator { label: "Severity-Bewertung" }

  routing = Gate { condition: "severity >= 4" }

  review = HumanQuery {
    label: "Experten-Prufung"
    fields: [
      { fieldType: "display",        key: "chunk_title" },
      { fieldType: "approve_reject", key: "entscheidung" },
      { fieldType: "textarea",       key: "kommentar" }
    ]
  }

  report = ReportGenerator {
    label: "Prufbericht"
    format: "pdf"
  }
}
```

## Datenmodell (PostgreSQL)

```sql
-- Mandantenfahigkeit via tenant_id
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE document_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    abr_period VARCHAR(50),
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_set_id UUID REFERENCES document_sets(id),
    chunk_idx INTEGER,
    title VARCHAR(500),
    pages INT4RANGE,
    content TEXT,
    tables JSONB,
    annotations JSONB,
    confidence FLOAT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chunk_id UUID REFERENCES chunks(id),
    positions JSONB,
    betrkv_categories JSONB,
    findings JSONB,
    severity INTEGER CHECK (severity BETWEEN 1 AND 5),
    routing VARCHAR(50),
    reviewer_decision JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_set_id UUID REFERENCES document_sets(id),
    format VARCHAR(10),
    content BYTEA,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

## Neue Weft-Nodes (Katalog)

### ai/generative/vllm

- Ersetzt den OpenRouter-gebundenen `LlmInference`-Node
- Sendet HTTP-POST an vLLM OpenAI-compatible API (`/v1/chat/completions`)
- Inputs: `prompt (String)`, `systemPrompt (String?)`, `config (Dict?)`
- Outputs: `response (String)`, `parsed (JsonDict)` bei parseJson: true
- Config: `model`, `baseUrl` (default `http://vllm:8000`), `temperature`, `parseJson`

### legal/betrkv-classifier

- Klassifiziert extrahierte Kostenpositionen in die 17 BetrKV-Kategorien
- Nutzt Pattern-Matching + vLLM-Fallback
- Input: `positions (List[JsonDict])`
- Output: `classified (List[JsonDict])` mit `{position, category, confidence, norm_ref}`

### legal/compliance-checker

- Fuhrt deterministische Prfungen aus (Rust Rule Engine)
- Input: `classified_positions (List[JsonDict])`, `contract_clauses (JsonDict?)`
- Output: `findings (List[JsonDict])`, `severity (Number)`

### output/report-generator

- Erstellt PDF-Prufbericht aus Findings
- Input: `analysis_result (JsonDict)`, `findings (List[JsonDict])`
- Output: `report_pdf (Binary)`, `report_json (JsonDict)`

### triggers/jsonl-ingestion

- Uberwacht Verzeichnis auf neue JSONL-Dateien
- Liest und iteriert uber Chunks
- Input: `directory (String)`, `pattern (String)`
- Output: `chunks (List[JsonDict])`, `metadata (JsonDict)`

## Human-in-the-Loop Mechanismus

Das Operator-UI (`operator-ui/`) nutzt die Weft Extension-API:

```
GET  /api/extension/token/{token}/tasks
     -> Liste aller pending HumanQuery-Tasks

POST /api/extension/token/{token}/complete/{executionId}
     -> Sendet { nodeId, input, callbackId } zuruck an den Executor

POST /api/extension/token/{token}/cancel/{executionId}
     -> Bricht Task ab (Skip)
```

Die Weft `HumanQuery`-Node pausiert den Workflow uber Restate (Durable Execution).
Der Operator-UI-Polling-Mechanismus (5s Intervall) zeigt neue Tasks in der
Warteschlange an. Nach Entscheidung des Bearbeiters setzt der Workflow automatisch fort.

## LLM-Strategie

| Einsatz | Modell | RAM/VRAM | Begrundung |
|---------|--------|----------|------------|
| Haupt-Analyse | Qwen 2.5 32B (AWQ 4-bit) | ~20 GB VRAM | Beste deutsche Textqualitat, 32K Kontext |
| Vertragsprufung | Mixtral 8x7B (Q4) | ~45 GB RAM | Besseres Reasoning bei juristischen Texten |
| Chunker-Fallback | Qwen 2.5 14B (Q4) | ~8 GB RAM | Ausreichend fur Section-Detection |

Alle Modelle laufen on-premises. Keine Daten verlassen den Server.
vLLM wird als Haupt-Inferenz-Engine genutzt (OpenAI-compatible API).
Ollama dient als Fallback fur CPU-only Umgebungen.

## Sicherheit & DSGVO

- **On-Premises**: Keine Daten verlassen den Server
- **vLLM lauft lokal**: Keine Cloud-API-Calls
- **Tenant-ID**: Strikte Mandantentrennung auf Datenbank-Ebene
- **Datenhaltung**: Konfigurierbare Loschfristen fur Uploads und Reports
- **Audit-Log**: Alle Aktionen protokolliert (tenant_id, user, timestamp)
