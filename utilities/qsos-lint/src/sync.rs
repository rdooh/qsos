use qsos_core::{ProjectLayout, Violation};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn audit_sync(layout: &ProjectLayout) -> Vec<Violation> {
    let mut violations = Vec::new();
    violations.extend(audit_adr_links(layout));
    violations.extend(audit_dsl_justification(layout));
    violations.extend(audit_code_imports(layout));
    violations
}

fn audit_adr_links(layout: &ProjectLayout) -> Vec<Violation> {
    let dsl_path = layout.architecture_dir.join("architecture.dsl");
    if !dsl_path.is_file() {
        return vec![Violation::note(
            "docs/architecture/architecture.dsl",
            "sync-skipped",
            "architecture.dsl missing — skipping sync ADR link check",
        )];
    }

    let dsl = fs::read_to_string(&dsl_path).unwrap_or_default();
    let rel_dsl = layout.rel_path(&dsl_path);
    let mut violations = Vec::new();

    for (id, _body) in load_accepted_adrs(layout) {
        if !dsl.contains(&format!("ADR-{id}")) {
            violations.push(Violation::error(
                rel_dsl.clone(),
                "adr-unlinked",
                format!("Accepted ADR-{id} not referenced in architecture.dsl"),
            ));
        }
    }

    violations
}

fn audit_dsl_justification(layout: &ProjectLayout) -> Vec<Violation> {
    let dsl_path = layout.architecture_dir.join("architecture.dsl");
    if !dsl_path.is_file() {
        return Vec::new();
    }

    let dsl = fs::read_to_string(&dsl_path).unwrap_or_default();
    let rel_dsl = layout.rel_path(&dsl_path);
    let accepted = load_accepted_adrs(layout);
    let mut violations = Vec::new();

    if accepted.is_empty() {
        return violations;
    }

    for caps in Regex::new(r"(\w+)\s*=\s*(softwareSystem|container|component)\s+")
        .unwrap()
        .captures_iter(&dsl)
    {
        let name = &caps[1];
        let referenced = accepted.iter().any(|(_, body)| body.contains(name));
        if !referenced {
            violations.push(Violation::error(
                rel_dsl.clone(),
                "dsl-unjustified",
                format!("element '{name}' not referenced in any Accepted ADR"),
            ));
        }
    }

    violations
}

fn audit_code_imports(layout: &ProjectLayout) -> Vec<Violation> {
    let dsl_path = layout.architecture_dir.join("architecture.dsl");
    if !dsl_path.is_file() {
        return Vec::new();
    }

    let dsl = fs::read_to_string(&dsl_path).unwrap_or_default();
    let containers = parse_containers(&dsl);
    if containers.is_empty() {
        return Vec::new();
    }

    let relationships = parse_relationships(&dsl);
    let container_dirs = map_container_dirs(layout, &containers);
    let import_re =
        Regex::new(r#"(?:import\s+[^'"]+\s+from|from|require)\s*\(?\s*['"]([^'"]+)['"]"#).unwrap();

    let mut violations = Vec::new();

    for (src_id, src_dir) in &container_dirs {
        if !src_dir.is_dir() {
            continue;
        }

        let mut code_deps = HashSet::new();

        for entry in WalkDir::new(src_dir)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "js" | "ts" | "tsx" | "jsx" | "rs") {
                continue;
            }

            let content = fs::read_to_string(path).unwrap_or_default();
            for caps in import_re.captures_iter(&content) {
                let import_path = &caps[1];
                let resolved = resolve_import(path, import_path, layout.root.as_path());
                let Some(resolved) = resolved else {
                    continue;
                };

                for (target_id, target_dir) in &container_dirs {
                    if target_id == src_id {
                        continue;
                    }
                    if path_is_under(&resolved, target_dir) {
                        code_deps.insert(target_id.clone());
                    }
                }
            }
        }

        for target_id in code_deps {
            let has_relation = relationships
                .iter()
                .any(|(from, to)| from == src_id && to == &target_id);
            if !has_relation {
                violations.push(Violation::error(
                    layout.rel_path(&dsl_path),
                    "import-drift",
                    format!(
                        "container '{src_id}' imports from '{target_id}' in code but no relationship exists in architecture.dsl"
                    ),
                ));
            }
        }
    }

    violations
}

fn parse_containers(dsl: &str) -> Vec<String> {
    Regex::new(r"(\w+)\s*=\s*container\s+")
        .unwrap()
        .captures_iter(dsl)
        .map(|c| c[1].to_string())
        .collect()
}

fn parse_relationships(dsl: &str) -> Vec<(String, String)> {
    Regex::new(r"(\w+)\s*->\s*(\w+)")
        .unwrap()
        .captures_iter(dsl)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

fn map_container_dirs(layout: &ProjectLayout, containers: &[String]) -> HashMap<String, PathBuf> {
    containers
        .iter()
        .map(|id| {
            let dir = layout.root.join("src").join(id);
            (id.clone(), dir)
        })
        .collect()
}

fn resolve_import(from_file: &Path, import_path: &str, root: &Path) -> Option<PathBuf> {
    if !import_path.starts_with('.') {
        if import_path.starts_with("@/") {
            let rel = import_path.trim_start_matches("@/");
            return Some(root.join("src").join(rel));
        }
        return None;
    }

    let mut resolved = from_file.parent()?.join(import_path);
    if resolved.extension().is_none() {
        for ext in ["ts", "tsx", "js", "jsx", "rs"] {
            let with_ext = resolved.with_extension(ext);
            if with_ext.is_file() {
                resolved = with_ext;
                break;
            }
        }
    }
    Some(resolved.canonicalize().unwrap_or(resolved))
}

fn path_is_under(path: &Path, dir: &Path) -> bool {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    path.starts_with(dir)
}

fn load_accepted_adrs(layout: &ProjectLayout) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if !layout.decisions_dir.is_dir() {
        return out;
    }
    let accepted_heading_re = Regex::new(r"(?m)^Accepted\s*$").unwrap();
    for entry in fs::read_dir(&layout.decisions_dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("ADR-") || !name.ends_with(".md") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let accepted = content.contains("status: Accepted")
            || content.contains("**Status:** Accepted")
            || accepted_heading_re.is_match(&content);
        if accepted {
            if let Some(id) = name.strip_prefix("ADR-").and_then(|s| s.get(0..3)) {
                out.push((id.to_string(), content));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::ProjectLayout;
    use std::path::Path;

    fn write_sync_clean(root: &Path) {
        fs::create_dir_all(root.join("docs/decisions")).unwrap();
        fs::create_dir_all(root.join("docs/architecture")).unwrap();
        fs::create_dir_all(root.join("docs/features")).unwrap();
        fs::create_dir_all(root.join("work")).unwrap();
        fs::write(
            root.join("docs/decisions/ADR-901-clean-decision.md"),
            "# ADR-901: Clean\n\nstatus: Accepted\n\nThe core container is documented.\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/architecture/architecture.dsl"),
            "workspace { model { core = container \"Core\" \"Core service\" \"Rust\" {} } }\n// ADR-901\n",
        )
        .unwrap();
        fs::write(root.join("work/tix-manifest.json"), r#"{"tickets":[]}"#).unwrap();
    }

    fn write_sync_adr_drift(root: &Path) {
        fs::create_dir_all(root.join("docs/decisions")).unwrap();
        fs::create_dir_all(root.join("docs/architecture")).unwrap();
        fs::create_dir_all(root.join("docs/features")).unwrap();
        fs::create_dir_all(root.join("work")).unwrap();
        fs::write(
            root.join("docs/decisions/ADR-902-drift-decision.md"),
            "# ADR-902: Drift\n\nstatus: Accepted\n\nOrphan ADR not in DSL.\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/architecture/architecture.dsl"),
            "workspace { model { core = container \"Core\" \"Core\" \"Rust\" {} } }\n",
        )
        .unwrap();
        fs::write(root.join("work/tix-manifest.json"), r#"{"tickets":[]}"#).unwrap();
    }

    fn write_sync_import_drift(root: &Path) {
        fs::create_dir_all(root.join("docs/decisions")).unwrap();
        fs::create_dir_all(root.join("docs/architecture")).unwrap();
        fs::create_dir_all(root.join("src/core")).unwrap();
        fs::create_dir_all(root.join("src/api")).unwrap();
        fs::create_dir_all(root.join("work")).unwrap();
        fs::write(
            root.join("docs/decisions/ADR-903-import-decision.md"),
            "# ADR-903\n\nstatus: Accepted\n\ncore and api containers.\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/architecture/architecture.dsl"),
            "workspace { model { core = container \"Core\" \"Core\" \"TS\" {} api = container \"API\" \"API\" \"TS\" {} } }\n// ADR-903\n",
        )
        .unwrap();
        fs::write(
            root.join("src/core/app.ts"),
            "import { svc } from '../api/service';\nexport const app = svc;\n",
        )
        .unwrap();
        fs::write(root.join("src/api/service.ts"), "export const svc = 1;\n").unwrap();
        fs::write(root.join("work/tix-manifest.json"), r#"{"tickets":[]}"#).unwrap();
    }

    #[test]
    fn sync_clean_fixture_passes() {
        let dir = tempfile::tempdir().unwrap();
        write_sync_clean(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let v = audit_sync(&layout);
        let errors: Vec<_> = v
            .iter()
            .filter(|x| x.severity == qsos_core::Severity::Error)
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn sync_reports_adr_drift() {
        let dir = tempfile::tempdir().unwrap();
        write_sync_adr_drift(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let v = audit_sync(&layout);
        assert!(v.iter().any(|x| x.rule == "adr-unlinked"));
    }

    #[test]
    fn import_path_resolution_finds_api() {
        let dir = tempfile::tempdir().unwrap();
        write_sync_import_drift(dir.path());
        let core_file = dir.path().join("src/core/app.ts");
        let api_dir = dir.path().join("src/api");
        let import_re =
            Regex::new(r#"(?:import\s+[^'"]+\s+from|from|require)\s*\(?\s*['"]([^'"]+)['"]"#)
                .unwrap();
        let content = fs::read_to_string(&core_file).unwrap();
        let import_path = import_re.captures(&content).unwrap()[1].to_string();
        let resolved = resolve_import(&core_file, &import_path, dir.path()).unwrap();
        assert!(path_is_under(&resolved, &api_dir), "{resolved:?} under {api_dir:?}");
    }

    #[test]
    fn import_regex_matches_esm_import() {
        let import_re =
            Regex::new(r#"(?:import\s+[^'"]+\s+from|from|require)\s*\(?\s*['"]([^'"]+)['"]"#)
                .unwrap();
        let content = "import { svc } from '../api/service';\n";
        assert_eq!(
            import_re.captures(content).unwrap()[1].to_string(),
            "../api/service".to_string()
        );
    }

    #[test]
    fn containers_parse_from_dsl() {
        let dsl = r#"workspace { model { core = container "Core" "Core" "TS" {} api = container "API" "API" "TS" {} } }"#;
        assert_eq!(parse_containers(dsl).len(), 2);
    }

    #[test]
    fn sync_reports_import_drift() {
        let dir = tempfile::tempdir().unwrap();
        write_sync_import_drift(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let v = audit_sync(&layout);
        assert!(
            v.iter().any(|x| x.rule == "import-drift"),
            "violations: {v:?}"
        );
    }
}
