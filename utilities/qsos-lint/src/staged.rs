use crate::baseline::{apply_baseline, AuditBaseline};
use qsos_core::{LintReport, ProjectLayout};
use std::path::Path;
use std::process::Command;

pub fn lint_staged(layout: &ProjectLayout) -> Result<LintReport, String> {
    let staged = git_staged_files(&layout.root)?;
    let baseline = AuditBaseline::load(&layout.root);

    if staged.is_empty() {
        return Ok(LintReport::new(Vec::new()));
    }

    let mut violations = Vec::new();
    for rel in staged {
        let path = layout.root.join(&rel);
        if !path.is_file() {
            continue;
        }
        if rel.starts_with("docs/decisions/") && rel.ends_with(".md") {
            violations.extend(crate::adr::audit_adr_file(layout, &path));
        } else if rel.ends_with(".feature") && rel.starts_with("docs/features/") {
            violations.extend(crate::feature::audit_feature_file(layout, &path));
        }
    }

    Ok(apply_baseline(LintReport::new(violations), baseline.as_ref()))
}

fn git_staged_files(root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args([
            "-C",
            root.to_str().ok_or("invalid project root")?,
            "diff",
            "--cached",
            "--name-only",
            "--diff-filter=ACMR",
        ])
        .output()
        .map_err(|e| format!("git not available: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.replace('\\', "/"))
        .filter(|l| !l.is_empty())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::ProjectLayout;
    use std::fs;
    use std::process::Command;

    fn init_git_repo(root: &Path) {
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(root)
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(root)
            .status()
            .unwrap();
    }

    #[test]
    fn lint_staged_only_checks_indexed_files() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        fs::create_dir_all(dir.path().join("docs/features")).unwrap();
        fs::write(
            dir.path().join("docs/features/bad.feature"),
            "---\nfeature: Bad\nstatus: @proposed\n---\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("docs/features/good.feature"),
            "---\nfeature: Good\nstatus: @accepted\n---\n\n# Good\n\n**Scenario: Works**\n  Given x\n  When y\n  Then z\n",
        )
        .unwrap();

        Command::new("git")
            .args(["add", "docs/features/bad.feature"])
            .current_dir(dir.path())
            .status()
            .unwrap();

        let layout = ProjectLayout::discover(dir.path());
        let report = lint_staged(&layout).unwrap();
        assert!(
            report
                .violations
                .iter()
                .all(|v| v.file.contains("bad.feature"))
        );
    }
}
