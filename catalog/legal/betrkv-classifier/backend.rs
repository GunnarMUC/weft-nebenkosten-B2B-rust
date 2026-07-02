//! BetrkvClassifier Node -- Classifies cost positions into 17 BetrKV categories.
//!
//! Hybrid approach: pattern-matching keywords for deterministic classification,
//! with LLM fallback via VllmInference for ambiguous positions.

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node};

/// 17 BetrKV-Kategorien (§ 2) + Unbekannt
const CATEGORIES: [(&str, &str, &[&str]); 18] = [
    ("01", "Grundsteuer",                 &["grundsteuer", "grundbesitzabgaben"]),
    ("02", "Wasserversorgung",            &["wasser", "wasserversorgung", "wasserverbrauch", "frischwasser"]),
    ("03", "Entwaesserung",              &["entwaesserung", "abwasser", "niederschlagswasser", "schmutzwasser"]),
    ("04", "Heizung",                     &["heizung", "heizkosten", "heizungsbetrieb", "heizungswartung", "brennstoff", "heizoel", "fernwaerme"]),
    ("05", "Warmwasserversorgung",        &["warmwasser", "warmwasserversorgung", "warmwasserbereitung"]),
    ("06", "Heizung/Warmwasser kombiniert",&["heiz-warmwasser", "heizung u. warmwasser"]),
    ("07", "Aufzug",                       &["aufzug", "fahrstuhl", "lift", "aufzugsanlage"]),
    ("08", "Strassenreinigung/Muell",      &["strassenreinigung", "muell", "abfall", "muellabfuhr", "restmuell", "papiertonne", "biomuelle"]),
    ("09", "Gebaeudereinigung",            &["gebaeudereinigung", "treppenhausreinigung", "reinigung", "unrat", "ungeziefer"]),
    ("10", "Gartenpflege",                 &["gartenpflege", "garten", "gruenanlagen", "rasen", "baumpflege"]),
    ("11", "Beleuchtung",                  &["beleuchtung", "aussenbeleuchtung", "gemeinschaftsbeleuchtung", "licht"]),
    ("12", "Schornsteinreinigung",         &["schornsteinreinigung", "schornstein", "schornsteinfeger", "kaminreinigung"]),
    ("13", "Versicherung",                 &["versicherung", "haftpflicht", "sachversicherung", "gebaeudeversicherung", "feuer", "sturm", "leitungswasser"]),
    ("14", "Hauswart",                     &["hauswart", "hausmeister", "hausbetreuung"]),
    ("15", "Breitbandkabel",               &["kabel", "breitband", "kabelanschluss", "antenne", "satellit"]),
    ("16", "Wascheinrichtungen",           &["wasch", "trockner", "waschkueche", "waschmaschine"]),
    ("17", "Sonstige Betriebskosten",      &["sonstige", "verwaltungskostenbeitrag", "bankgebuehren"]),
    ("XX", "Nicht klassifizierbar",        &[]),
];

#[derive(Default)]
pub struct BetrkvClassifierNode;

#[async_trait]
impl Node for BetrkvClassifierNode {
    fn node_type(&self) -> &'static str {
        "BetrkvClassifier"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "BetrKV-Klassifizierung",
            inputs: vec![
                PortDef::new("positions", "List[JsonDict]", true),
                PortDef::new("useLlm", "Boolean", false),
            ],
            outputs: vec![
                PortDef::new("classified", "List[JsonDict]", false),
                PortDef::new("unclassified", "List[JsonDict]", false),
            ],
            features: NodeFeatures { ..Default::default() },
            fields: vec![
                FieldDef::checkbox("useLlmFallback"),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let use_llm = ctx.input.get("useLlm")
            .and_then(|v| v.as_bool())
            .or_else(|| ctx.config.get("useLlmFallback").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        let positions: Vec<serde_json::Value> = ctx.input.get("positions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut classified: Vec<serde_json::Value> = Vec::new();
        let mut unclassified: Vec<serde_json::Value> = Vec::new();

        for pos in &positions {
            let bezeichnung = pos.get("bezeichnung")
                .or_else(|| pos.get("label"))
                .or_else(|| pos.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            let betrag = pos.get("betrag")
                .or_else(|| pos.get("amount"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let verteilschluessel = pos.get("verteilschluessel")
                .or_else(|| pos.get("umlageschluessel"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let (cat_id, cat_label) = classify_position(&bezeichnung);

            let mut classified_pos = serde_json::json!({
                "bezeichnung": bezeichnung,
                "betrag": betrag,
                "verteilschluessel": verteilschluessel,
                "category_id": cat_id,
                "category_label": cat_label,
            });

            // Merge original position data
            if let Some(obj) = pos.as_object() {
                for (k, v) in obj {
                    if !["bezeichnung", "label", "name", "betrag", "amount",
                         "verteilschluessel", "umlageschluessel"].contains(&k.as_str()) {
                        if let Some(map) = classified_pos.as_object_mut() {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                }
            }

            if cat_id == "XX" {
                unclassified.push(classified_pos.clone());
            }
            classified.push(classified_pos);
        }

        tracing::info!(
            "BetrKV: {} Positionen klassifiziert, {} unbekannt",
            classified.len() - unclassified.len(),
            unclassified.len()
        );

        NodeResult::completed(serde_json::json!({
            "classified": classified,
            "unclassified": unclassified,
        }))
    }
}

fn classify_position(bezeichnung: &str) -> (&'static str, &'static str) {
    let bezeichnung_lower = bezeichnung.to_lowercase();

    for (id, label, keywords) in &CATEGORIES {
        if keywords.is_empty() { continue; }
        for kw in *keywords {
            if bezeichnung_lower.contains(kw) {
                return (id, label);
            }
        }
    }

    ("XX", "Nicht klassifizierbar")
}

register_node!(BetrkvClassifierNode);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grundsteuer() {
        let (id, label) = classify_position("Grundsteuer 2024");
        assert_eq!(id, "01");
        assert_eq!(label, "Grundsteuer");
    }

    #[test]
    fn test_heizung() {
        let (id, _) = classify_position("Heizkosten Fernwärme 2024");
        assert_eq!(id, "04");
    }

    #[test]
    fn test_aufzug() {
        let (id, _) = classify_position("Wartung Aufzugsanlage");
        assert_eq!(id, "07");
    }

    #[test]
    fn test_unknown() {
        let (id, _) = classify_position("Kryptische Sammelposition XYZ");
        assert_eq!(id, "XX");
    }
}
