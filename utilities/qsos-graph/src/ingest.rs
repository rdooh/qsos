use qsos_core::{GraphEdge, GraphNode, ProjectLayout};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TicketRecord {
    pub id: String,
    pub title: String,
    pub status: String,
    pub features: Vec<String>,
    pub adrs: Vec<String>,
    pub depends_on: Vec<String>,
    pub file: String,
}

#[derive(Debug, Clone)]
pub struct FeatureRecord {
    pub path: String,
    pub title: String,
    pub ticket: Option<String>,
    pub scenarios: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdrRecord {
    pub id: String,
    pub title: String,
    pub file: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct DslElement {
    pub name: String,
    pub element_type: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContractRecord {
    pub id: String,
    pub title: String,
    pub file: String,
    pub adr: Option<String>,
}

pub fn ingest(layout: &ProjectLayout) -> IngestResult {
    IngestResult {
        tickets: ingest_tickets(layout),
        features: ingest_features(layout),
        adrs: ingest_adrs(layout),
        dsl_elements: ingest_dsl_elements(layout),
        contracts: ingest_contracts(layout),
    }
}

#[derive(Debug, Default)]
pub struct IngestResult {
    pub tickets: Vec<TicketRecord>,
    pub features: Vec<FeatureRecord>,
    pub adrs: Vec<AdrRecord>,
    pub dsl_elements: Vec<DslElement>,
    pub contracts: Vec<ContractRecord>,
}

impl IngestResult {
    pub fn nodes(&self) -> Vec<GraphNode> {
        let mut nodes = Vec::new();
        let mut seen = HashSet::new();

        for ticket in &self.tickets {
            push_node(
                &mut nodes,
                &mut seen,
                &ticket.id,
                "ticket",
                &format!("{} — {}", ticket.id, ticket.title),
            );
        }

        for feature in &self.features {
            push_node(
                &mut nodes,
                &mut seen,
                &feature.path,
                "feature",
                &feature.title,
            );
            for scenario in &feature.scenarios {
                let id = scenario_id(&feature.path, scenario);
                push_node(&mut nodes, &mut seen, &id, "scenario", scenario);
            }
        }

        for adr in &self.adrs {
            push_node(
                &mut nodes,
                &mut seen,
                &adr.id,
                "adr",
                &format!("{} — {}", adr.id, adr.title),
            );
        }

        for element in &self.dsl_elements {
            push_node(
                &mut nodes,
                &mut seen,
                &element.name,
                "dsl_element",
                &format!("{} ({})", element.name, element.element_type),
            );
        }

        for contract in &self.contracts {
            push_node(
                &mut nodes,
                &mut seen,
                &contract.id,
                "contract",
                &contract.title,
            );
        }

        nodes
    }
}

fn push_node(nodes: &mut Vec<GraphNode>, seen: &mut HashSet<String>, id: &str, kind: &str, label: &str) {
    if seen.insert(id.to_string()) {
        nodes.push(GraphNode {
            id: id.to_string(),
            kind: kind.to_string(),
            label: label.to_string(),
        });
    }
}

pub fn scenario_id(feature_path: &str, scenario_name: &str) -> String {
    format!("{feature_path}::{scenario_name}")
}

fn ingest_tickets(layout: &ProjectLayout) -> Vec<TicketRecord> {
    let mut tickets = Vec::new();
    let manifest_path = layout.work_dir.join("tix-manifest.json");
    let manifest_entries = load_manifest_entries(&manifest_path);

    for (id, path, title, status) in manifest_entries {
        let ticket_path = layout.root.join(&path);
        let content = fs::read_to_string(&ticket_path).unwrap_or_default();
        let frontmatter = parse_frontmatter(&content);
        tickets.push(TicketRecord {
            id: id.clone(),
            title: frontmatter
                .get("title")
                .cloned()
                .unwrap_or(title),
            status: frontmatter
                .get("status")
                .cloned()
                .unwrap_or(status),
            features: frontmatter_list_from_content(&content, "features"),
            adrs: normalize_adr_ids(&frontmatter_list_from_content(&content, "adrs")),
            depends_on: frontmatter_list_from_content(&content, "depends_on"),
            file: path,
        });
    }

    tickets
}

fn load_manifest_entries(manifest_path: &Path) -> Vec<(String, String, String, String)> {
    let raw = match fs::read_to_string(manifest_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let manifest: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    manifest
        .get("tickets")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    Some((
                        entry.get("id")?.as_str()?.to_string(),
                        entry.get("path")?.as_str()?.to_string(),
                        entry.get("title")?.as_str()?.to_string(),
                        entry.get("status")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn ingest_features(layout: &ProjectLayout) -> Vec<FeatureRecord> {
    let mut features = Vec::new();
    if !layout.features_dir.is_dir() {
        return features;
    }

    for entry in fs::read_dir(&layout.features_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("feature") {
            continue;
        }
        let rel = layout.rel_path(&path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let frontmatter = parse_frontmatter(&content);
        let title = frontmatter
            .get("feature")
            .cloned()
            .or_else(|| extract_markdown_title(&content))
            .unwrap_or_else(|| rel.clone());
        let ticket = frontmatter.get("ticket").cloned();
        let scenarios = parse_scenarios(&content);
        features.push(FeatureRecord {
            path: rel,
            title,
            ticket,
            scenarios,
        });
    }

    features
}

fn ingest_adrs(layout: &ProjectLayout) -> Vec<AdrRecord> {
    let mut adrs = Vec::new();
    if !layout.decisions_dir.is_dir() {
        return adrs;
    }

    let re = Regex::new(r"^ADR-(\d{3})-([a-z0-9-]+)\.md$").unwrap();
    for entry in fs::read_dir(&layout.decisions_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "README.md" {
            continue;
        }
        let Some(caps) = re.captures(&name) else {
            continue;
        };
        let id = format!("ADR-{}", &caps[1]);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let title = content
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l.trim_start_matches("# ").trim().to_string())
            .unwrap_or_else(|| name.clone());
        adrs.push(AdrRecord {
            id,
            title,
            file: layout.rel_path(&path),
            body: content,
        });
    }

    adrs.sort_by(|a, b| a.id.cmp(&b.id));
    adrs
}

fn ingest_dsl_elements(layout: &ProjectLayout) -> Vec<DslElement> {
    let dsl_path = layout.architecture_dir.join("architecture.dsl");
    let content = match fs::read_to_string(&dsl_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let re = Regex::new(r"(\w+)\s*=\s*(softwareSystem|container|component|person)\s+").unwrap();
    let mut elements = Vec::new();
    let mut seen = HashSet::new();
    for caps in re.captures_iter(&content) {
        let name = caps[1].to_string();
        if seen.insert(name.clone()) {
            elements.push(DslElement {
                name,
                element_type: caps[2].to_string(),
            });
        }
    }
    elements
}

fn ingest_contracts(layout: &ProjectLayout) -> Vec<ContractRecord> {
    let contracts_dir = layout.root.join("docs/contracts");
    if !contracts_dir.is_dir() {
        return Vec::new();
    }

    let mut contracts = Vec::new();
    for entry in fs::read_dir(&contracts_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = value
            .get("$id")
            .or_else(|| value.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        let title = value
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        let adr = value
            .get("adr")
            .and_then(|v| v.as_str())
            .map(normalize_adr_id);
        contracts.push(ContractRecord {
            id,
            title,
            file: layout.rel_path(&path),
            adr,
        });
    }
    contracts
}

pub fn build_edges(ingest: &IngestResult) -> Vec<GraphEdge> {
    let mut edges = Vec::new();
    let mut seen = HashSet::new();

    let feature_by_path: HashMap<&str, &FeatureRecord> = ingest
        .features
        .iter()
        .map(|f| (f.path.as_str(), f))
        .collect();
    let adr_by_id: HashMap<&str, &AdrRecord> = ingest.adrs.iter().map(|a| (a.id.as_str(), a)).collect();

    for ticket in &ingest.tickets {
        for feature_path in &ticket.features {
            push_edge(
                &mut edges,
                &mut seen,
                &ticket.id,
                feature_path,
                "ticket→feature",
            );
        }

        for adr_ref in &ticket.adrs {
            for feature_path in &ticket.features {
                push_edge(
                    &mut edges,
                    &mut seen,
                    feature_path,
                    adr_ref,
                    "feature→ADR",
                );
            }
        }

        for dep in &ticket.depends_on {
            push_edge(
                &mut edges,
                &mut seen,
                &ticket.id,
                dep,
                "ticket→ticket",
            );
        }
    }

    for feature in &ingest.features {
        if let Some(ticket_id) = &feature.ticket {
            push_edge(
                &mut edges,
                &mut seen,
                ticket_id,
                &feature.path,
                "ticket→feature",
            );
        }

        for scenario in &feature.scenarios {
            let scenario_node = scenario_id(&feature.path, scenario);
            push_edge(
                &mut edges,
                &mut seen,
                &scenario_node,
                &feature.path,
                "scenario→file",
            );
        }
    }

    for adr in &ingest.adrs {
        for element in &ingest.dsl_elements {
            if adr.body.contains(&element.name) {
                push_edge(
                    &mut edges,
                    &mut seen,
                    &adr.id,
                    &element.name,
                    "ADR→dsl_element",
                );
            }
        }
    }

    for contract in &ingest.contracts {
        if let Some(adr_id) = &contract.adr {
            if adr_by_id.contains_key(adr_id.as_str()) {
                push_edge(
                    &mut edges,
                    &mut seen,
                    adr_id,
                    &contract.id,
                    "ADR→contract",
                );
            }
        }
    }

    // Reserved for future query helpers
    let _ = feature_by_path;

    edges
}

fn push_edge(edges: &mut Vec<GraphEdge>, seen: &mut HashSet<(String, String, String)>, from: &str, to: &str, kind: &str) {
    let key = (from.to_string(), to.to_string(), kind.to_string());
    if seen.insert(key) {
        edges.push(GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            kind: kind.to_string(),
        });
    }
}

fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if !content.starts_with("---") {
        return map;
    }
    let rest = &content[3..];
    let Some(end) = rest.find("---") else {
        return map;
    };
    for line in rest[..end].lines() {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() && !val.is_empty() {
                map.insert(key, val);
            }
        }
    }
    map
}

pub fn frontmatter_list_from_content(content: &str, key: &str) -> Vec<String> {
    if !content.starts_with("---") {
        return Vec::new();
    }
    let rest = &content[3..];
    let Some(end) = rest.find("---") else {
        return Vec::new();
    };
    let block = &rest[..end];
    let mut in_section = false;
    let mut items = Vec::new();
    for line in block.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == key {
                in_section = true;
                let val = v.trim();
                if !val.is_empty() {
                    if val.starts_with('[') {
                        // inline yaml list — skip, handle below via - lines only
                    } else {
                        items.push(val.trim_matches('"').to_string());
                    }
                }
                continue;
            } else if !line.trim_start().starts_with('-') {
                in_section = false;
            }
        }
        if in_section && line.trim_start().starts_with("- ") {
            items.push(line.trim()[2..].trim().to_string());
        }
    }
    items
}

fn parse_scenarios(content: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*\*\*Scenario(?::|\s+Outline:)\s*(.+?)\*\*").unwrap();
    re.captures_iter(content)
        .map(|caps| caps[1].trim().to_string())
        .collect()
}

fn extract_markdown_title(content: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
}

fn normalize_adr_ids(paths: &[String]) -> Vec<String> {
    paths.iter().filter_map(|p| adr_id_from_path(p)).collect()
}

fn normalize_adr_id(value: &str) -> String {
    adr_id_from_path(value).unwrap_or_else(|| value.to_string())
}

fn adr_id_from_path(path: &str) -> Option<String> {
    Regex::new(r"ADR-(\d{3})")
        .unwrap()
        .find(path)
        .map(|m| m.as_str().to_string())
}
