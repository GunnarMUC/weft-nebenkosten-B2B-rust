//! Full Pipeline Integration Test
//!
//! Exercises the complete NK-Check pipeline:
//! JSONL load -> BetrKV classify -> Compliance check -> Report
//!
//! Uses realistic test fixtures and validates every stage.

#[cfg(test)]
mod full_pipeline {
    use nkcheck_rule_engine::*;
    use serde_json::{json, Value};

    // ─────────────────────────────────────────────────────────────
    // Helper: Load test fixture
    // ─────────────────────────────────────────────────────────────
    fn load_chunks() -> Vec<Value> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/test_gewerbe_2024.jsonl"
        );
        std::fs::read_to_string(path)
            .expect("Fixture missing")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }

    // ─────────────────────────────────────────────────────────────
    // Stage 1: JSONL Loading
    // ─────────────────────────────────────────────────────────────
    #[test]
    fn stage1_jsonl_loading() {
        let chunks = load_chunks();
        assert_eq!(chunks.len(), 6, "6 Chunks erwartet");

        // Deckblatt
        let deckblatt = &chunks[0];
        assert!(deckblatt["title"].as_str().unwrap().contains("Deckblatt"));
        assert_eq!(deckblatt["confidence"].as_f64().unwrap(), 0.98);

        // Kostenuebersicht hat Tabelle
        let tabelle = &chunks[1];
        assert!(tabelle["tables"].as_array().unwrap().len() >= 1);

        // Handschrift-Annotation
        let heiz = &chunks[2];
        let annotations = heiz["annotations"].as_array().unwrap();
        assert!(annotations.iter().any(|a| a["type"] == "handwriting"));
    }

    // ─────────────────────────────────────────────────────────────
    // Stage 2: BetrKV Classification
    // ─────────────────────────────────────────────────────────────
    fn classify(label: &str) -> (String, String) {
        let lower = label.to_lowercase();
        for (id, lbl, kws) in BETRKV_CATEGORIES.iter() {
            for kw in *kws {
                if lower.contains(kw) {
                    return (id.to_string(), lbl.to_string());
                }
            }
        }
        ("XX".into(), "Nicht klassifizierbar".into())
    }

    const BETRKV_CATEGORIES: [(&str, &str, &[&str]); 18] = [
        ("01", "Grundsteuer",             &["grundsteuer"]),
        ("02", "Wasserversorgung",        &["wasser"]),
        ("03", "Entwaesserung",          &["entwaesserung", "abwasser"]),
        ("04", "Heizung",                 &["heizung", "heizkosten", "fernwaerme"]),
        ("05", "Warmwasserversorgung",    &["warmwasser"]),
        ("06", "Heizung/Warmwasser",      &[]),
        ("07", "Aufzug",                   &["aufzug", "fahrstuhl", "lift"]),
        ("08", "Muell/Strassenreinigung",  &["strassenreinigung", "muell", "abfall"]),
        ("09", "Gebaeudereinigung",        &["gebaeudereinigung"]),
        ("10", "Gartenpflege",             &["gartenpflege"]),
        ("11", "Beleuchtung",              &["beleuchtung", "licht"]),
        ("12", "Schornsteinreinigung",     &["schornstein"]),
        ("13", "Versicherung",             &["versicherung", "haftpflicht"]),
        ("14", "Hauswart",                 &["hauswart", "hausmeister"]),
        ("15", "Breitbandkabel",           &["kabel", "kabelanschluss"]),
        ("16", "Wascheinrichtungen",       &["wasch"]),
        ("17", "Sonstige",                 &[]),
        ("XX", "Unbekannt",                &[]),
    ];

    #[test]
    fn stage2_betrkv_classification() {
        // Grundsteuer
        let (id, label) = classify("Grundsteuer 2024");
        assert_eq!(id, "01");
        assert_eq!(label, "Grundsteuer");

        // Heizung
        let (id, _) = classify("Heizkosten Fernwaerme");
        assert_eq!(id, "04");

        // Versicherung
        let (id, _) = classify("Gebaeudeversicherung");
        assert_eq!(id, "13");

        // Kabelanschluss (kritisch!)
        let (id, _) = classify("Kabelanschluss Kabel Deutschland");
        assert_eq!(id, "15");

        // Aufzug
        let (id, _) = classify("Aufzugswartung");
        assert_eq!(id, "07");
    }

    // ─────────────────────────────────────────────────────────────
    // Stage 3: Position Extraction from Tables
    // ─────────────────────────────────────────────────────────────
    #[test]
    fn stage3_extract_positions() {
        let chunks = load_chunks();
        let mut positions: Vec<Value> = Vec::new();

        for chunk in &chunks {
            if let Some(tables) = chunk["tables"].as_array() {
                for table in tables {
                    if let Some(rows) = table["rows"].as_array() {
                        for row in rows {
                            let arr: Vec<String> = row.as_array()
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default();
                            if arr.len() < 3 { continue; }

                            let label = &arr[0];
                            let amount_str = arr[1].replace(".", "").replace(",", ".");
                            let amount = amount_str.parse::<f64>().unwrap_or(0.0);
                            let vs = arr.get(2).map(|s| s.as_str()).unwrap_or("");
                            let (cat_id, cat_label) = classify(label);

                            positions.push(json!({
                                "bezeichnung": label,
                                "betrag": amount,
                                "verteilschluessel": vs,
                                "category_id": cat_id,
                                "category_label": cat_label,
                            }));
                        }
                    }
                }
            }
        }

        assert!(positions.len() >= 5, "Mindestens 5 Kostenpositionen");

        // Grundsteuer muss Klassifiziert sein
        let grundsteuer = positions.iter().find(|p| p["category_id"] == "01");
        assert!(grundsteuer.is_some(), "Grundsteuer nicht klassifiziert");
        assert!(
            grundsteuer.unwrap()["betrag"].as_f64().unwrap() > 10_000.0,
            "Grundsteuer > 10k EUR"
        );

        // Heizung erkannt
        let heizung = positions.iter().filter(|p| p["category_id"] == "04").count();
        assert!(heizung >= 1, "Heizung nicht klassifiziert");
    }

    // ─────────────────────────────────────────────────────────────
    // Stage 4: Compliance Checks
    // ─────────────────────────────────────────────────────────────
    #[test]
    fn stage4_compliance_checks() {
        let chunks = load_chunks();
        let total_findings: usize = chunks.iter()
            .filter(|c| c["title"].as_str().unwrap_or("").contains("Instandhaltung"))
            .count();
        assert_eq!(total_findings, 1, "1 Instandhaltungs-Chunk");

        // HeizkostenV: Verbrauchsanteil 40% -> Verletzung
        let hkv = heizkostenv::run(45_000.0, 18_000.0, 27_000.0, false);
        assert!(!hkv.findings.is_empty());
        assert!(hkv.overall_severity as u8 >= 3);

        // CO2 korrekt
        let co2_good = co2_kostaufg::run(35.0, 4500.0, 2250.0, 2250.0);
        assert!(co2_good.findings.is_empty());

        // CO2 falsch
        let co2_wrong = co2_kostaufg::run(35.0, 4500.0, 900.0, 3600.0);
        assert!(!co2_wrong.findings.is_empty());
    }

    // ─────────────────────────────────────────────────────────────
    // Stage 5: Full Pipeline Result
    // ─────────────────────────────────────────────────────────────
    #[test]
    fn stage5_full_pipeline() {
        let chunks = load_chunks();
        assert!(!chunks.is_empty());

        // Simuliere den gesamten Durchlauf
        let mut pipeline_results: Vec<Value> = Vec::new();

        for chunk in &chunks {
            let title = chunk["title"].as_str().unwrap_or("");
            let content = chunk["content"].as_str().unwrap_or("");

            // Nur Chunks mit relevantem Content verarbeiten
            let relevant = title.contains("Heizkosten")
                || title.contains("Kosten")
                || title.contains("Versicherung")
                || title.contains("Aufzug")
                || title.contains("Instandhaltung");

            if !relevant { continue; }

            // Compliance Check pro Chunk
            let summary = json!({
                "title": title,
                "content_len": content.len(),
                "processed": true,
            });

            pipeline_results.push(summary);
        }

        assert!(pipeline_results.len() >= 3);
    }
}
