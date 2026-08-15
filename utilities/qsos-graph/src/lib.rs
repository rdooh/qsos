mod ingest;

use ingest::{build_edges, ingest, IngestResult};
use qsos_core::{GraphEdge, GraphNode, GraphRegistry, ProjectLayout, QueryResult};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn compile(layout: &ProjectLayout) -> GraphRegistry {
    let ingested = ingest(layout);
    build_registry(&ingested)
}

pub fn compile_and_write(layout: &ProjectLayout) -> GraphRegistry {
    let registry = compile(layout);
    write_registry(layout, &registry);
    registry
}

pub fn load_registry(layout: &ProjectLayout) -> Option<GraphRegistry> {
    let path = registry_path(layout);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn registry_path(layout: &ProjectLayout) -> PathBuf {
    layout.work_dir.join("graph-registry.json")
}

pub fn ensure_registry(layout: &ProjectLayout) -> GraphRegistry {
    if load_registry(layout).is_none() || registry_is_stale(layout) {
        compile_and_write(layout)
    } else {
        load_registry(layout).expect("registry present after stale check")
    }
}

fn write_registry(layout: &ProjectLayout, registry: &GraphRegistry) {
    let path = registry_path(layout);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(registry).unwrap_or_else(|_| "{}".into());
    let _ = fs::write(path, json);
}

fn registry_is_stale(layout: &ProjectLayout) -> bool {
    let path = registry_path(layout);
    let Ok(registry_meta) = fs::metadata(&path) else {
        return true;
    };
    let Ok(registry_mtime) = registry_meta.modified() else {
        return true;
    };

    let contracts_dir = layout.root.join("docs/contracts");
    let watch_dirs = [
        layout.decisions_dir.as_path(),
        layout.features_dir.as_path(),
        layout.work_dir.as_path(),
        layout.architecture_dir.as_path(),
        contracts_dir.as_path(),
    ];

    for dir in watch_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in WalkDir::new(dir).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(source_mtime) = meta.modified() {
                    if source_mtime > registry_mtime {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn build_registry(ingested: &IngestResult) -> GraphRegistry {
    GraphRegistry {
        nodes: ingested.nodes(),
        edges: build_edges(ingested),
    }
}

pub fn query_ticket(layout: &ProjectLayout, ticket_id: &str) -> QueryResult {
    let registry = ensure_registry(layout);
    QueryResult::from_registry(
        "ticket",
        ticket_id,
        subgraph_for_ticket(&registry, ticket_id),
    )
}

pub fn query_file(layout: &ProjectLayout, file_path: &str) -> QueryResult {
    let registry = ensure_registry(layout);
    let rel = normalize_query_path(layout, file_path);
    let seed = resolve_node_id(&registry, &rel);
    QueryResult::from_registry("file", &rel, subgraph_for_seed(&registry, &seed))
}

pub fn query_blast_radius(layout: &ProjectLayout, artifact: &str) -> QueryResult {
    let registry = ensure_registry(layout);
    let rel = normalize_query_path(layout, artifact);
    let seed = resolve_node_id(&registry, &rel);
    QueryResult::from_registry("blast-radius", &rel, blast_radius(&registry, &seed))
}

fn normalize_query_path(layout: &ProjectLayout, path: &str) -> String {
    let path = path.replace('\\', "/");
    let as_path = Path::new(&path);
    if as_path.is_absolute() {
        layout.rel_path(as_path)
    } else {
        path.trim_start_matches("./").to_string()
    }
}

fn resolve_node_id(registry: &GraphRegistry, path_or_id: &str) -> String {
    let normalized = path_or_id.replace('\\', "/");

    if registry.nodes.iter().any(|n| n.id == normalized) {
        return normalized;
    }

    if let Some(node) = registry
        .nodes
        .iter()
        .find(|n| normalized.ends_with(&n.id) || n.id.ends_with(&normalized))
    {
        return node.id.clone();
    }

    if let Some(caps) = regex::Regex::new(r"ADR-(\d{3})")
        .unwrap()
        .captures(&normalized)
    {
        let adr_id = format!("ADR-{}", &caps[1]);
        if registry.nodes.iter().any(|n| n.id == adr_id) {
            return adr_id;
        }
    }

    normalized
}

fn subgraph_for_ticket(registry: &GraphRegistry, ticket_id: &str) -> GraphRegistry {
    let node_ids = collect_connected(registry, ticket_id);
    filter_registry(registry, &node_ids)
}

fn subgraph_for_seed(registry: &GraphRegistry, seed: &str) -> GraphRegistry {
    let node_ids = collect_connected(registry, seed);
    filter_registry(registry, &node_ids)
}

fn blast_radius(registry: &GraphRegistry, seed: &str) -> GraphRegistry {
    let mut downstream = HashSet::new();
    let mut queue = VecDeque::from([seed.to_string()]);
    downstream.insert(seed.to_string());

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

#[cfg(test)]
mod tests {
    use super::*;
    use qsos_core::ProjectLayout;
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

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
            r#"{"tickets":[{"id":"QSO-901","title":"Demo ticket","status":"todo","path":"work/QSO-901-demo/QSO-901-demo.md"}]}"#,
        )
        .unwrap();

        fs::write(
            root.join("work/QSO-901-demo/QSO-901-demo.md"),
            "---\nid: QSO-901\ntitle: Demo ticket\nstatus: todo\nfeatures:\n  - docs/features/demo.feature\nadrs:\n  - docs/decisions/ADR-901-demo-decision.md\n---\n",
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
    }

    #[test]
    fn query_ticket_returns_connected_subgraph_with_summary() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let result = query_ticket(&layout, "QSO-901");
        assert_eq!(result.query_type, "ticket");
        assert!(result.nodes.iter().any(|n| n.id == "QSO-901"));
        assert!(result.summary.features >= 1);
    }

    #[test]
    fn query_file_by_feature_path() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let result = query_file(&layout, "docs/features/demo.feature");
        assert_eq!(result.query_type, "file");
        assert!(result.nodes.iter().any(|n| n.kind == "scenario"));
    }

    #[test]
    fn query_blast_radius_from_adr_file() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        let result = query_blast_radius(&layout, "docs/decisions/ADR-901-demo-decision.md");
        assert_eq!(result.query_type, "blast-radius");
        assert!(result.nodes.iter().any(|n| n.kind == "dsl_element"));
    }

    #[test]
    fn auto_recompiles_when_registry_stale() {
        let dir = tempfile::tempdir().unwrap();
        write_minimal_fixture(dir.path());
        let layout = ProjectLayout::discover(dir.path());
        compile_and_write(&layout);
        thread::sleep(Duration::from_millis(1100));
        fs::write(
            dir.path().join("docs/features/demo.feature"),
            "---\nfeature: Demo Feature\nticket: QSO-901\nstatus: @accepted\n---\n\n# Demo Feature\n\n**Scenario: Graph compiles**\n  Given a fixture\n  When compile runs\n  Then nodes exist\n\n**Scenario: Query works**\n  Given a graph\n  When querying\n  Then subgraph returns\n",
        )
        .unwrap();
        let result = query_ticket(&layout, "QSO-901");
        assert!(result.summary.scenarios >= 2);
    }

    #[test]
    fn scenario_id_format() {
        assert_eq!(
            ingest::scenario_id("docs/features/demo.feature", "Graph compiles"),
            "docs/features/demo.feature::Graph compiles"
        );
    }
}
