use super::{Finding, Severity, AnalysisResult};

/// Statistische Plausibilitaets-Checks.
/// Vergleicht Kostenansaetze mit Branchen-Benchmarks und Vorjahreswerten.
///
/// Deutschland-Durchschnitt: Nebenkosten ca. 2,50-3,50 EUR/m2/Monat (2024).
/// Fuer Gewerbe tendenziell hoeher (3,00-5,00 EUR/m2/Monat).
const GEWERBE_MIN_EUR_PER_SQM: f64 = 1.50;
const GEWERBE_MAX_EUR_PER_SQM: f64 = 8.00;

/// Prueft ob die Gesamtkosten pro m2 im plausiblen Bereich liegen.
pub fn check_plausibilitaet(
    total_costs: f64,
    flaeche_qm: f64,
    abr_months: i32,
    vorjahres_kosten: Option<f64>,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if flaeche_qm <= 0.0 || abr_months <= 0 {
        return findings;
    }

    let eur_per_sqm_per_month = total_costs / flaeche_qm / abr_months as f64;

    if eur_per_sqm_per_month < GEWERBE_MIN_EUR_PER_SQM {
        findings.push(Finding {
            check_id: "PL_01".into(),
            description: format!(
                "Kosten pro m2 ungewoehnlich niedrig: {:.2} EUR/m2/Monat",
                eur_per_sqm_per_month
            ),
            severity: Severity::Low,
            legal_ref: None,
            affected_position: Some("Gesamtkosten".into()),
            actual_value: Some(format!("{:.2} EUR/m2/Monat", eur_per_sqm_per_month)),
            expected_value: Some(format!("> {:.2} EUR/m2/Monat (Gewerbe-Minimum)", GEWERBE_MIN_EUR_PER_SQM)),
            recommendation: Some("Prufen ob alle Kostenarten erfasst wurden".into()),
        });
    }

    if eur_per_sqm_per_month > GEWERBE_MAX_EUR_PER_SQM {
        findings.push(Finding {
            check_id: "PL_02".into(),
            description: format!(
                "Kosten pro m2 ungewoehnlich hoch: {:.2} EUR/m2/Monat",
                eur_per_sqm_per_month
            ),
            severity: Severity::Medium,
            legal_ref: None,
            affected_position: Some("Gesamtkosten".into()),
            actual_value: Some(format!("{:.2} EUR/m2/Monat", eur_per_sqm_per_month)),
            expected_value: Some(format!("< {:.2} EUR/m2/Monat (Gewerbe-Maximum)", GEWERBE_MAX_EUR_PER_SQM)),
            recommendation: Some("Positionen auf unzulaessige Umlagen prufen".into()),
        });
    }

    // Vorjahresvergleich (signifikanter Anstieg)
    if let Some(vorjahr) = vorjahres_kosten {
        if vorjahr > 0.0 {
            let anstieg_pct = ((total_costs - vorjahr) / vorjahr) * 100.0;
            if anstieg_pct > 50.0 {
                findings.push(Finding {
                    check_id: "PL_03".into(),
                    description: format!(
                        "Kosten stark gestiegen gegenuber Vorjahr: +{:.1}%",
                        anstieg_pct
                    ),
                    severity: Severity::Medium,
                    legal_ref: None,
                    affected_position: Some("Gesamtkosten".into()),
                    actual_value: Some(format!(
                        "{:.2} EUR (Vorjahr: {:.2} EUR, +{:.1}%)",
                        total_costs, vorjahr, anstieg_pct
                    )),
                    expected_value: Some("< 50% Anstieg".into()),
                    recommendation: Some("Signifikante Kostensteigerung prufen und Begruendung anfordern".into()),
                });
            }
        }
    }

    findings
}

pub fn run(
    total_costs: f64,
    flaeche_qm: f64,
    abr_months: i32,
    vorjahres_kosten: Option<f64>,
) -> AnalysisResult {
    AnalysisResult::from_findings(check_plausibilitaet(
        total_costs,
        flaeche_qm,
        abr_months,
        vorjahres_kosten,
    ))
}
