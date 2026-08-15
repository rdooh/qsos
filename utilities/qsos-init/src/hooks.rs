use serde::Serialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub const HOOK_MARKER: &str = "qsos-pre-commit-hook v1";

#[derive(Debug, Clone, Serialize, Default)]
pub struct HookReport {
    pub hook_installed: bool,
    pub hook_path: String,
    pub baseline_written: bool,
}

pub fn hook_path(root: &Path) -> std::path::PathBuf {
    root.join(".git/hooks/pre-commit")
}

pub fn is_git_repo(root: &Path) -> bool {
    root.join(".git").is_dir()
}

pub fn hook_installed(root: &Path) -> bool {
    let path = hook_path(root);
    fs::read_to_string(&path)
        .map(|c| c.contains(HOOK_MARKER))
        .unwrap_or(false)
}

pub fn install_hooks(root: &Path, qsos_bin: &str) -> Result<HookReport, String> {
    if !is_git_repo(root) {
        return Err("not a git repository — run git init first".into());
    }

    let path = hook_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create hooks dir: {e}"))?;
    }

    let script = hook_script(qsos_bin);
    fs::write(&path, script).map_err(|e| format!("write pre-commit hook: {e}"))?;
    let mut perms = fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).map_err(|e| format!("chmod hook: {e}"))?;

    Ok(HookReport {
        hook_installed: true,
        hook_path: path.to_string_lossy().to_string(),
        baseline_written: false,
    })
}

pub fn write_baseline(root: &Path) -> Result<(), String> {
    use qsos_core::ProjectLayout;
    use qsos_lint::{AuditBaseline, lint_project};

    let layout = ProjectLayout::discover(root);
    let report = lint_project(&layout);
    let baseline = AuditBaseline::from_report(&report);
    AuditBaseline::save(root, &baseline)
}

pub fn install_hooks_with_baseline(root: &Path, qsos_bin: &str) -> Result<HookReport, String> {
    write_baseline(root)?;
    let mut report = install_hooks(root, qsos_bin)?;
    report.baseline_written = true;
    Ok(report)
}

pub fn check_hooks(root: &Path) -> HookReport {
    HookReport {
        hook_installed: hook_installed(root),
        hook_path: hook_path(root).to_string_lossy().to_string(),
        baseline_written: root.join(".audit-baseline.json").is_file(),
    }
}

fn hook_script(qsos_bin: &str) -> String {
    format!(
        r#"#!/bin/sh
# {HOOK_MARKER}
set -e
ROOT="$(git rev-parse --show-toplevel)"
"{qsos_bin}" lint --staged --root "$ROOT"
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn init_git(root: &Path) {
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .unwrap();
    }

    #[test]
    fn install_and_detect_hook() {
        let dir = tempfile::tempdir().unwrap();
        init_git(dir.path());
        let report = install_hooks(dir.path(), "qsos").unwrap();
        assert!(report.hook_installed);
        assert!(hook_installed(dir.path()));
        let content = fs::read_to_string(hook_path(dir.path())).unwrap();
        assert!(content.contains(HOOK_MARKER));
        assert!(content.contains("lint --staged"));
    }

    #[test]
    fn check_reports_missing_hook() {
        let dir = tempfile::tempdir().unwrap();
        init_git(dir.path());
        let report = check_hooks(dir.path());
        assert!(!report.hook_installed);
    }
}
