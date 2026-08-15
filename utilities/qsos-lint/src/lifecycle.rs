use qsos_core::{ProjectLayout, Severity, Violation};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;

pub fn audit_lifecycle(layout: &ProjectLayout) -> Vec<Violation> {
    let manifest_path = layout.work_dir.join("tix-manifest.json");
    if !manifest_path.is_file() {
        return vec![Violation::note(
            "work/tix-manifest.json",
            "lifecycle-skipped",
            "manifest missing — skipping lifecycle cross-check",
        )];
    }

    let manifest: serde_json::Value = match fs::read_to_string(&manifest_path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                return vec![Violation::error(
                    layout.rel_path(&manifest_path),
                    "manifest-parse-error",
                    e.to_string(),
                )];
            }
        },
        Err(e) => {
            return vec![Violation::error(
                layout.rel_path(&manifest_path),
                "manifest-read-error",
                e.to_string(),
            )];
        }
    };

    let ticket_status: HashMap<String, String> = manifest
        .get("tickets")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    Some((
                        t.get("id")?.as_str()?.to_string(),
                        t.get("status")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut linked_features: HashSet<String> = HashSet::new();
    for entry in manifest
        .get("tickets")
        .and_then(|t| t.as_array())
        .into_iter()
        .flatten()
    {
        if let Some(path) = entry.get("path").and_then(|p| p.as_str()) {
            if let Ok(content) = fs::read_to_string(layout.root.join(path)) {
                for line in content.lines() {
                    if line.starts_with("features:") || line.contains("docs/features/") {
                        linked_features.extend(extract_feature_paths(line));
                    }
                }
                if let Some(fm) = content.split("---").nth(1) {
                    for line in fm.lines() {
                        if line.trim_start().starts_with("- docs/features/") {
                            linked_features.insert(
                                line.trim()
                                    .trim_start_matches("- ")
                                    .trim()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    let mut violations = Vec::new();
    if !layout.features_dir.is_dir() {
        return violations;
    }

    for entry in fs::read_dir(&layout.features_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("feature") {
            continue;
        }
        let rel = layout.rel_path(&path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let status = extract_status(&content).unwrap_or_default();
        let ticket = extract_ticket(&content);

        if let Some(ticket_id) = ticket {
            if let Some(ts) = ticket_status.get(&ticket_id) {
                if ts == "done" && status == "@proposed" {
                    violations.push(Violation::error(
                        rel.clone(),
                        "stale-tag",
                        format!("ticket {ticket_id} is done but feature is still @proposed"),
                    ));
                }
                if (ts == "in-progress" || ts == "done") && status == "@proposed" {
                    violations.push(Violation::error(
                        rel.clone(),
                        "skipped-acceptance",
                        format!("ticket {ticket_id} advanced but feature never left @proposed"),
                    ));
                }
            }
        } else if !linked_features.contains(&rel) {
            violations.push(Violation {
                file: rel,
                line: None,
                rule: "orphan-feature".into(),
                description: "feature not linked to any ticket in manifest scan".into(),
                severity: Severity::Note,
            });
        }
    }

    violations
}

fn extract_status(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    for line in rest[..end].lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == "status" {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn extract_ticket(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    for line in rest[..end].lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == "ticket" {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn extract_feature_paths(line: &str) -> Vec<String> {
    Regex::new(r"docs/features/[\w.-]+\.feature")
        .unwrap()
        .find_iter(line)
        .map(|m| m.as_str().to_string())
        .collect()
}
