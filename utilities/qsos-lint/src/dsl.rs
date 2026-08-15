use qsos_core::{ProjectLayout, Severity, Violation};
use regex::Regex;
use std::fs;

pub fn audit_dsl(layout: &ProjectLayout) -> Vec<Violation> {
    let dsl_path = layout.architecture_dir.join("architecture.dsl");
    if !dsl_path.is_file() {
        return vec![Violation::note(
            "docs/architecture/architecture.dsl",
            "dsl-skipped",
            "architecture.dsl missing — skipping DSL coverage check",
        )];
    }

    let dsl = match fs::read_to_string(&dsl_path) {
        Ok(c) => c,
        Err(e) => {
            return vec![Violation::error(
                layout.rel_path(&dsl_path),
                "dsl-read-error",
                e.to_string(),
            )];
        }
    };

    let rel_dsl = layout.rel_path(&dsl_path);
    let accepted = load_accepted_adr_bodies(layout);
    let mut violations = Vec::new();

    for caps in Regex::new(r"(\w+)\s*=\s*(softwareSystem|container|component)\s+")
        .unwrap()
        .captures_iter(&dsl)
    {
        let name = &caps[1];
        let referenced = accepted.iter().any(|(_, body)| body.contains(name));
        if !referenced && !accepted.is_empty() {
            violations.push(Violation::note(
                rel_dsl.clone(),
                "dsl-unjustified",
                format!("element '{name}' not referenced in any Accepted ADR"),
            ));
        }
    }

    for (id, _body) in &accepted {
        if !dsl.contains(&format!("ADR-{id}")) {
            violations.push(Violation {
                file: rel_dsl.clone(),
                line: None,
                rule: "adr-unlinked".into(),
                description: format!("Accepted ADR-{id} not referenced in architecture.dsl"),
                severity: Severity::Note,
            });
        }
    }

    if dsl.contains("Target") {
        let target_re = Regex::new(r#"tags\s*"[^"]*Target"#).unwrap();
        if target_re.is_match(&dsl) {
            for (id, body) in &accepted {
                if body.contains("Target") {
                    continue;
                }
                let _ = id;
            }
            if !accepted.iter().any(|(_, b)| b.contains("Target")) {
                violations.push(Violation::error(
                    rel_dsl,
                    "target-no-adr",
                    "DSL has Target-tagged elements but no Accepted ADR documents Target work",
                ));
            }
        }
    }

    violations
}

fn load_accepted_adr_bodies(layout: &ProjectLayout) -> Vec<(String, String)> {
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

    #[test]
    fn skips_when_no_dsl() {
        let layout = ProjectLayout::discover("/nonexistent");
        let v = audit_dsl(&layout);
        assert!(v.iter().any(|x| x.rule == "dsl-skipped"));
    }
}
