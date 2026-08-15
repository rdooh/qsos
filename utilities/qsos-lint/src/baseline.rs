use qsos_core::{LintReport, Violation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBaseline {
    pub violations: Vec<BaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BaselineEntry {
    pub file: String,
    pub rule: String,
}

impl AuditBaseline {
    pub fn load(root: &Path) -> Option<Self> {
        let path = root.join(".audit-baseline.json");
        let raw = fs::read_to_string(path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn save(root: &Path, baseline: &Self) -> Result<(), String> {
        let path = root.join(".audit-baseline.json");
        let json = serde_json::to_string_pretty(baseline)
            .map_err(|e| format!("serialize baseline: {e}"))?;
        fs::write(path, json).map_err(|e| format!("write baseline: {e}"))
    }

    pub fn from_report(report: &LintReport) -> Self {
        let violations = report
            .violations
            .iter()
            .filter(|v| v.severity == qsos_core::Severity::Error)
            .map(|v| BaselineEntry {
                file: v.file.clone(),
                rule: v.rule.clone(),
            })
            .collect();
        Self { violations }
    }
}

pub fn apply_baseline(mut report: LintReport, baseline: Option<&AuditBaseline>) -> LintReport {
    let Some(baseline) = baseline else {
        return report;
    };
    let suppressed: HashSet<&BaselineEntry> = baseline.violations.iter().collect();
    report.violations.retain(|v| {
        !suppressed.contains(&BaselineEntry {
            file: v.file.clone(),
            rule: v.rule.clone(),
        })
    });
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::Violation;

    #[test]
    fn baseline_suppresses_matching_violations() {
        let baseline = AuditBaseline {
            violations: vec![BaselineEntry {
                file: "docs/decisions/ADR-001.md".into(),
                rule: "adr-required-section".into(),
            }],
        };
        let report = LintReport::new(vec![
            Violation::error("docs/decisions/ADR-001.md", "adr-required-section", "missing"),
            Violation::error("docs/features/x.feature", "feature-not-empty", "blank"),
        ]);
        let filtered = apply_baseline(report, Some(&baseline));
        assert_eq!(filtered.violations.len(), 1);
        assert_eq!(filtered.violations[0].rule, "feature-not-empty");
    }
}
