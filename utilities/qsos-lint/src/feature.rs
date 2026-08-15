use qsos_core::{ProjectLayout, Violation};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const VALID_LIFECYCLE: &[&str] = &["@proposed", "@accepted", "@in-progress", "@done", "@deprecated"];

pub fn audit_features(layout: &ProjectLayout) -> Vec<Violation> {
    let mut violations = Vec::new();
    if !layout.features_dir.is_dir() {
        return violations;
    }
    for entry in fs::read_dir(&layout.features_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("feature") {
            violations.extend(audit_feature_file(layout, &path));
        }
    }
    violations
}

pub fn audit_feature_file(layout: &ProjectLayout, path: &Path) -> Vec<Violation> {
    let rel = layout.rel_path(path);
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return vec![Violation::error(
                rel,
                "feature-read-error",
                format!("cannot read file: {e}"),
            )];
        }
    };

    let mut violations = Vec::new();

    if content.trim().is_empty() {
        violations.push(Violation::error(rel.clone(), "feature-not-empty", "file is blank"));
        return violations;
    }

    let status = extract_frontmatter_field(&content, "status");
    if status
        .as_deref()
        .map(|s| VALID_LIFECYCLE.contains(&s))
        .unwrap_or(false)
    {
        // ok
    } else {
        violations.push(Violation::error(
            rel.clone(),
            "feature-lifecycle-tag",
            format!(
                "frontmatter status must be one of {}",
                VALID_LIFECYCLE.join(", ")
            ),
        ));
    }

    let feature_count = content.matches("Feature:").count();
    if feature_count == 0 {
        violations.push(Violation::error(
            rel.clone(),
            "feature-keyword-present",
            "file must contain at least one Feature: declaration",
        ));
    }

    check_scenario_names(&content, &rel, &mut violations);
    check_outline_examples(&content, &rel, &mut violations);
    check_style(&content, &rel, &mut violations, content.starts_with("---"));

    violations
}

fn extract_frontmatter_field(content: &str, key: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    for line in rest[..end].lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case(key) {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn check_scenario_names(content: &str, rel: &str, violations: &mut Vec<Violation>) {
    let re = Regex::new(r"(?m)^\s*\*\*Scenario(?::|\s+Outline:)\s*(.+?)\*\*").unwrap();
    let mut seen = HashSet::new();
    for caps in re.captures_iter(content) {
        let name = caps[1].trim().to_string();
        if !seen.insert(name.clone()) {
            violations.push(Violation::error(
                rel,
                "duplicate-scenario-name",
                format!("duplicate scenario name: {name}"),
            ));
        }
    }
}

fn check_outline_examples(content: &str, rel: &str, violations: &mut Vec<Violation>) {
    let outline_re = Regex::new(r"(?m)\*\*Scenario Outline:").unwrap();
    for m in outline_re.find_iter(content) {
        let tail = &content[m.start()..];
        if !tail.contains("Examples:") && !tail.contains("**Examples:") {
            violations.push(Violation::error(
                rel,
                "outline-has-examples",
                "Scenario Outline must include Examples table",
            ));
        }
    }
}

fn check_style(content: &str, rel: &str, violations: &mut Vec<Violation>, hybrid: bool) {
    if !content.is_empty() && !content.ends_with('\n') {
        violations.push(Violation::error(
            rel,
            "single-newline-eof",
            "file must end with a newline",
        ));
    }

    let step_re = Regex::new(r"^(Given|When|Then|And|But)\b").unwrap();

    for (i, line) in content.lines().enumerate() {
        let line_num = (i + 1) as u32;
        if line.ends_with(' ') || line.ends_with('\t') {
            violations.push(Violation::at_line(
                rel,
                line_num,
                "no-trailing-whitespace",
                "line has trailing whitespace",
            ));
        }

        if hybrid {
            continue;
        }

        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        // Classic Gherkin indentation (non-hybrid files only)
        if trimmed.starts_with("Feature:") && indent != 0 {
            violations.push(Violation::at_line(rel, line_num, "indentation", "Feature: must be at column 0"));
        } else if (trimmed.starts_with("Scenario:") || trimmed.starts_with("Background:")) && indent != 2 {
            violations.push(Violation::at_line(rel, line_num, "indentation", "Scenario/Background must be indented 2 spaces"));
        } else if step_re.is_match(trimmed) && indent != 4 {
            violations.push(Violation::at_line(rel, line_num, "indentation", "steps must be indented 4 spaces"));
        } else if trimmed.starts_with('|') && indent != 6 {
            violations.push(Violation::at_line(rel, line_num, "indentation", "table rows must be indented 6 spaces"));
        }
    }
}
