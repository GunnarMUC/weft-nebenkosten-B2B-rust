//! ComplianceChecker Node -- Deterministic regulatory compliance checks.
//!
//! Embedded Rust rule engine implementing checks against:
//! - BetrKV § 2 (Kabelanschluss-Stichtag, sonstige Kosten)
//! - HeizkostenV § 7 (50/70-Verteilung)
//! - CO2KostAufG (10-Stufen-Modell)
//! - BGB § 556 (Abrechnungsfrist)

use async_trait::async_trait;
use crate::node::{Node, NodeMetadata, NodeFeatures, PortDef, ExecutionContext, FieldDef};
use crate::{NodeResult, register_node};

// ── CO2 Stufenmodell ──
const CO2_STUFEN: [(f64, f64); 10] = [
    (12.0, 0.00), (17.0, 0.10), (22.0, 0.20), (27.0, 0.30), (32.0, 0.40),
    (37.0, 0.50), (42.0, 0.60), (47.0, 0.70), (52.0, 0.80), (999.0, 0.95),
];

/// Severity levels matching the standalone rule-engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity { Low = 1, Medium = 2, High = 3, VeryHigh = 4, Critical = 5 }

#[derive(Default)]
pub struct ComplianceCheckerNode;

#[async_trait]
impl Node for ComplianceCheckerNode {
    fn node_type(&self) -> &'static str {
        "ComplianceChecker"
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            label: "Compliance-Check",
            inputs: vec![
                PortDef::new("classifiedPositions", "List[JsonDict]", true),
                PortDef::new("metadata", "JsonDict", false),
            ],
            outputs: vec![
                PortDef::new("findings", "List[JsonDict]", false),
                PortDef::new("severity", "Number", false),
                PortDef::new("routing", "String", false),
            ],
            features: NodeFeatures { ..Default::default() },
            fields: vec![
                FieldDef::checkbox("checkFristen"),
                FieldDef::checkbox("checkVerteilschluessel"),
                FieldDef::checkbox("checkCo2"),
                FieldDef::checkbox("checkKabelanschluss"),
                FieldDef::checkbox("checkPlausibilitaet"),
            ],
        }
    }

    async fn execute(&self, ctx: ExecutionContext) -> NodeResult {
        let positions: Vec<serde_json::Value> = ctx.input.get("classifiedPositions")
            .and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let metadata = ctx.input.get("metadata").cloned()
            .unwrap_or(serde_json::Value::Null);

        let mut findings: Vec<serde_json::Value> = Vec::new();

        // ── BetrKV-Checks ──
        check_kabelanschluss_stichtag(&positions, &mut findings);
        check_sonstige_kosten(&positions, &mut findings);
        check_verteilschluessel(&positions, &mut findings);

        // ── HeizkostenV-Check ──
        check_heizkosten_verteilung(&positions, &metadata, &mut findings);

        // ── CO2-Check ──
        check_co2_split(&metadata, &mut findings);

        // ── Fristen-Check ──
        check_abrechnungsfrist(&metadata, &mut findings);

        // ── Plausibilitaet ──
        check_plausibilitaet(&positions, &metadata, &mut findings);

        let overall_severity = findings.iter()
            .filter_map(|f| f.get("severity").and_then(|s| s.as_i64()))
            .max().unwrap_or(1);

        let routing = match overall_severity {
            4..=5 => "escalate",
            2..=3 => "review",
            _ => "auto",
        };

        NodeResult::completed(serde_json::json!({
            "findings": findings,
            "severity": overall_severity,
            "routing": routing,
        }))
    }
}

fn add_finding(findings: &mut Vec<serde_json::Value>, id: &str, desc: &str,
               sev: Severity, law: &str, position: Option<&str>,
               actual: Option<&str>, expected: Option<&str>, rec: Option<&str>) {
    findings.push(serde_json::json!({
        "checkId": id,
        "description": desc,
        "severity": sev as u8,
        "legalRef": law,
        "affectedPosition": position,
        "actualValue": actual,
        "expectedValue": expected,
        "recommendation": rec,
    }));
}

/// BetrKV: Kabelanschluss seit 01.07.2024 nicht mehr umlagefaehig
fn check_kabelanschluss_stichtag(positions: &[serde_json::Value], findings: &mut Vec<serde_json::Value>) {
    for pos in positions {
        let cat = pos.get("category_id").and_then(|v| v.as_str()).unwrap_or("");
        if cat == "15" {
            let label = pos.get("bezeichnung").and_then(|v| v.as_str()).unwrap_or("");
            let betrag = pos.get("betrag").and_then(|v| v.as_f64()).unwrap_or(0.0);
            add_finding(findings, "BETRKV_KABEL",
                &format!("Kabelanschlusskosten '{}' -- seit 01.07.2024 keine Umlage mehr (TKG § 71)", label),
                Severity::High, "TKG § 71 i.V.m. BetrKV § 2 Nr. 15",
                Some(label),
                Some(&format!("{:.2} EUR", betrag)),
                Some("0,00 EUR"),
                Some("Position beanstanden, Rueckforderung prufen"));
            break;
        }
    }
}

/// BetrKV: Sonstige Kosten nur bei ausdruecklicher Vereinbarung
fn check_sonstige_kosten(positions: &[serde_json::Value], findings: &mut Vec<serde_json::Value>) {
    for pos in positions {
        let cat = pos.get("category_id").and_then(|v| v.as_str()).unwrap_or("");
        if cat == "17" {
            let label = pos.get("bezeichnung").and_then(|v| v.as_str()).unwrap_or("");
            add_finding(findings, "BETRKV_SONSTIGE",
                &format!("'{}' als sonstige Betriebskosten (§ 2 Nr. 17) klassifiziert", label),
                Severity::Low, "BetrKV § 2 Nr. 17",
                Some(label), None, None,
                Some("Prufen ob ausdrueckliche Vereinbarung vorliegt"));
        }
    }
}

/// BetrKV/BGB: Verteilerschluessel muessen dokumentiert sein
fn check_verteilschluessel(positions: &[serde_json::Value], findings: &mut Vec<serde_json::Value>) {
    let missing: Vec<&str> = positions.iter()
        .filter_map(|p| {
            match p.get("verteilschluessel").and_then(|v| v.as_str()) {
                Some(v) if v.is_empty() => p.get("bezeichnung").and_then(|v| v.as_str()),
                None => p.get("bezeichnung").and_then(|v| v.as_str()),
                _ => None,
            }
        }).collect();

    if !missing.is_empty() {
        add_finding(findings, "BETRKV_VERTEILER",
            &format!("{} Position(en) ohne dokumentierten Verteilerschluessel", missing.len()),
            Severity::Medium, "BGB § 556 Abs. 3",
            None, Some("Fehlende Verteilerschluessel"), Some("Dokumentierte Verteilerschluessel"),
            Some("Verteilerschluessel beim Vermieter anfordern"));
    }
}

/// HeizkostenV: 50/70-Regel
fn check_heizkosten_verteilung(positions: &[serde_json::Value], metadata: &serde_json::Value,
                                 findings: &mut Vec<serde_json::Value>) {
    let total: f64 = positions.iter()
        .filter(|p| {
            let cat = p.get("category_id").and_then(|v| v.as_str()).unwrap_or("");
            ["04", "05", "06"].contains(&cat)
        })
        .filter_map(|p| p.get("betrag").and_then(|v| v.as_f64()))
        .sum();

    if total <= 0.0 { return; }

    let verbrauch_pct = metadata.get("heizVerbrauchAnteil")
        .and_then(|v| v.as_f64()).unwrap_or(-1.0);
    let flaeche_pct = metadata.get("heizFlaecheAnteil")
        .and_then(|v| v.as_f64()).unwrap_or(-1.0);

    if verbrauch_pct >= 0.0 {
        if verbrauch_pct < 50.0 {
            add_finding(findings, "HKV_VERTEILUNG",
                &format!("Verbrauchsanteil {:.0}% unter 50% (HeizkostenV § 7)", verbrauch_pct),
                Severity::High, "HeizkostenV § 7 Abs. 1",
                Some("Heizkostenverteilung"),
                Some(&format!("{:.0}% verbrauchsabhaengig", verbrauch_pct)),
                Some("50-70%"),
                Some("Verteilung anpassen: mindestens 50% nach Verbrauch"));
        }
        if verbrauch_pct > 70.0 {
            add_finding(findings, "HKV_VERTEILUNG_MAX",
                &format!("Verbrauchsanteil {:.0}% ueber 70%", verbrauch_pct),
                Severity::Medium, "HeizkostenV § 7 Abs. 1",
                Some("Heizkostenverteilung"),
                Some(&format!("{:.0}% verbrauchsabhaengig", verbrauch_pct)),
                Some("Maximal 70%"),
                Some("Verteilung anpassen"));
        }
    }

    if flaeche_pct >= 0.0 && (flaeche_pct < 30.0 || flaeche_pct > 50.0) {
        add_finding(findings, "HKV_FLAECHE",
            &format!("Flachenanteil {:.0}% ausserhalb 30-50%", flaeche_pct),
            Severity::Medium, "HeizkostenV § 7",
            Some("Flachenverteilung"),
            Some(&format!("{:.0}% nach Flache", flaeche_pct)),
            Some("30-50%"),
            Some("Flachenanteil prufen"));
    }
}

/// CO2KostAufG: Stufenmodell
fn check_co2_split(metadata: &serde_json::Value, findings: &mut Vec<serde_json::Value>) {
    let co2_per_sqm = match metadata.get("co2PerSqm").and_then(|v| v.as_f64()) {
        Some(v) if v > 0.0 => v,
        _ => return,
    };
    let total = match metadata.get("co2Gesamtkosten").and_then(|v| v.as_f64()) {
        Some(v) if v > 0.0 => v,
        _ => return,
    };

    let mut vermieter_anteil = 0.0;
    let mut stufe = 0;
    for (i, &(max, anteil)) in CO2_STUFEN.iter().enumerate() {
        if co2_per_sqm <= max || i == 9 {
            vermieter_anteil = anteil;
            stufe = i + 1;
            break;
        }
    }

    let actual_vermieter = metadata.get("co2VermieterAnteilEUR")
        .and_then(|v| v.as_f64()).unwrap_or(-1.0);

    if actual_vermieter >= 0.0 {
        let expected = total * vermieter_anteil;
        if (actual_vermieter - expected).abs() > 1.0 {
            add_finding(findings, "CO2_STUFE",
                &format!("CO2-Aufteilung fehlerhaft (Stufe {}/{})", stufe, CO2_STUFEN.len()),
                Severity::High,
                &format!("CO2KostAufG § 7 (Stufe {})", stufe),
                Some("CO2-Kostenverteilung"),
                Some(&format!("Vermieter: {:.2} EUR (Ist)", actual_vermieter)),
                Some(&format!("Vermieter: {:.2} EUR ({:.0}%)", expected, vermieter_anteil * 100.0)),
                Some(&format!("CO2-Aufteilung korrigieren: {}% Vermieteranteil", (vermieter_anteil * 100.0) as i32)));
        }
    }
}

/// BGB: Abrechnungsfrist (12 Monate nach Abrechnungszeitraum)
fn check_abrechnungsfrist(metadata: &serde_json::Value, findings: &mut Vec<serde_json::Value>) {
    let abr_ende = match metadata.get("abrPeriodEnd").and_then(|v| v.as_str()) {
        Some(v) => v, _ => return,
    };
    let abr_datum = match metadata.get("abrDocumentDate").and_then(|v| v.as_str()) {
        Some(v) => v, _ => return,
    };

    if abr_datum > abr_ende {
        add_finding(findings, "FR_01",
            &format!("Abrechnungsdatum {} liegt nach Ende der Abrechnungsperiode {}", abr_datum, abr_ende),
            Severity::VeryHigh, "BGB § 556 Abs. 3 Satz 3",
            Some(abr_ende),
            Some(&format!("Dokument vom {}", abr_datum)),
            Some(&format!("Spaetestens 12 Monate nach {} ", abr_ende)),
            Some("Verspaetung prufen: Nachforderungen koennen ausgeschlossen sein"));
    }
}

/// Plausibilitaet: Kosten pro m²
fn check_plausibilitaet(positions: &[serde_json::Value], metadata: &serde_json::Value,
                         findings: &mut Vec<serde_json::Value>) {
    let flaeche = match metadata.get("flaecheQm").and_then(|v| v.as_f64()) {
        Some(v) if v > 0.0 => v,
        _ => return,
    };
    let abr_months = metadata.get("abrMonths").and_then(|v| v.as_i64()).unwrap_or(12);

    let total: f64 = positions.iter()
        .filter_map(|p| p.get("betrag").and_then(|v| v.as_f64()))
        .sum();

    if total <= 0.0 || abr_months <= 0 { return; }

    let eur_per_sqm = total / flaeche / abr_months as f64;

    if eur_per_sqm > 8.0 {
        add_finding(findings, "PL_AUSREISSER",
            &format!("Kosten {:.2} EUR/m²/Monat aussergewohnlich hoch", eur_per_sqm),
            Severity::Medium, "",
            Some("Gesamtkosten"),
            Some(&format!("{:.2} EUR/m²/Monat", eur_per_sqm)),
            Some("< 8,00 EUR/m²/Monat"),
            Some("Kostenpositionen auf unzulaessige Umlagen prufen"));
    }

    if eur_per_sqm < 1.0 {
        add_finding(findings, "PL_NIEDRIG",
            &format!("Kosten {:.2} EUR/m²/Monat ungewoehnlich niedrig", eur_per_sqm),
            Severity::Low, "",
            Some("Gesamtkosten"),
            Some(&format!("{:.2} EUR/m²/Monat", eur_per_sqm)),
            Some("> 1,00 EUR/m²/Monat"),
            Some("Prufen ob alle Kostenarten erfasst wurden"));
    }
}

register_node!(ComplianceCheckerNode);
