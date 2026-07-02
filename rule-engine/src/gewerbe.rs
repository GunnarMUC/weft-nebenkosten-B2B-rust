use super::{Finding, Severity, AnalysisResult};

/// Pruefungen fuer Gewerbemietverhaeltnisse.
/// Anders als bei Wohnraum gilt hier die Parteiautonomie --
/// der Mietvertrag bestimmt die Umlagefaehigkeit.

/// Centerkosten / Servicepauschalen muessen transparent aufgeschluesselt sein.
pub fn check_centerkosten(positionen: &[GewerbePosition]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for pos in positionen {
        let label_lower = pos.bezeichnung.to_lowercase();

        let is_sammelposition = ["centerkosten", "servicepauschale", "objektmanagement",
            "verwaltungspauschale", "nebenkostenpauschale", "betriebskostenpauschale"]
            .iter()
            .any(|kw| label_lower.contains(kw));

        if is_sammelposition && pos.detailgrad == Detailgrad::Sammelposition {
            findings.push(Finding {
                check_id: "GEW_01".into(),
                description: format!(
                    "Unklare Sammelposition: '{}' -- keine detaillierte Aufschluesselung",
                    pos.bezeichnung
                ),
                severity: Severity::High,
                legal_ref: Some("BGB § 307 (Transparenzgebot)".into()),
                affected_position: Some(pos.bezeichnung.clone()),
                actual_value: Some(format!("{:.2} EUR als Sammelposition", pos.betrag)),
                expected_value: Some("Einzeln aufgeschluesselte Positionen".into()),
                recommendation: Some("Detaillierte Aufschluesselung beim Vermieter anfordern".into()),
            });
        }
    }

    findings
}

/// Verwaltung und Instandhaltung duerfen nicht als Betriebskosten deklariert werden.
pub fn check_instandhaltung_abgrenzung(positionen: &[GewerbePosition]) -> Vec<Finding> {
    let mut findings = Vec::new();

    let inst_keywords = ["instandhaltung", "instandsetzung", "reparatur", "sanierung",
        "modernisierung", "renovierung", "erneuerung"];

    let verw_keywords = ["verwaltung", "buchhaltung", "porto", "kontofuehrung",
        "eigenleistung", "eigenaufwand"];

    for pos in positionen {
        let label_lower = pos.bezeichnung.to_lowercase();

        for kw in &inst_keywords {
            if label_lower.contains(kw) {
                findings.push(Finding {
                    check_id: "GEW_02".into(),
                    description: format!(
                        "Kostenposition '{}' klingt nach Instandhaltung, nicht Betriebskosten",
                        pos.bezeichnung
                    ),
                    severity: Severity::High,
                    legal_ref: Some("BGB § 535 (Instandhaltungspflicht Vermieter)".into()),
                    affected_position: Some(pos.bezeichnung.clone()),
                    actual_value: Some(format!("{:.2} EUR", pos.betrag)),
                    expected_value: Some("0,00 EUR (nicht umlagefaehig)".into()),
                    recommendation: Some("Position beanstanden -- Instandhaltung ist Vermietersache".into()),
                });
                break;
            }
        }

        for kw in &verw_keywords {
            if label_lower.contains(kw) {
                findings.push(Finding {
                    check_id: "GEW_03".into(),
                    description: format!(
                        "Kostenposition '{}' klingt nach Verwaltungskosten, nicht Betriebskosten",
                        pos.bezeichnung
                    ),
                    severity: Severity::Medium,
                    legal_ref: Some("BGB § 556 (keine Umlage von Verwaltungskosten)".into()),
                    affected_position: Some(pos.bezeichnung.clone()),
                    actual_value: Some(format!("{:.2} EUR", pos.betrag)),
                    expected_value: Some("0,00 EUR (nicht umlagefaehig)".into()),
                    recommendation: Some("Prufen ob ausdrueckliche Vereinbarung im Gewerbemietvertrag".into()),
                });
                break;
            }
        }
    }

    findings
}

/// Vorwegabzuege bei Mischobjekten (Wohn-/Gewerbemix).
pub fn check_mischobjekt_vorwegabzuege(
    gewerbe_flaeche: f64,
    wohn_flaeche: f64,
    nutzungs_spezifische_kosten: &[(&str, f64, &str)], // (bezeichnung, betrag, zuordnung)
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (bez, betrag, zuordnung) in nutzungs_spezifische_kosten {
        let gesamt_flaeche = gewerbe_flaeche + wohn_flaeche;
        if gesamt_flaeche <= 0.0 {
            continue;
        }

        // Wenn Kosten der Wohnnutzung zugeordnet sind, aber voll auf Gewerbe umgelegt
        if zuordnung == "wohn" {
            let gewerbe_anteil_flaeche = gewerbe_flaeche / gesamt_flaeche;
            let tolerable_anteil = *betrag * gewerbe_anteil_flaeche;

            findings.push(Finding {
                check_id: "GEW_04".into(),
                description: format!(
                    "Moeglicherweise fehlender Vorwegabzug: '{}' ist Wohnnutzungskosten",
                    bez
                ),
                severity: Severity::Medium,
                legal_ref: Some("BGB § 556a (Vorwegabzug bei Mischobjekten)".into()),
                affected_position: Some(bez.to_string()),
                actual_value: Some(format!("{:.2} EUR ohne Vorwegabzug", betrag)),
                expected_value: Some(format!("Vorwegabzug prufen (Gewerbeanteil: {:.1}%)", gewerbe_anteil_flaeche * 100.0)),
                recommendation: Some("Korrekte Trennung nach Nutzungsart anfordern".into()),
            });
        }
    }

    findings
}

pub fn run(positionen: &[GewerbePosition]) -> AnalysisResult {
    let mut all_findings = Vec::new();
    all_findings.extend(check_centerkosten(positionen));
    all_findings.extend(check_instandhaltung_abgrenzung(positionen));
    AnalysisResult::from_findings(all_findings)
}

#[derive(Debug, Clone)]
pub struct GewerbePosition {
    pub bezeichnung: String,
    pub betrag: f64,
    pub detailgrad: Detailgrad,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Detailgrad {
    Einzelposition,
    Sammelposition,
    Unbekannt,
}
