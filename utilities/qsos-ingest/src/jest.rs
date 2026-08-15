use crate::{slugify, TestRecord};
use serde_json::Value;

pub fn detect(content: &str) -> bool {
    serde_json::from_str::<Value>(content)
        .ok()
        .and_then(|v| {
            v.get("testResults")
                .and_then(|r| r.as_array())
                .filter(|a| !a.is_empty())
                .and(v.get("numTotalTests").is_some().then_some(()))
        })
        .is_some()
}

pub fn parse(content: &str, source: &str) -> Vec<TestRecord> {
    let Ok(json) = serde_json::from_str::<Value>(content) else {
        return Vec::new();
    };

    let mut tests = Vec::new();
    let Some(results) = json.get("testResults").and_then(|v| v.as_array()) else {
        return tests;
    };

    for suite in results {
        let suite_name = suite
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let Some(cases) = suite.get("testResults").and_then(|v| v.as_array()) else {
            continue;
        };

        for case in cases {
            let name = case
                .get("fullName")
                .or_else(|| case.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("(unnamed)")
                .to_string();

            let status = case
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let ancestors: Vec<String> = case
                .get("ancestorTitles")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            let context = format!("{} {}", ancestors.join(" "), name);
            let scenario_ref = extract_scenario_ref(&context).or_else(|| extract_scenario_ref(&name));
            let short_name = name.split(' ').last().unwrap_or(&name).to_string();
            let id = slugify(&format!("{suite_name}::{short_name}"));

            tests.push(TestRecord {
                id,
                name,
                suite_name: suite_name.clone(),
                status,
                scenario_ref,
                source: source.to_string(),
            });
        }
    }

    tests
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
    fn parses_jest_fixture() {
        let json = r#"{
  "numTotalTests": 1,
  "testResults": [{
    "name": "demo.spec.ts",
    "testResults": [{
      "fullName": "demo Graph compiles",
      "status": "passed",
      "ancestorTitles": ["demo"]
    }]
  }]
}"#;
        let tests = parse(json, "test-results/unit.json");
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].status, "passed");
    }
}
