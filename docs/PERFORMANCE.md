# Performance-Analyse: Durchsatz & Parallelisierung

## Modellannahmen

| Parameter | Wert |
|-----------|------|
| Dokument-Grosse | 250 Seiten (typische Gewerbe-Abrechnung) |
| Seiten OCR (gescannt) | 50% (125 Seiten OCR, 125 searchable) |
| Chunks pro Dokument | ~125 (1 Chunk = 2 Seiten) |
| LLM-Calls pro Chunk | 2 (Extraktion + Klassifizierung) |
| Input-Tokens pro Call | ~1.500 (deutscher Nebenkosten-Text) |
| Output-Tokens pro Call | ~500 (JSON-Antwort) |
| Human-Review-Quote | 20% der Dokumente |

## Pipeline-Stufen und ihre Latenz

```
Stage 1: CHUNKER
  OCR:          125 Seiten × 3s  = 375s (6,3 min)
  Suchbare PDF: 125 Seiten × 0,5s = 63s
  Chunking + LLM:                 30s
  ─────────────────────────────────────
  Total Chunker:                  ~8 min pro 250-Seiten-Dokument
  Parallelisierbar: Ja (RQ-Worker skalierbar)

Stage 2: vLLM-INFERENZ (Haupt-Bottleneck)
  1 Chunk = 1.500 Input + 500 Output = 2.000 Tokens
  Pro Call (je nach GPU): siehe Tabelle unten
  
Stage 3: REGEL-ENGINE (Rust)
  Compliance-Checks: < 100ms pro Chunk
  Nicht der Bottleneck

Stage 4: HUMAN REVIEW
  Pro Task: 1-3 Minuten Bearbeiter-Zeit
  Async via Weft Durable Execution (kein System-Bottleneck)
```

## Durchsatz nach Hardware-Tier

### Tier 1: Mac Studio M2 Ultra (96 GB RAM, CPU-Inferenz)

| Metrik | Wert |
|--------|------|
| LLM-Engine | Ollama (Metal/CPU) |
| Tokens/Sekunde | ~8 tok/s |
| Zeit pro LLM-Call | 2.000 / 8 = **250 s** |
| Chunks parallel (OLLAMA_NUM_PARALLEL=4) | 4 |
| **Chunks/Stunde** | 4 × 3.600/250 = **~58** |
| LLM-Calls/Stunde | ~116 |
| **250-Seiten-Dokument (125 Chunks, 250 Calls)** | **~4,3 Stunden** |
| Queue-frei bis | **1 Dokument alle 4h** (~2 Dok/Tag) |

> Fazit: Fur 2-3 Abrechnungen pro Tag ausreichend. Kein Echtzeit-Betrieb.

---

### Tier 2: 1× RTX 4090 (24 GB VRAM)

| Metrik | Wert |
|--------|------|
| LLM-Engine | vLLM (continuous batching) |
| Tokens/Sekunde | ~45 tok/s |
| Zeit pro LLM-Call | 2.000 / 45 = **44 s** |
| Effektive Parallelitat (Batching) | ~3-4 concurrent |
| **Chunks/Stunde** | 3.600/44 × 3,5 = **~286** |
| LLM-Calls/Stunde | ~570 |
| **250-Seiten-Dokument (250 Calls)** | **~26 Minuten** |
| Queue-frei bis | **~2 Dokumente/Stunde** (~55 Dok/Tag) |

> Fazit: Gut fur laufenden Betrieb einer mittleren Hausverwaltung.

---

### Tier 3: 1× A100 80 GB

| Metrik | Wert |
|--------|------|
| LLM-Engine | vLLM |
| Tokens/Sekunde | ~65 tok/s |
| Zeit pro LLM-Call | 2.000 / 65 = **31 s** |
| Effektive Parallelitat | ~5-6 concurrent |
| **Chunks/Stunde** | 3.600/31 × 5,5 = **~640** |
| LLM-Calls/Stunde | ~1.280 |
| **250-Seiten-Dokument (250 Calls)** | **~12 Minuten** |
| Queue-frei bis | **~5 Dokumente/Stunde** (~130 Dok/Tag) |

> Fazit: Geeignet fur Grosskunden mit hohem Volumen. Ein Halbtag reicht fur Monatsabrechnung.

---

### Tier 4: 2× A100 80 GB (Tensor Parallel)

| Metrik | Wert |
|--------|------|
| LLM-Engine | vLLM (tensor-parallel=2) |
| Tokens/Sekunde | ~110 tok/s |
| Zeit pro LLM-Call | 2.000 / 110 = **18 s** |
| Effektive Parallelitat | ~8-10 concurrent |
| **Chunks/Stunde** | 3.600/18 × 9 = **~1.800** |
| LLM-Calls/Stunde | ~3.600 |
| **250-Seiten-Dokument (250 Calls)** | **~4 Minuten** |
| Queue-frei bis | **~14 Dokumente/Stunde** (~360 Dok/Tag) |

> Fazit: Nahezu Echtzeit. Verarbeitet selbst grosste Mandanten-Tageslast ohne Warteschlange.

---

## Woran die Queue entsteht

```
Ankunftsrate (Seiten/h) > Verarbeitungsrate (Seiten/h) → Warteschlange wachst

             Verarbeitungsrate (Seiten/h)
Tier 1:      ~115 Seiten/h
Tier 2:      ~570 Seiten/h
Tier 3:    ~1.280 Seiten/h
Tier 4:    ~3.600 Seiten/h
```

Die Formel: **Chunks/Stunde × 2 Seiten/Chunk = Seiten/Stunde**

**Praktische Regel**: Pro RTX 4090 konnen ~2 komplette Industrie-Abrechnungen
pro Stunde verarbeitet werden, ohne dass sich eine Warteschlange aufbaut.

## Bottleneck-Analyse

```
┌──────────────────────────────────────────────────────────┐
│ Pipeline-Stufe          │ Anteil an Gesamtzeit           │
├──────────────────────────────────────────────────────────┤
│ Chunker (OCR)           │ 5-10% (skalierbar)             │
│ vLLM-Inferenz           │ 85-90% ← HAUPT-BOTTLENECK      │
│ Regel-Engine (Rust)     │ <1%                            │
│ Report-Generator        │ 2-3%                           │
│ Human Review            │ Async (kein System-Bottleneck) │
└──────────────────────────────────────────────────────────┘
```

## Optimierungs-Empfehlungen

1. **RPQ-Modelle statt AWQ**: Bei neueren GPUs (H100) liefern FP8-Modelle
   bessere Qualitat bei gleicher Geschwindigkeit
2. **Prefix-Caching in vLLM**: Der System-Prompt (BetrKV-Katalog, Prufschema)
   ist fur alle Chunks identisch. vLLM cached ihn → ~30% Token-Ersparnis
3. **Chunk-Grosse optimieren**: 2 Seiten/Chunk statt 1 → halbe Anzahl LLM-Calls
4. **Klassifizierung cachen**: Der BetrkvClassifier-Node ist Pattern-Matching
   (kein LLM). Die Regel-Engine lauft deterministisch. Nur die initiale
   Extraktion braucht das LLM.
5. **Speculative Decoding**: Draft-Modell (Qwen 2.5 7B) + Target (Qwen 2.5 32B)
   → 2-3× schnellere Inferenz bei gleicher Qualitat

## Zusammenfassung

| Hardware | Dok/Stunde | Dok/Tag | Geeignet fur |
|----------|-----------|---------|-------------|
| Mac Studio 96 GB | 0,25 | 2 | Pilotbetrieb |
| 1× RTX 4090 | 2 | 55 | Mittelstand |
| 1× A100 | 5 | 130 | Grosskunden |
| 2× A100 | 14 | 360 | Enterprise |

> **Empfehlung fur Produktion**: 1× RTX 4090 oder A100 als Einstieg.
> Queue-Monitoring via Restate Dashboard. Bei >80% Auslastung aufstocken.
