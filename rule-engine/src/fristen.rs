use super::{Finding, Severity, AnalysisResult};
use chrono::NaiveDate;

/// BGB Sig 556 Abs. 3: Abrechnungsfrist 12 Monate nach Ende des Abrechnungszeitraums.
pub fn check_abrechnungsfrist(
    abr_period_end: &str,     // "2024-12-31"
    abr_document_date: &str,  // Datum des Abrechnungsdokuments
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let end_date = match NaiveDate::parse_from_str(abr_period_end, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            findings.push(Finding {
                check_id: "FR_00".into(),
                description: "Abrechnungszeitraum-Ende nicht lesbar".into(),
                severity: Severity::Medium,
                legal_ref: None,
                affected_position: None,
                actual_value: Some(abr_period_end.to_string()),
                expected_value: Some("YYYY-MM-DD Format".into()),
                recommendation: Some("Abrechnungszeitraum manuell prufen".into()),
            });
            return findings;
        }
    };

    let doc_date = match NaiveDate::parse_from_str(abr_document_date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            findings.push(Finding {
                check_id: "FR_00".into(),
                description: "Abrechnungsdatum nicht lesbar".into(),
                severity: Severity::Medium,
                legal_ref: None,
                affected_position: None,
                actual_value: Some(abr_document_date.to_string()),
                expected_value: Some("YYYY-MM-DD Format".into()),
                recommendation: Some("Abrechnungsdatum manuell prufen".into()),
            });
            return findings;
        }
    };

    // Deadline: 12 Monate nach Ende des Abrechnungszeitraums
    let deadline = end_date
        .checked_add_months(chrono::Months::new(12))
        .unwrap_or(end_date);

    if doc_date > deadline {
        let days_overdue = (doc_date - deadline).num_days();
        findings.push(Finding {
            check_id: "FR_01".into(),
            description: format!(
                "Abrechnungsfrist uberschritten ({} Tage nach Frist)",
                days_overdue
            ),
            severity: Severity::VeryHigh,
            legal_ref: Some("BGB § 556 Abs. 3 Satz 3".into()),
            affected_position: Some("Abrechnungszeitraum".into()),
            actual_value: Some(format!(
                "Erstellt am {}, Frist endete am {}",
                doc_date.format("%d.%m.%Y"),
                deadline.format("%d.%m.%Y")
            )),
            expected_value: Some(format!(
                "Spatestens am {} (12 Monate nach Abrechnungszeitraum)",
                deadline.format("%d.%m.%Y")
            )),
            recommendation: Some("Verspatung prufen: Nachforderungen konnen ausgeschlossen sein (SS 556 III 3 BGB)".into()),
        });
    }

    // Abrechnungszeitraum maximal 12 Monate
    let days_in_period = (end_date - end_date
        .checked_sub_months(chrono::Months::new(12))
        .unwrap_or(end_date))
        .num_days();

    if days_in_period > 366 {
        findings.push(Finding {
            check_id: "FR_02".into(),
            description: format!(
                "Abrechnungszeitraum uberschreitet 12 Monate ({} Tage)",
                days_in_period
            ),
            severity: Severity::High,
            legal_ref: Some("BGB § 556 Abs. 3 Satz 1".into()),
            affected_position: Some("Abrechnungszeitraum".into()),
            actual_value: Some(format!("{} Tage", days_in_period)),
            expected_value: Some("Maximal 12 Monate (365/366 Tage)".into()),
            recommendation: Some("Abrechnungszeitraum muss auf max. 12 Monate begrenzt sein".into()),
        });
    }

    findings
}

pub fn run(abr_period_end: &str, abr_document_date: &str) -> AnalysisResult {
    AnalysisResult::from_findings(check_abrechnungsfrist(abr_period_end, abr_document_date))
}
