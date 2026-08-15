use qsos_core::{EXIT_ERROR, EXIT_SUCCESS};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InitConfig {
    pub name: String,
    pub prefix: String,
    pub description: String,
    pub test_runner: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMode {
    Write,
    Check,
    DryRun,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct InitReport {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
    pub missing: Vec<String>,
    pub planned: Vec<String>,
}

impl InitReport {
    pub fn exit_code(&self, mode: InitMode) -> i32 {
        match mode {
            InitMode::Check if !self.missing.is_empty() => EXIT_ERROR,
            _ => EXIT_SUCCESS,
        }
    }
}

pub fn run_init(root: &Path, config: &InitConfig, mode: InitMode) -> Result<InitReport, String> {
    let plan = build_plan(root, config);
    let mut report = InitReport::default();

    match mode {
        InitMode::Check => {
            for entry in &plan {
                let exists = entry.path.exists();
                if !exists {
                    report.missing.push(entry.rel.clone());
                }
            }
            return Ok(report);
        }
        InitMode::DryRun => {
            for entry in &plan {
                if entry.path.exists() {
                    report.skipped.push(entry.rel.clone());
                } else {
                    report.planned.push(entry.rel.clone());
                }
            }
            return Ok(report);
        }
        InitMode::Write => {}
    }

    for entry in &plan {
        if entry.path.exists() {
            report.skipped.push(entry.rel.clone());
            continue;
        }

        if let Some(parent) = entry.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }

        if entry.is_dir {
            fs::create_dir_all(&entry.path)
                .map_err(|e| format!("mkdir {}: {e}", entry.path.display()))?;
        } else if let Some(content) = &entry.content {
            let mut file = fs::File::create(&entry.path)
                .map_err(|e| format!("create {}: {e}", entry.path.display()))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("write {}: {e}", entry.path.display()))?;
        }

        report.created.push(entry.rel.clone());
    }

    Ok(report)
}

pub fn required_paths(root: &Path, config: &InitConfig) -> BTreeSet<String> {
    build_plan(root, config)
        .into_iter()
        .map(|e| e.rel)
        .collect()
}

struct PlanEntry {
    path: PathBuf,
    rel: String,
    is_dir: bool,
    content: Option<String>,
}

fn build_plan(root: &Path, config: &InitConfig) -> Vec<PlanEntry> {
    let slug = config.name.to_lowercase().replace(' ', "-");
    let ticket_pattern = format!(
        "work/{}-NNN-slug/{}-NNN-slug.md",
        config.prefix.trim_end_matches('-'),
        config.prefix.trim_end_matches('-')
    );

    let dirs = [
        "docs",
        "docs/features",
        "docs/decisions",
        "docs/architecture",
        "docs/architecture/diagrams",
        "docs/contracts",
        "docs/statecharts",
        "docs/releases",
        "docs/standards",
        "work",
        "testing",
        "test-results",
    ];

    let mut entries: Vec<PlanEntry> = dirs
        .iter()
        .map(|d| plan_dir(root, d))
        .collect();

    entries.push(plan_file(
        root,
        "catalog-mesh.yaml",
        catalog_mesh_yaml(config, &slug, &ticket_pattern),
    ));
    entries.push(plan_file(
        root,
        "docs/README.md",
        docs_readme(&config.name),
    ));
    entries.push(plan_file(
        root,
        "docs/architecture/architecture.dsl",
        architecture_dsl(&config.name, &config.description),
    ));
    entries.push(plan_file(
        root,
        "work/README.md",
        work_readme(&config.prefix),
    ));
    entries.push(plan_file(
        root,
        "testing/README.md",
        testing_readme(),
    ));
    entries.push(plan_file(
        root,
        "work/tix-manifest.json",
        r#"{
  "tickets": []
}
"#
        .to_string(),
    ));
    entries.push(plan_file(
        root,
        "testing/manifest.json",
        testing_manifest(config.test_runner.as_deref()),
    ));
    entries.push(plan_file(
        root,
        ".gitignore",
        gitignore_content(),
    ));

    entries
}

fn plan_dir(root: &Path, rel: &str) -> PlanEntry {
    PlanEntry {
        path: root.join(rel),
        rel: rel.to_string(),
        is_dir: true,
        content: None,
    }
}

fn plan_file(root: &Path, rel: &str, content: String) -> PlanEntry {
    PlanEntry {
        path: root.join(rel),
        rel: rel.to_string(),
        is_dir: false,
        content: Some(content),
    }
}

fn catalog_mesh_yaml(config: &InitConfig, slug: &str, ticket_pattern: &str) -> String {
    format!(
        r#"apiVersion: catalog-mesh.io/v1alpha1
kind: Component
metadata:
  name: {slug}
  description: "{description}"
  prefix: "{prefix}"
  domain: ""
  tags: []
spec:
  type: tool
  lifecycle: active
  owner: ""
  ticket_manifest: work/tix-manifest.json
  ticket_pattern: "{ticket_pattern}"
  depends_on: []
"#,
        slug = slug,
        description = config.description.replace('"', "\\\""),
        prefix = config.prefix,
        ticket_pattern = ticket_pattern,
    )
}

fn architecture_dsl(name: &str, description: &str) -> String {
    format!(
        r#"workspace {{
    model {{
        system = softwareSystem "{name}" "{description}" {{
        }}
    }}
}}
"#
    )
}

fn docs_readme(name: &str) -> String {
    format!(
        r#"# docs/

Durable specification and evidence for **{name}**.

| Subdirectory | Role |
|---|---|
| `features/` | Gherkin feature files |
| `decisions/` | MADR ADRs |
| `architecture/` | Structurizr DSL and generated diagrams |
| `contracts/` | JSON Schema contracts |
| `statecharts/` | XState statecharts |
| `releases/` | Release attestations |
| `standards/` | Project-specific standards |

See [project-structure.md](standards/project-structure.md) when present.
"#
    )
}

fn work_readme(prefix: &str) -> String {
    format!(
        r#"# work/

Point-in-time work artifacts for **{prefix}** tickets.

- `tix-manifest.json` — compiled ticket registry
- `{prefix}NNN-slug/` — one folder per ticket with markdown, evidence, and logs

Ticket logs (`logs/`) are git-ignored; verification evidence is committed.
"#
    )
}

fn testing_readme() -> String {
    r#"# testing/

Test harness configuration and manifests — **not** executable test code.

Test source files live in the application source tree. Transient runner output goes to `test-results/` (git-ignored).
"#
    .to_string()
}

fn testing_manifest(test_runner: Option<&str>) -> String {
    let runner = test_runner
        .map(|r| format!("\"{r}\""))
        .unwrap_or_else(|| "null".into());
    format!(
        r#"{{
  "unit_runner": {runner},
  "e2e_runner": null,
  "utilities_runner": null,
  "coverage_threshold": null,
  "pre_commit_hook": false,
  "pre_push_hook": false,
  "decisions": []
}}
"#
    )
}

fn gitignore_content() -> String {
    r#"# QSOS transient paths
logs/
test-results/
work/*/logs/
.qsos/current-run.json
work/graph-registry.json
"#
    .to_string()
}

pub fn normalize_prefix(prefix: &str) -> Result<String, String> {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return Err("prefix must not be empty".into());
    }
    let upper = trimmed.to_uppercase();
    if !upper.ends_with('-') {
        return Ok(format!("{upper}-"));
    }
    Ok(upper)
}

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("name must not be empty".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_lint::lint_project;
    use qsos_core::ProjectLayout;
    use tempfile::tempdir;

    fn test_config() -> InitConfig {
        InitConfig {
            name: "test-poc".into(),
            prefix: "TST-".into(),
            description: "Test PoC project".into(),
            test_runner: Some("pytest".into()),
        }
    }

    #[test]
    fn init_wizard_scaffolds_and_passes_lint() {
        let dir = tempdir().unwrap();
        let config = test_config();
        let report = run_init(dir.path(), &config, InitMode::Write).unwrap();
        assert!(!report.created.is_empty());
        assert!(report.skipped.is_empty());

        assert!(dir.path().join("catalog-mesh.yaml").is_file());
        assert!(dir.path().join("work/tix-manifest.json").is_file());
        assert!(dir.path().join("docs/architecture/architecture.dsl").is_file());

        let layout = ProjectLayout::discover(dir.path());
        let lint = lint_project(&layout);
        let errors: Vec<_> = lint
            .violations
            .iter()
            .filter(|v| v.severity == qsos_core::Severity::Error)
            .collect();
        assert!(errors.is_empty(), "lint errors: {:?}", errors);
    }

    #[test]
    fn init_check_reports_gaps() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("docs/features")).unwrap();
        let config = test_config();
        let report = run_init(dir.path(), &config, InitMode::Check).unwrap();
        assert!(!report.missing.is_empty());
        assert!(report
            .missing
            .iter()
            .any(|p| p == "catalog-mesh.yaml"));
    }

    #[test]
    fn init_dry_run_plans_without_writes() {
        let dir = tempdir().unwrap();
        let config = test_config();
        let report = run_init(dir.path(), &config, InitMode::DryRun).unwrap();
        assert!(!report.planned.is_empty());
        assert!(!dir.path().join("catalog-mesh.yaml").exists());
    }

    #[test]
    fn init_is_idempotent() {
        let dir = tempdir().unwrap();
        let config = test_config();
        let first = run_init(dir.path(), &config, InitMode::Write).unwrap();
        let second = run_init(dir.path(), &config, InitMode::Write).unwrap();
        assert!(!first.created.is_empty());
        assert_eq!(second.created.len(), 0);
        assert_eq!(second.skipped.len(), first.created.len());
    }

    #[test]
    fn normalize_prefix_adds_trailing_dash() {
        assert_eq!(normalize_prefix("poc").unwrap(), "POC-");
        assert_eq!(normalize_prefix("POC-").unwrap(), "POC-");
    }
}
