//! NK-Check Rule Engine
//!
//! Deterministic compliance checks for German utility bill regulations.
//! No LLM dependency -- all checks are rule-based and verifiable.
//!
//! Modules:
//! - `betrkv`: SS 2 BetrKV category classification
//! - `heizkostenv`: SS 7 HeizkostenV distribution checks
//! - `co2_kostaufg`: CO2 cost splitting per Stufenmodell
//! - `gewerbe`: Commercial lease specific checks
//! - `fristen`: Filing deadlines (SS 556 BGB)
//! - `plausibilitaet`: Statistical plausibility checks

pub mod betrkv;
pub mod heizkostenv;
pub mod co2_kostaufg;
pub mod gewerbe;
pub mod fristen;
pub mod plausibilitaet;

use serde::{Deserialize, Serialize};

/// Severity of a finding (1 = low, 5 = critical)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    VeryHigh = 4,
    Critical = 5,
}

/// An individual finding from a compliance check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Check identifier (e.g. "CO2_01", "FR_02")
    pub check_id: String,
    /// Human-readable description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Legal reference (e.g. "SS 7 HeizkostenV")
    pub legal_ref: Option<String>,
    /// Affected position / line item
    pub affected_position: Option<String>,
    /// Actual value found
    pub actual_value: Option<String>,
    /// Expected value per regulation
    pub expected_value: Option<String>,
    /// Recommended action
    pub recommendation: Option<String>,
}

/// Result of a document analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// All findings (violations and warnings)
    pub findings: Vec<Finding>,
    /// Overall severity (max of all findings)
    pub overall_severity: Severity,
    /// Suggested routing: auto, review, escalate
    pub routing: String,
    /// Summary text for display
    pub summary: String,
}

impl AnalysisResult {
    pub fn from_findings(findings: Vec<Finding>) -> Self {
        let overall_severity = findings
            .iter()
            .map(|f| f.severity)
            .max()
            .unwrap_or(Severity::Low);

        let routing = match overall_severity {
            Severity::Critical | Severity::VeryHigh => "escalate",
            Severity::High | Severity::Medium => "review",
            Severity::Low => "auto",
        };

        let summary = format!(
            "{} Findings gefunden (Schweregrad: {:?}, Routing: {})",
            findings.len(),
            overall_severity,
            routing
        );

        Self {
            findings,
            overall_severity,
            routing: routing.to_string(),
            summary,
        }
    }
}
