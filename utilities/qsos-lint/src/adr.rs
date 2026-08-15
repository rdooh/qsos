use qsos_core::{ProjectLayout, Violation};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const ADR_FILENAME: &str = r"^ADR-(\d{3})-([a-z0-9-]+)\.md$";
const VALID_STATUSES: &[&str] = &["Proposed", "Accepted", "Superseded", "Rejected"];

pub fn audit_adrs(layout: &ProjectLayout) -> Vec<Violation> {
    let mut violations = Vec::new();
    if !layout.decisions_dir.is_dir() {
        return violations;
    }

    let re = Regex::new(ADR_FILENAME).unwrap();
    let mut entries: Vec<(u32, PathBuf)> = Vec::new();

    for entry in fs::read_dir(&layout.decisions_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "README.md" {
            continue;
        }
        let rel = layout.rel_path(&path);
        if let Some(caps) = re.captures(&name) {
            let num: u32 = caps[1].parse().unwrap_or(0);
            entries.push((num, path.clone()));
        } else if name.ends_with(".md") {
            violations.push(Violation::error(
                rel,
                "adr-naming-convention",
                "filename must match ADR-NNN-slug.md",
            ));
        }
    }

    entries.sort_by_key(|(n, _)| *n);
    check_sequence(&entries, &mut violations);

    for (_, path) in &entries {
        violations.extend(audit_adr_file(layout, path));
    }

    violations
}

pub fn audit_adr_file(layout: &ProjectLayout, path: &Path) -> Vec<Violation> {
    let rel = layout.rel_path(path);
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return vec![Violation::error(
                rel,
                "adr-read-error",
                format!("cannot read file: {e}"),
            )];
        }
    };

    let mut violations = Vec::new();
    let id = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.split('-').nth(1))
        .unwrap_or("???");

    let status = extract_field(&content, "status")
        .or_else(|| extract_bold_field(&content, "Status"))
        .or_else(|| extract_heading_value(&content, "Status"))
        .or_else(|| extract_plain_field(&content, "Status"));
    let date = extract_field(&content, "date")
        .or_else(|| extract_bold_field(&content, "Date"))
        .or_else(|| extract_plain_field(&content, "Date"));
    let makers = extract_field(&content, "decision makers")
        .or_else(|| extract_field(&content, "decision-makers"))
        .or_else(|| extract_bold_field(&content, "Decision makers"))
        .or_else(|| extract_plain_field(&content, "Decision makers"));

    if status.as_deref().unwrap_or("").is_empty() {
        violations.push(Violation::error(
            rel.clone(),
            "adr-required-metadata",
            format!("[ADR-{id}] missing Status"),
        ));
    } else if !VALID_STATUSES.iter().any(|s| status.as_deref() == Some(*s)) {
        violations.push(Violation::error(
            rel.clone(),
            "adr-valid-status",
            format!(
                "[ADR-{id}] invalid status: {}",
                status.as_deref().unwrap_or("")
            ),
        ));
    }

    if date.as_deref().unwrap_or("").is_empty() {
        violations.push(Violation::error(
            rel.clone(),
            "adr-required-metadata",
            format!("[ADR-{id}] missing Date"),
        ));
    }
    if makers.as_deref().unwrap_or("").is_empty() {
        violations.push(Violation::error(
            rel.clone(),
            "adr-required-metadata",
            format!("[ADR-{id}] missing Decision makers"),
        ));
    }

    for section in ["Context", "Decision", "Consequences"] {
        if !has_section(&content, section) {
            // ADR-008/009 use narrative sections; Decision may be implied by numbered list under ## Decision
            if section == "Decision" && content.contains("## Decision") {
                continue;
            }
            violations.push(Violation::error(
                rel.clone(),
                "adr-required-section",
                format!("[ADR-{id}] missing non-empty ## {section}"),
            ));
        }
    }

    let status_val = status.unwrap_or_default();
    if (status_val == "Accepted" || status_val == "Superseded")
        && !has_section(&content, "Considered Options")
    {
        violations.push(Violation {
            file: rel.clone(),
            line: None,
            rule: "adr-considered-options".into(),
            description: format!(
                "[ADR-{id}] Accepted/Superseded ADR should have ## Considered Options"
            ),
            severity: qsos_core::Severity::Note,
        });
    }

    if status_val == "Superseded" {
        let adr_refs: HashSet<String> = Regex::new(r"ADR-\d{3}")
            .unwrap()
            .find_iter(&content)
            .map(|m| m.as_str().to_string())
            .collect();
        if adr_refs.len() <= 1 {
            violations.push(Violation::error(
                rel.clone(),
                "adr-superseded-links",
                format!("[ADR-{id}] Superseded ADR must reference another ADR"),
            ));
        }
    }

    violations
}

fn check_sequence(entries: &[(u32, PathBuf)], violations: &mut Vec<Violation>) {
    if entries.is_empty() {
        return;
    }
    let nums: Vec<u32> = entries.iter().map(|(n, _)| *n).collect();
    if nums[0] != 1 {
        violations.push(Violation::error(
            "docs/decisions/",
            "adr-monotonic-sequence",
            format!("sequence must start at 001, found {:03}", nums[0]),
        ));
    }
    for w in nums.windows(2) {
        if w[1] != w[0] + 1 {
            for gap in (w[0] + 1)..w[1] {
                violations.push(Violation::error(
                    "docs/decisions/",
                    "adr-monotonic-sequence",
                    format!("missing ADR-{gap:03}"),
                ));
            }
        }
    }
}

fn extract_field(content: &str, key: &str) -> Option<String> {
    if let Some(yaml) = content.strip_prefix("---") {
        if let Some(end) = yaml[1..].find("---") {
            let block = &yaml[1..end + 1];
            for line in block.lines() {
                let line = line.trim();
                if let Some((k, v)) = line.split_once(':') {
                    if k.trim().eq_ignore_ascii_case(key) {
                        let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                        if !val.is_empty() {
                            return Some(val);
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_plain_field(content: &str, label: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?m)^{label}:\s*(.+)$")).unwrap();
    re.captures(content)
        .map(|c| c[1].trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_heading_value(content: &str, title: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?m)^##\s+{title}\s*\n+\s*([^\n#]+)")).unwrap();
    re.captures(content)
        .map(|c| c[1].trim().trim_matches('*').to_string())
        .filter(|s| !s.is_empty())
}

fn extract_bold_field(content: &str, label: &str) -> Option<String> {
    let re = Regex::new(&format!(r"\*\*{label}:\*\*\s*(.+)")).unwrap();
    re.captures(content)
        .map(|c| c[1].trim().trim_matches('*').to_string())
        .filter(|s| !s.is_empty())
}

fn has_section(content: &str, title: &str) -> bool {
    let re = Regex::new(&format!(r"(?m)^##\s+{title}\s*$")).unwrap();
    if !re.is_match(content) {
        return false;
    }
    let after = content.split(&format!("## {title}")).nth(1).unwrap_or("");
    let body = after.lines().skip(1).take_while(|l| !l.starts_with("## ")).collect::<Vec<_>>().join("\n");
    !body.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::ProjectLayout;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn catches_bad_adr_filename() {
        let dir = tempdir().unwrap();
        let decisions = dir.path().join("docs/decisions");
        fs::create_dir_all(&decisions).unwrap();
        fs::write(decisions.join("bad-name.md"), "# x").unwrap();
        let layout = ProjectLayout::discover(dir.path());
        let v = audit_adrs(&layout);
        assert!(v.iter().any(|x| x.rule == "adr-naming-convention"));
    }
}
