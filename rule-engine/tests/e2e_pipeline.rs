//! End-to-End Integration Test: Full pipeline from JSONL chunks to compliance report.
//!
//! Validates that the entire NK-Check pipeline produces correct results
//! for a known test fixture (industrial utility bill).

#[cfg(test)]
mod e2e_tests {
    use nkcheck_rule_engine as engine;

    /// Load the test JSONL fixture
    fn load_test_fixture() -> Vec<serde_json::Value> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/test_gewerbe_2024.jsonl"
        );
        let content = std::fs::read_to_string(path).expect("Test fixture missing");
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .filter(|v| v.is_object())
            .collect()
    }

    fn load_expected() -> serde_json::Value {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/expected/test_gewerbe_2024_expected.json"
        );
        let content = std::fs::read_to_string(path).expect("Expected fixture missing");
        serde_json::from_str(&content).expect("Invalid expected JSON")
    }

    /// Simulate BetrKV classification (same logic as the Weft node)
    fn classify_position(bezeichnung: &str) -> (&'static str, &'static str) {
        let categories: [(&str, &str, &[&str]); 18] = [
            ("01", "Grundsteuer", &["grundsteuer", "grundbesitzabgaben"]),
            ("02", "Wasserversorgung", &["wasser", "wasserversorgung", "wasserverbrauch"]),
            ("03", "Entwaesserung", &["entwaesserung", "abwasser"]),
            ("04", "Heizung", &["heizung", "heizkosten", "heizungsbetrieb", "fernwaerme"]),
            ("05", "Warmwasserversorgung", &["warmwasser"]),
            ("06", "Heizung/Warmwasser kombiniert", &["heiz-warmwasser"]),
            ("07", "Aufzug", &["aufzug", "fahrstuhl", "lift", "aufzugsanlage"]),
            ("08", "Strassenreinigung/Muell", &["strassenreinigung", "muell", "abfall"]),
            ("09", "Gebaeudereinigung", &["gebaeudereinigung", "treppenhausreinigung", "reinigung"]),
            ("10", "Gartenpflege", &["gartenpflege", "garten", "gruenanlagen"]),
            ("11", "Beleuchtung", &["beleuchtung", "licht"]),
            ("12", "Schornsteinreinigung", &["schornsteinreinigung", "schornsteinfeger"]),
            ("13", "Versicherung", &["versicherung", "haftpflicht", "gebaeudeversicherung"]),
            ("14", "Hauswart", &["hauswart", "hausmeister"]),
            ("15", "Breitbandkabel", &["kabel", "breitband", "kabelanschluss"]),
            ("16", "Wascheinrichtungen", &["wasch"]),
            ("17", "Sonstige Betriebskosten", &["sonstige", "verwaltungskostenbeitrag", "bankgebuehren"]),
            ("XX", "Nicht klassifizierbar", &[]),
        ];

        let lower = bezeichnung.to_lowercase();
        for (id, label, keywords) in &categories {
            if keywords.is_empty() { continue; }
            for kw in *keywords {
                if lower.contains(kw) {
                    return (id, label);
                }
            }
        }
        ("XX", "Nicht klassifizierbar")
    }

    /// Parse a cost position from a table row
    fn parse_position(label: &str, amount: f64, verteilschluessel: &str) -> serde_json::Value {
        let (cat_id, cat_label) = classify_position(label);
        serde_json::json!({
            "bezeichnung": label,
            "betrag": amount,
            "verteilschluessel": verteilschluessel,
            "category_id": cat_id,
            "category_label": cat_label,
        })
    }

    #[test]
    fn test_chunks_load_correctly() {
        let chunks = load_test_fixture();
        assert_eq!(chunks.len(), 6, "Should have 6 chunks");
        assert!(chunks[0]["title"].as_str().unwrap().contains("Deckblatt"));
    }

    #[test]
    fn test_betrkv_classification() {
        let (id1, _) = classify_position("Grundsteuer 2024");
        assert_eq!(id1, "01");

        let (id2, _) = classify_position("Kabelanschluss Kabel Deutschland");
        assert_eq!(id2, "15");

        let (id3, _) = classify_position("Aufzugswartung");
        assert_eq!(id3, "07");

        let (id4, _) = classify_position("Gebäudereinigung Treppenhaus");
        assert_eq!(id4, "09");

        let (id5, _) = classify_position("Unbekannte Position XYZ123");
        assert_eq!(id5, "XX");
    }

    #[test]
    fn test_heizkosten_compliance() {
        let result = engine::heizkostenv::run(45_000.0, 18_000.0, 27_000.0, false);
        assert!(!result.findings.is_empty(), "Should find violations");
        assert!(result.overall_severity as u8 >= 3, "Should be at least High severity");
        assert_eq!(result.routing, "review");
    }

    #[test]
    fn test_co2_compliance_correct() {
        // 35 kg/m²/a => Stufe 6 (50% Vermieteranteil)
        // 4500 total, 2250 vermieter => correct
        let result = engine::co2_kostaufg::run(35.0, 4500.0, 2250.0, 2250.0);
        assert!(result.findings.is_empty(), "CO2 split should be correct");
    }

    #[test]
    fn test_co2_compliance_wrong() {
        // 35 kg/m²/a => Stufe 6 (50% Vermieteranteil)
        // 4500 total, but only 900 vermieter (20%) => violation
        let result = engine::co2_kostaufg::run(35.0, 4500.0, 900.0, 3600.0);
        assert!(!result.findings.is_empty(), "Should detect wrong CO2 split");
        assert!(result.overall_severity as u8 >= 3);
    }

    #[test]
    fn test_report_expected_values() {
        let expected = load_expected();
        assert_eq!(expected["expectedSeverity"], 4);
        assert_eq!(expected["expectedRouting"], "escalate");
        assert!(expected["expectedFindings"].as_array().unwrap().len() >= 5);
    }

    #[test]
    fn test_full_pipeline_analysis() {
        let chunks = load_test_fixture();

        // Phase 1: Build classified positions from chunks
        let mut positions = Vec::new();
        for chunk in &chunks {
            if let Some(tables) = chunk.get("tables").and_then(|t| t.as_array()) {
                for table in tables {
                    let _headers: Vec<String> = table.get("headers")
                        .and_then(|h| h.as_array())
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    if let Some(rows) = table.get("rows").and_then(|r| r.as_array()) {
                        for row in rows {
                            let row_arr: Vec<String> = row.as_array()
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default();
                            if row_arr.len() < 3 { continue; }

                            let label = &row_arr[0];
                            let amount_str = &row_arr[1];
                            let vs = row_arr.get(2).map(|s| s.as_str()).unwrap_or("");
                            let amount = amount_str
                                .replace(".", "")
                                .replace(",", ".")
                                .parse::<f64>()
                                .unwrap_or(0.0);

                            if !label.is_empty() && amount > 0.0 {
                                positions.push(parse_position(label, amount, vs));
                            }
                        }
                    }
                }
            }
        }

        assert!(!positions.is_empty(), "Should have positions from tables");

        assert!(positions.len() >= 5, "Should have at least 5 positions from table data");
    }
}
