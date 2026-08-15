use qsos_core::ProjectLayout;
use std::collections::HashSet;
use std::fs;

mod jest;
mod junit;
mod resolver;

pub use resolver::{resolve_tests, ResolveReport};

#[derive(Debug, Clone)]
pub struct TestRecord {
    pub id: String,
    pub name: String,
    pub suite_name: String,
    pub status: String,
    pub scenario_ref: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IngestReport {
    pub tests_ingested: usize,
    pub resolved: usize,
    pub unresolved: usize,
}

pub fn ingest(layout: &ProjectLayout) -> Result<IngestReport, String> {
    let tests = load_test_results(layout)?;
    if tests.is_empty() {
        return Ok(IngestReport {
            tests_ingested: 0,
            resolved: 0,
            unresolved: 0,
        });
    }

    let mut registry = qsos_graph::ensure_registry(layout);
    let report = resolve_tests(&mut registry, &tests);
    qsos_graph::save_registry(layout, &registry);
    Ok(IngestReport {
        tests_ingested: tests.len(),
        resolved: report.resolved,
        unresolved: report.unresolved,
    })
}

fn load_test_results(layout: &ProjectLayout) -> Result<Vec<TestRecord>, String> {
    let results_dir = layout.root.join("test-results");
    if !results_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut tests = Vec::new();
    let mut seen_ids = HashSet::new();

    for entry in fs::read_dir(&results_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let rel = layout.rel_path(&path);
        let parsed = if path.extension().and_then(|e| e.to_str()) == Some("json")
            && jest::detect(&content)
        {
            jest::parse(&content, &rel)
        } else if path.extension().and_then(|e| e.to_str()) == Some("xml")
            && junit::detect(&content)
        {
            junit::parse(&content, &rel)
        } else if jest::detect(&content) {
            jest::parse(&content, &rel)
        } else if junit::detect(&content) {
            junit::parse(&content, &rel)
        } else {
            continue;
        };

        for test in parsed {
            if seen_ids.insert(test.id.clone()) {
                tests.push(test);
            }
        }
    }

    Ok(tests)
}

pub fn slugify(value: &str) -> String {
    let lower = value.to_lowercase();
    let re = regex::Regex::new(r"[^a-z0-9]+").unwrap();
    re.replace_all(&lower, "-")
        .trim_matches('-')
        .to_string()
}

pub fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .replace(['"', '\''], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::ProjectLayout;
    use std::path::Path;

    fn write_ingest_fixture(root: &Path) {
        fs::create_dir_all(root.join("docs/features")).unwrap();
        fs::create_dir_all(root.join("docs/decisions")).unwrap();
        fs::create_dir_all(root.join("docs/architecture")).unwrap();
        fs::create_dir_all(root.join("work/QSO-901-demo")).unwrap();
        fs::create_dir_all(root.join("test-results")).unwrap();

        fs::write(
            root.join("docs/features/demo.feature"),
            "---\nfeature: Demo Feature\nticket: QSO-901\nstatus: @accepted\n---\n\n# Demo Feature\n\n**Scenario: Graph compiles**\n  Given a fixture\n  When compile runs\n  Then nodes exist\n\n**Scenario: Query works**\n  Given a graph\n  When querying\n  Then subgraph returns\n",
        )
        .unwrap();

        fs::write(
            root.join("docs/architecture/architecture.dsl"),
            "workspace { model { core = container \"Core\" \"Core\" \"Rust\" {} } }\n",
        )
        .unwrap();

        fs::write(
            root.join("work/tix-manifest.json"),
            r#"{"tickets":[{"id":"QSO-901","title":"Demo","status":"todo","path":"work/QSO-901-demo/QSO-901-demo.md"}]}"#,
        )
        .unwrap();

        fs::write(
            root.join("work/QSO-901-demo/QSO-901-demo.md"),
            "---\nid: QSO-901\ntitle: Demo\nstatus: todo\nfeatures:\n  - docs/features/demo.feature\n---\n",
        )
        .unwrap();

        fs::write(
            root.join("test-results/unit.xml"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="demo" tests="2">
    <testcase classname="demo" name="Graph compiles" time="0.01"/>
    <testcase classname="demo" name="failing test" time="0.02">
      <failure message="assertion failed">expected true</failure>
    </testcase>
  </testsuite>
</testsuites>"#,
        )
        .unwrap();
    }

    #[test]
    fn ingest_adds_verifies_edges() {
        let dir = tempfile::tempdir().unwrap();
        write_ingest_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());

        let report = ingest(&layout).unwrap();
        assert_eq!(report.tests_ingested, 2);
        assert!(report.resolved >= 1);

        let registry = qsos_graph::load_registry(&layout).unwrap();
        let verifies: Vec<_> = registry
            .edges
            .iter()
            .filter(|e| e.kind.starts_with("VERIFIES"))
            .collect();
        assert_eq!(verifies.len(), 1);
        assert_eq!(
            verifies[0].to,
            "docs/features/demo.feature::Graph compiles"
        );
    }
}
