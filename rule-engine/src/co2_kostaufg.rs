use super::{Finding, Severity, AnalysisResult};

/// CO2-Kostenaufteilungsgesetz (CO2KostAufG)
/// 10-Stufen-Modell nach spezifischem CO2-Ausstoss je m2/a

const STUFEN: [(f64, f64, f64); 10] = [
    // (min, max, vermieter_anteil)
    (0.0, 12.0, 0.0),
    (12.0, 17.0, 0.10),
    (17.0, 22.0, 0.20),
    (22.0, 27.0, 0.30),
    (27.0, 32.0, 0.40),
    (32.0, 37.0, 0.50),
    (37.0, 42.0, 0.60),
    (42.0, 47.0, 0.70),
    (47.0, 52.0, 0.80),
    (52.0, f64::MAX, 0.95),
];

/// Ermittelt die CO2-Stufe basierend auf dem spezifischen CO2-Ausstoss (kg/m2/a).
pub fn get_stufe(co2_per_sqm: f64) -> (usize, f64) {
    for (i, &(_min, _max, anteil)) in STUFEN.iter().enumerate() {
        if co2_per_sqm <= _max || i == 9 {
            return (i + 1, anteil); // 1-basiert
        }
    }
    (10, 0.95)
}

/// Prueft ob die CO2-Aufteilung korrekt nach Stufenmodell erfolgt ist.
pub fn check_co2_split(
    co2_per_sqm: f64,
    total_co2_costs: f64,
    actual_landlord_share: f64,
    actual_tenant_share: f64,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let (stufe, expected_landlord_share_pct) = get_stufe(co2_per_sqm);
    let expected_landlord_amount = total_co2_costs * expected_landlord_share_pct;
    let expected_tenant_amount = total_co2_costs * (1.0 - expected_landlord_share_pct);
    let tolerance = 0.50; // 50 Cent Toleranz

    if (actual_landlord_share - expected_landlord_amount).abs() > tolerance {
        findings.push(Finding {
            check_id: "CO2_01".into(),
            description: format!(
                "CO2-Kostenaufteilung fehlerhaft (Stufe {}). ",
                stufe
            ),
            severity: Severity::High,
            legal_ref: Some(format!("CO2KostAufG § 7 (Stufe {})", stufe)),
            affected_position: Some("CO2-Kostenverteilung".into()),
            actual_value: Some(format!(
                "Vermieter: {:.2} EUR, Mieter: {:.2} EUR",
                actual_landlord_share, actual_tenant_share
            )),
            expected_value: Some(format!(
                "Vermieter: {:.2} EUR ({:.0}%), Mieter: {:.2} EUR ({:.0}%)",
                expected_landlord_amount,
                expected_landlord_share_pct * 100.0,
                expected_tenant_amount,
                (1.0 - expected_landlord_share_pct) * 100.0
            )),
            recommendation: Some(format!(
                "CO2-Aufteilung gemaess Stufe {} korrigieren ({}% Vermieteranteil)",
                stufe,
                (expected_landlord_share_pct * 100.0) as i32
            )),
        });
    }

    findings
}

pub fn run(
    co2_per_sqm: f64,
    total_co2_costs: f64,
    landlord_share: f64,
    tenant_share: f64,
) -> AnalysisResult {
    AnalysisResult::from_findings(check_co2_split(
        co2_per_sqm,
        total_co2_costs,
        landlord_share,
        tenant_share,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stufe_passivhaus() {
        let (stufe, anteil) = get_stufe(8.0);
        assert_eq!(stufe, 1);
        assert_eq!(anteil, 0.0);
    }

    #[test]
    fn test_get_stufe_schlecht() {
        let (stufe, anteil) = get_stufe(55.0);
        assert_eq!(stufe, 10);
        assert_eq!(anteil, 0.95);
    }

    #[test]
    fn test_co2_split_correct() {
        // Ein Gebaude mit 25 kg/m2/a (Stufe 4, 30% Vermieter)
        let findings = check_co2_split(25.0, 1000.0, 300.0, 700.0);
        assert!(findings.is_empty());
    }
}
