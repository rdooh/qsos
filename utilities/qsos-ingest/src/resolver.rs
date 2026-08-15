use crate::{normalize, slugify, TestRecord};
use qsos_core::{GraphEdge, GraphNode, GraphRegistry};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ResolveReport {
    pub resolved: usize,
    pub unresolved: usize,
}

pub fn resolve_tests(registry: &mut GraphRegistry, tests: &[TestRecord]) -> ResolveReport {
    let scenarios: Vec<&GraphNode> = registry
        .nodes
        .iter()
        .filter(|n| n.kind == "scenario")
        .collect();

    let mut by_title: HashMap<String, String> = HashMap::new();
    let mut by_slug: HashMap<String, String> = HashMap::new();

    for sc in &scenarios {
        by_title.insert(normalize(&sc.label), sc.id.clone());
        by_slug.insert(slugify(&sc.label), sc.id.clone());
        by_slug.insert(slugify(&sc.id), sc.id.clone());
    }

    let mut report = ResolveReport::default();

    for test in tests {
        let test_node_id = format!("test::{}", test.id);
        upsert_test_node(registry, &test_node_id, test);

        let (target, match_pass) = match resolve_target(test, &by_slug, &by_title) {
            Some(v) => v,
            None => {
                report.unresolved += 1;
                continue;
            }
        };

        push_verifies_edge(registry, &test_node_id, &target, &test.status, match_pass);
        report.resolved += 1;
    }

    report
}

fn resolve_target(
    test: &TestRecord,
    by_slug: &HashMap<String, String>,
    by_title: &HashMap<String, String>,
) -> Option<(String, &'static str)> {
    if let Some(ref scenario_ref) = test.scenario_ref {
        let ref_slug = slugify(scenario_ref);
        if let Some(id) = by_slug.get(&ref_slug) {
            return Some((id.clone(), "tag-annotation"));
        }
    }

    let norm_name = normalize(&test.name);
    if let Some(id) = by_title.get(&norm_name) {
        return Some((id.clone(), "name-match"));
    }

    if let Some(short) = test.name.split(' ').last() {
        let short_norm = normalize(short);
        if let Some(id) = by_title.get(&short_norm) {
            return Some((id.clone(), "name-match"));
        }
    }

    None
}

fn upsert_test_node(registry: &mut GraphRegistry, id: &str, test: &TestRecord) {
    if registry.nodes.iter().any(|n| n.id == id) {
        return;
    }
    registry.nodes.push(GraphNode {
        id: id.to_string(),
        kind: "test".to_string(),
        label: format!("{} ({})", test.name, test.status),
    });
}

fn push_verifies_edge(
    registry: &mut GraphRegistry,
    from: &str,
    to: &str,
    status: &str,
    match_pass: &str,
) {
    registry.edges.retain(|e| !(e.from == from && e.to == to && e.kind.starts_with("VERIFIES")));
    registry.edges.push(GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
        kind: format!("VERIFIES:{status}:{match_pass}"),
    });
}
