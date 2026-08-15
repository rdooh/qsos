use crate::{slugify, TestRecord};

pub fn detect(content: &str) -> bool {
    content.contains("<testsuites") || content.contains("<testsuite")
}

pub fn parse(content: &str, source: &str) -> Vec<TestRecord> {
    let mut tests = Vec::new();
    let mut pos = 0;

    while pos < content.len() {
        let Some(open_idx) = content[pos..].find("<testcase") else {
            break;
        };
        let open_idx = pos + open_idx;
        let Some(tag_end_idx) = content[open_idx..].find('>') else {
            break;
        };
        let tag_end_idx = open_idx + tag_end_idx;
        let open_tag = &content[open_idx..=tag_end_idx];
        let is_self_close = open_tag.ends_with("/>");

        let (body, after_idx) = if is_self_close {
            (String::new(), tag_end_idx + 1)
        } else {
            let close_tag = "</testcase>";
            let Some(close_idx) = content[tag_end_idx..].find(close_tag) else {
                pos = tag_end_idx + 1;
                continue;
            };
            let close_idx = tag_end_idx + close_idx;
            (
                content[tag_end_idx + 1..close_idx].to_string(),
                close_idx + close_tag.len(),
            )
        };
        pos = after_idx;

        let attrs = open_tag
            .trim_start_matches("<testcase")
            .trim_end_matches('>')
            .trim_end_matches('/');
        let name = attr(attrs, "name").unwrap_or_default();
        let classname = attr(attrs, "classname").unwrap_or_default();

        let status = if body.to_lowercase().contains("<failure") {
            "failed"
        } else if body.to_lowercase().contains("<error") {
            "error"
        } else if body.to_lowercase().contains("<skipped") {
            "skipped"
        } else {
            "passed"
        };

        let scenario_ref = extract_scenario_ref(&body).or_else(|| extract_scenario_ref(&name));
        let id = slugify(&format!("{classname}::{name}"));

        tests.push(TestRecord {
            id,
            name: if name.is_empty() {
                "(unnamed)".into()
            } else {
                name
            },
            suite_name: classname,
            status: status.to_string(),
            scenario_ref,
            source: source.to_string(),
        });
    }

    tests
}

fn attr(attrs: &str, name: &str) -> Option<String> {
    let pattern = format!(" {name}=");
    let start = attrs
        .find(&pattern)
        .map(|i| i + 1)
        .or_else(|| {
            if attrs.starts_with(&format!("{name}=")) {
                Some(0)
            } else {
                None
            }
        })?;
    let rest = &attrs[start + name.len() + 1..];
    if rest.starts_with('"') {
        let end = rest[1..].find('"')? + 1;
        return Some(rest[1..end].to_string());
    }
    if rest.starts_with('\'') {
        let end = rest[1..].find('\'')? + 1;
        return Some(rest[1..end].to_string());
    }
    rest.split_whitespace()
        .next()
        .map(|v| v.trim_end_matches('/').to_string())
}

fn extract_scenario_ref(text: &str) -> Option<String> {
    regex::Regex::new(r#"(?i)@scenario[\s(]+["']?([^"'\s)]+)["']?"#)
        .unwrap()
        .captures(text)
        .map(|c| c[1].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_junit_fixture() {
        let xml = r#"<?xml version="1.0"?><testsuites><testsuite><testcase classname="suite" name="Graph compiles" time="0.1"/></testsuite></testsuites>"#;
        let tests = parse(xml, "test-results/unit.xml");
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "Graph compiles");
        assert_eq!(tests[0].status, "passed");
    }
}
