mod ingest;

use ingest::{build_edges, ingest, IngestResult};
use qsos_core::{GraphEdge, GraphNode, GraphRegistry, ProjectLayout};
use std::collections::{HashSet, VecDeque};
use std::fs;

pub fn compile(layout: &ProjectLayout) -> GraphRegistry {
    let ingested = ingest(layout);
    build_registry(&ingested)
}

pub fn compile_and_write(layout: &ProjectLayout) -> GraphRegistry {
    let registry = compile(layout);
    let path = registry_path(layout);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "{}".into());
    let _ = fs::write(&path, json);
    registry
}

pub fn load_registry(layout: &ProjectLayout) -> Option<GraphRegistry> {
    let path = registry_path(layout);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn registry_path(layout: &ProjectLayout) -> std::path::PathBuf {
    layout.work_dir.join("graph-registry.json")
}

fn build_registry(ingested: &IngestResult) -> GraphRegistry {
    GraphRegistry {
        nodes: ingested.nodes(),
        edges: build_edges(ingested),
    }
}

pub fn query_ticket(layout: &ProjectLayout, ticket_id: &str) -> GraphRegistry {
    let registry = load_registry(layout).unwrap_or_else(|| compile_and_write(layout));
    subgraph_for_ticket(&registry, ticket_id)
}

pub fn query_file(layout: &ProjectLayout, file_path: &str) -> GraphRegistry {
    let registry = load_registry(layout).unwrap_or_else(|| compile_and_write(layout));
    subgraph_for_file(&registry, file_path)
}

pub fn query_blast_radius(layout: &ProjectLayout, artifact_path: &str) -> GraphRegistry {
    let registry = load_registry(layout).unwrap_or_else(|| compile_and_write(layout));
    blast_radius(&registry, artifact_path)
}

fn subgraph_for_ticket(registry: &GraphRegistry, ticket_id: &str) -> GraphRegistry {
    let node_ids = collect_connected(registry, ticket_id);
    filter_registry(registry, &node_ids)
}

fn subgraph_for_file(registry: &GraphRegistry, file_path: &str) -> GraphRegistry {
    let normalized = normalize_path(file_path);
    let seed = registry
        .nodes
        .iter()
        .find(|n| normalize_path(&n.id) == normalized || n.id.ends_with(&normalized))
        .map(|n| n.id.clone())
        .unwrap_or_else(|| normalized.clone());
    let node_ids = collect_connected(registry, &seed);
    filter_registry(registry, &node_ids)
}

fn blast_radius(registry: &GraphRegistry, artifact_id: &str) -> GraphRegistry {
    let mut downstream = HashSet::new();
    let mut queue = VecDeque::from([artifact_id.to_string()]);
    downstream.insert(artifact_id.to_string());

    while let Some(current) = queue.pop_front() {
        for edge in &registry.edges {
            if edge.from == current && downstream.insert(edge.to.clone()) {
                queue.push_back(edge.to.clone());
            }
        }
    }

    filter_registry(registry, &downstream)
}

fn collect_connected(registry: &GraphRegistry, seed: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    let mut queue = VecDeque::from([seed.to_string()]);
    ids.insert(seed.to_string());

    while let Some(current) = queue.pop_front() {
        for edge in &registry.edges {
            if edge.from == current && ids.insert(edge.to.clone()) {
                queue.push_back(edge.to.clone());
            }
            if edge.to == current && ids.insert(edge.from.clone()) {
                queue.push_back(edge.from.clone());
            }
        }
    }

    ids
}

fn filter_registry(registry: &GraphRegistry, node_ids: &HashSet<String>) -> GraphRegistry {
    let nodes: Vec<GraphNode> = registry
        .nodes
        .iter()
        .filter(|n| node_ids.contains(&n.id))
        .cloned()
        .collect();
    let edges: Vec<GraphEdge> = registry
        .edges
        .iter()
        .filter(|e| node_ids.contains(&e.from) && node_ids.contains(&e.to))
        .cloned()
        .collect();
    GraphRegistry { nodes, edges }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::ProjectLayout;
    use std::fs;
    use std::path::Path;

    fn write_minimal_fixture(root: &Path) {
        fs::create_dir_all(root.join("docs/decisions")).unwrap();
        fs::create_dir_all(root.join("docs/features")).unwrap();
        fs::create_dir_all(root.join("docs/architecture")).unwrap();
        fs::create_dir_all(root.join("work/QSO-901-demo")).unwrap();

        fs::write(
            root.join("docs/decisions/ADR-901-demo-decision.md"),
            "# ADR-901: Demo Decision\n\nStatus: Accepted\n\nThe qsosCore container documents graph compilation.\n",
        )
        .unwrap();

        fs::write(
            root.join("docs/features/demo.feature"),
            "---\nfeature: Demo Feature\nticket: QSO-901\nstatus: @accepted\n---\n\n# Demo Feature\n\n**Scenario: Graph compiles**\n  Given a fixture project\n  When compile runs\n  Then nodes exist\n",
        )
        .unwrap();

        fs::write(
            root.join("docs/architecture/architecture.dsl"),
            "workspace {\n  model {\n    qsosCore = container \"QSOS Core CLI\" \"Lint and graph.\" \"Rust\" {}\n  }\n}\n",
        )
        .unwrap();

        fs::write(
            root.join("work/tix-manifest.json"),
            r#"{"tickets":[{"id":"QSO-901","title":"Demo ticket","status":"open","path":"work/QSO-901-demo/QSO-901-demo.md"}]}"#,
        )
        .unwrap();

        fs::write(
            root.join("work/QSO-901-demo/QSO-901-demo.md"),
            "---\nid: QSO-901\ntitle: Demo ticket\nstatus: open\nfeatures:\n  - docs/features/demo.feature\nadrs:\n  - docs/decisions/ADR-901-demo-decision.md\n---\n",
        )
        .unwrap();
    }

    #[test]
    fn compiles_fixture_with_all_node_and_edge_types() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let registry = compile(&layout);

        assert!(!registry.nodes.is_empty());
        assert!(!registry.edges.is_empty());

        let kinds: HashSet<_> = registry.nodes.iter().map(|n| n.kind.as_str()).collect();
        for expected in ["ticket", "feature", "scenario", "adr", "dsl_element"] {
            assert!(kinds.contains(expected), "missing node kind {expected}");
        }

        let edge_kinds: HashSet<_> = registry.edges.iter().map(|e| e.kind.as_str()).collect();
        for expected in [
            "ticket→feature",
            "feature→ADR",
            "ADR→dsl_element",
            "scenario→file",
        ] {
            assert!(edge_kinds.contains(expected), "missing edge kind {expected}");
        }
    }

    #[test]
    fn writes_registry_to_work_dir() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        compile_and_write(&layout);
        assert!(registry_path(&layout).is_file());
    }

    #[test]
    fn query_ticket_returns_connected_subgraph() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let sub = query_ticket(&layout, "QSO-901");
        assert!(sub.nodes.iter().any(|n| n.id == "QSO-901"));
        assert!(sub.nodes.iter().any(|n| n.kind == "feature"));
    }

    #[test]
    fn scenario_id_format() {
        assert_eq!(
            ingest::scenario_id("docs/features/demo.feature", "Graph compiles"),
            "docs/features/demo.feature::Graph compiles"
        );
    }
}
