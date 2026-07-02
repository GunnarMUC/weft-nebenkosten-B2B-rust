use super::{Finding, Severity, AnalysisResult};

/// SS 7 HeizkostenV: 50/70-Regel
/// Mindestens 50%, hoechstens 70% der Heizkosten muessen verbrauchsabhaengig abgerechnet werden.
pub const MIN_VERBRAUCHSANTEIL: f64 = 0.50;
pub const MAX_VERBRAUCHSANTEIL: f64 = 0.70;

/// Prueft die Verbrauchsabhaengige Abrechnung nach HeizkostenV.
pub fn check_heizkosten(
    total_heating_costs: f64,
    consumption_based_amount: f64,
    _area_based_amount: f64,
    has_fernablesbare_zaehler: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if total_heating_costs <= 0.0 {
        return findings;
    }

    let consumption_ratio = consumption_based_amount / total_heating_costs;

    // 50/70-Regel
    if consumption_ratio < MIN_VERBRAUCHSANTEIL {
        findings.push(Finding {
            check_id: "HKV_01".into(),
            description: format!(
                "Verbrauchsanteil zu niedrig: {:.1}% (< 50%)",
                consumption_ratio * 100.0
            ),
            severity: Severity::High,
            legal_ref: Some("HeizkostenV § 7 Abs. 1".into()),
            affected_position: Some("Heizkostenverteilung".into()),
            actual_value: Some(format!("{:.1}% verbrauchsabhaengig", consumption_ratio * 100.0)),
            expected_value: Some("50-70% verbrauchsabhaengig".into()),
            recommendation: Some("Verteilung anpassen: mindestens 50% nach Verbrauch".into()),
        });
    }

    if consumption_ratio > MAX_VERBRAUCHSANTEIL {
        findings.push(Finding {
            check_id: "HKV_02".into(),
            description: format!(
                "Verbrauchsanteil zu hoch: {:.1}% (> 70%)",
                consumption_ratio * 100.0
            ),
            severity: Severity::Medium,
            legal_ref: Some("HeizkostenV § 7 Abs. 1".into()),
            affected_position: Some("Heizkostenverteilung".into()),
            actual_value: Some(format!("{:.1}% verbrauchsabhaengig", consumption_ratio * 100.0)),
            expected_value: Some("50-70% verbrauchsabhaengig".into()),
            recommendation: Some("Verteilung anpassen: hoechstens 70% nach Verbrauch".into()),
        });
    }

    // Fernablesbare Zaehler (Pflicht seit 01.12.2022)
    if !has_fernablesbare_zaehler {
        findings.push(Finding {
            check_id: "HKV_03".into(),
            description: "Keine fernablesbaren Zaehler dokumentiert (Pflicht seit 01.12.2022)".to_string(),
            severity: Severity::Medium,
            legal_ref: Some("HeizkostenV § 5 Abs. 2".into()),
            affected_position: None,
            actual_value: Some("Keine fernablesbaren Zaehler".into()),
            expected_value: Some("Fernablesbare Zaehler nach § 5 Abs. 2 HeizkostenV".into()),
            recommendation: Some("Prufen ob fernablesbare Zaehler vorhanden und korrekt ausgelesen".into()),
        });
    }

    findings
}

pub fn run(
    total_heating_costs: f64,
    consumption_amount: f64,
    area_amount: f64,
    has_fernablesbare_zaehler: bool,
) -> AnalysisResult {
    AnalysisResult::from_findings(check_heizkosten(
        total_heating_costs,
        consumption_amount,
        area_amount,
        has_fernablesbare_zaehler,
    ))
}
