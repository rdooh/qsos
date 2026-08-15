mod adr;
mod baseline;
mod dsl;
mod feature;
mod lifecycle;
mod staged;
mod sync;

use qsos_core::{LintReport, ProjectLayout, Violation};

pub use adr::audit_adrs;
pub use baseline::{apply_baseline, AuditBaseline};
pub use dsl::audit_dsl;
pub use feature::audit_features;
pub use lifecycle::audit_lifecycle;
pub use staged::lint_staged;
pub use sync::audit_sync;

pub fn lint_project(layout: &ProjectLayout) -> LintReport {
    let mut violations = Vec::new();
    violations.extend(audit_adrs(layout));
    violations.extend(audit_features(layout));
    violations.extend(audit_lifecycle(layout));
    violations.extend(audit_dsl(layout));
    LintReport::new(violations)
}

pub fn lint_project_sync(layout: &ProjectLayout) -> LintReport {
    let mut violations = lint_project(layout).violations;
    violations.extend(audit_sync(layout));
    LintReport::new(violations)
}

pub fn lint_project_with_baseline(layout: &ProjectLayout) -> LintReport {
    let baseline = AuditBaseline::load(&layout.root);
    apply_baseline(lint_project(layout), baseline.as_ref())
}

pub fn lint_staged_with_baseline(layout: &ProjectLayout) -> Result<LintReport, String> {
    lint_staged(layout)
}

pub fn lint_file(layout: &ProjectLayout, target: &std::path::Path) -> LintReport {
    let rel = layout.rel_path(target);
    let mut violations = Vec::new();

    if target.extension().and_then(|e| e.to_str()) == Some("md")
        && rel.starts_with("docs/decisions/")
    {
        violations.extend(adr::audit_adr_file(layout, target));
    } else if target.extension().and_then(|e| e.to_str()) == Some("feature") {
        violations.extend(feature::audit_feature_file(layout, target));
    } else {
        violations.push(Violation::error(
            rel,
            "unsupported-file",
            "qsos lint --file supports docs/decisions/*.md and docs/features/*.feature",
        ));
    }

    LintReport::new(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn lint_qsos_repo_has_known_notes_only() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("qsos root");
        let layout = ProjectLayout::discover(&root);
        let report = lint_project(&layout);
        let errors: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.severity == qsos_core::Severity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "unexpected lint errors: {:?}",
            errors
        );
    }
}
