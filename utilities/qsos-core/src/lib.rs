use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_VIOLATIONS: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Violation {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub rule: String,
    pub description: String,
    pub severity: Severity,
}

impl Violation {
    pub fn error(file: impl Into<String>, rule: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line: None,
            rule: rule.into(),
            description: description.into(),
            severity: Severity::Error,
        }
    }

    pub fn at_line(
        file: impl Into<String>,
        line: u32,
        rule: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line: Some(line),
            rule: rule.into(),
            description: description.into(),
            severity: Severity::Error,
        }
    }

    pub fn note(file: impl Into<String>, rule: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line: None,
            rule: rule.into(),
            description: description.into(),
            severity: Severity::Note,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    pub violations: Vec<Violation>,
}

impl LintReport {
    pub fn new(violations: Vec<Violation>) -> Self {
        Self { violations }
    }

    pub fn has_errors(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Error)
    }

    pub fn exit_code(&self) -> i32 {
        if self.has_errors() {
            EXIT_VIOLATIONS
        } else {
            EXIT_SUCCESS
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRegistry {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuerySummary {
    pub tickets: usize,
    pub features: usize,
    pub scenarios: usize,
    pub adrs: usize,
    pub dsl_elements: usize,
    pub contracts: usize,
    pub edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query_type: String,
    pub query: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub summary: QuerySummary,
}

impl QueryResult {
    pub fn from_registry(query_type: impl Into<String>, query: impl Into<String>, registry: GraphRegistry) -> Self {
        let summary = summarize(&registry.nodes, &registry.edges);
        Self {
            query_type: query_type.into(),
            query: query.into(),
            nodes: registry.nodes,
            edges: registry.edges,
            summary,
        }
    }
}

pub fn summarize(nodes: &[GraphNode], edges: &[GraphEdge]) -> QuerySummary {
    let mut summary = QuerySummary {
        edges: edges.len(),
        ..Default::default()
    };
    for node in nodes {
        match node.kind.as_str() {
            "ticket" => summary.tickets += 1,
            "feature" => summary.features += 1,
            "scenario" => summary.scenarios += 1,
            "adr" => summary.adrs += 1,
            "dsl_element" => summary.dsl_elements += 1,
            "contract" => summary.contracts += 1,
            _ => {}
        }
    }
    summary
}

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    pub decisions_dir: PathBuf,
    pub features_dir: PathBuf,
    pub architecture_dir: PathBuf,
    pub work_dir: PathBuf,
}

impl ProjectLayout {
    pub fn discover(start: impl AsRef<Path>) -> Self {
        let root = start.as_ref().to_path_buf();
        Self {
            decisions_dir: root.join("docs/decisions"),
            features_dir: root.join("docs/features"),
            architecture_dir: root.join("docs/architecture"),
            work_dir: root.join("work"),
            root,
        }
    }

    pub fn rel_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}
