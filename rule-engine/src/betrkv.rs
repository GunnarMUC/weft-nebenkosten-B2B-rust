use super::{Finding, Severity, AnalysisResult};

/// BetrKV Kategorien aus SS 2
pub const BETRKV_CATEGORIES: [(&str, &str); 18] = [
    ("1", "Grundsteuer"),
    ("2", "Wasserversorgung"),
    ("3", "Entwaesserung"),
    ("4", "Heizung"),
    ("5", "Warmwasserversorgung"),
    ("6", "Heizung/Warmwasser kombiniert"),
    ("7", "Aufzug"),
    ("8", "Strassenreinigung/Muell"),
    ("9", "Gebaeudereinigung"),
    ("10", "Gartenpflege"),
    ("11", "Beleuchtung"),
    ("12", "Schornsteinreinigung"),
    ("13", "Versicherung"),
    ("14", "Hauswart"),
    ("15", "Breitbandkabel"),
    ("16", "Wascheinrichtungen"),
    ("17", "Sonstige"),
    ("UNBEKANNT", "Nicht klassifizierbar"),
];

/// Stichtag: Kabelanschluss nicht mehr umlagefaehig seit 01.07.2024
const KABEL_STICHTAG: &str = "2024-07-01";

/// Prueft eine Liste klassifizierter Positionen gegen BetrKV-Regeln.
pub fn check_positions(classified: &[ClassifiedPosition]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for pos in classified {
        // Kabelanschluss-Stichtag (Kategorie 15)
        if pos.category == "15"
            && pos.abr_period_end.as_deref().unwrap_or("1970-01-01") >= KABEL_STICHTAG {
                findings.push(Finding {
                    check_id: "BETRKV_01".into(),
                    description: "Kabelanschlusskosten nach Stichtag 01.07.2024".to_string(),
                    severity: Severity::High,
                    legal_ref: Some("TKG § 71, BetrKV § 2 Nr. 15".into()),
                    affected_position: Some(pos.bezeichnung.clone()),
                    actual_value: Some(format!("{:.2} EUR", pos.betrag)),
                    expected_value: Some("0,00 EUR (keine Umlage)".into()),
                    recommendation: Some("Kostenposition beanstanden, Rueckforderung prufen".into()),
                });
            }

        // Kategorie "UNBEKANNT"
        if pos.category == "UNBEKANNT" && pos.betrag > 0.0 {
            findings.push(Finding {
                check_id: "BETRKV_02".into(),
                description: format!("Nicht klassifizierbare Kostenposition: {}", pos.bezeichnung),
                severity: Severity::Medium,
                legal_ref: Some("BetrKV § 2".into()),
                affected_position: Some(pos.bezeichnung.clone()),
                actual_value: Some(format!("{:.2} EUR", pos.betrag)),
                expected_value: None,
                recommendation: Some("Manuelle Prufung der Umlagefahigkeit empfohlen".into()),
            });
        }

        // Sonstige Kosten (§ 2 Nr. 17) nur wenn ausdruecklich vereinbart
        if pos.category == "17" {
            findings.push(Finding {
                check_id: "BETRKV_03".into(),
                description: "Sonstige Betriebskosten (§ 2 Nr. 17) -- ".to_string(),
                severity: Severity::Low,
                legal_ref: Some("BetrKV § 2 Nr. 17".into()),
                affected_position: Some(pos.bezeichnung.clone()),
                actual_value: None,
                expected_value: None,
                recommendation: Some("Prufen ob ausdrueckliche Vereinbarung im Mietvertrag vorliegt".into()),
            });
        }
    }

    // Pruefung: Alle Kostenarten muessen dokumentiert sein
    let has_verteilschluessel = classified.iter().all(|p| !p.verteilschluessel.is_empty());
    if !has_verteilschluessel {
        findings.push(Finding {
            check_id: "BETRKV_04".into(),
            description: "Nicht alle Kostenpositionen haben einen dokumentierten Verteilerschluessel".to_string(),
            severity: Severity::Medium,
            legal_ref: Some("BGB § 556 Abs. 3".into()),
            affected_position: None,
            actual_value: Some("Fehlende Verteilerschluessel".into()),
            expected_value: Some("Jede Position benoetigt dokumentierten Verteilerschluessel".into()),
            recommendation: Some("Verteilerschluessel beim Vermieter anfordern".into()),
        });
    }

    findings
}

pub fn run(classified: &[ClassifiedPosition]) -> AnalysisResult {
    AnalysisResult::from_findings(check_positions(classified))
}

#[derive(Debug, Clone)]
pub struct ClassifiedPosition {
    pub bezeichnung: String,
    pub betrag: f64,
    pub category: String,
    pub verteilschluessel: String,
    pub abr_period_end: Option<String>,
}
