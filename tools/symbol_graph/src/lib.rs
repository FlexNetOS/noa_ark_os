use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use noa_core::symbols::stable_symbol_id;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tree_sitter::{Language, Node, Parser};
use walkdir::WalkDir;

pub mod notebook;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("unsupported language for path {0}")]
    UnsupportedLanguage(String),
    #[error("parser error: {0}")]
    Parser(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolNode {
    pub stable_id: String,
    pub language: String,
    pub name: String,
    pub kind: String,
    pub file: String,
    pub signature: String,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SymbolGraph {
    pub nodes: BTreeMap<String, SymbolNode>,
    pub edges: Vec<SymbolEdge>,
}

impl SymbolGraph {
    pub fn find(&self, stable_id: &str) -> Option<&SymbolNode> {
        self.nodes.get(stable_id)
    }

    pub fn edges_from(&self, stable_id: &str) -> impl Iterator<Item = &SymbolEdge> {
        let target = stable_id.to_string();
        self.edges.iter().filter(move |edge| edge.from == target)
    }

    pub fn load(store_root: impl AsRef<Path>) -> Result<Self, GraphError> {
        let root = store_root.as_ref();
        let nodes_path = root.join("nodes.jsonl");
        let edges_path = root.join("edges.jsonl");
        let mut graph = SymbolGraph::default();

        if nodes_path.exists() {
            let reader = BufReader::new(std::fs::File::open(&nodes_path)?);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let node: SymbolNode = serde_json::from_str(&line)?;
                graph.nodes.insert(node.stable_id.clone(), node);
            }
        }

        if edges_path.exists() {
            let reader = BufReader::new(std::fs::File::open(&edges_path)?);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let edge: SymbolEdge = serde_json::from_str(&line)?;
                graph.edges.push(edge);
            }
        }

        Ok(graph)
    }
}

pub struct SymbolGraphBuilder {
    root: PathBuf,
    store_root: PathBuf,
    nodes: HashMap<String, SymbolNode>,
    edges: Vec<SymbolEdge>,
}

impl SymbolGraphBuilder {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let store_root = root.join(".workspace").join("indexes").join("symbol_graph");
        Self {
            root,
            store_root,
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn with_store_root(mut self, store: impl Into<PathBuf>) -> Self {
        self.store_root = store.into();
        self
    }

    pub fn index(mut self) -> Result<SymbolGraph, GraphError> {
        for entry in WalkDir::new(&self.root) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if let Err(err) = self.index_file(path) {
                eprintln!(
                    "[symbol-graph] skipping {} due to error: {}",
                    path.display(),
                    err
                );
            }
        }
        self.persist()?;
        SymbolGraph::load(&self.store_root)
    }

    pub fn index_file(&mut self, path: &Path) -> Result<(), GraphError> {
        let Some((language_id, language)) = language_for(path) else {
            self.record_generic_file(path)?;
            return Ok(());
        };
        let source = fs::read_to_string(path)?;
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .map_err(|err| GraphError::Parser(err.to_string()))?;
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| GraphError::Parser("failed to parse source".into()))?;
        let cursor = tree.walk();
        let mut stack = vec![cursor.clone()];

        while let Some(mut current) = stack.pop() {
            let node = current.node();
            collect_symbol(
                node,
                &source,
                path,
                language_id,
                &mut self.nodes,
                &mut self.edges,
            )?;
            if current.goto_first_child() {
                loop {
                    stack.push(current.clone());
                    if !current.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn persist(&self) -> Result<(), GraphError> {
        fs::create_dir_all(&self.store_root)?;
        let mut graph = SymbolGraph::load(&self.store_root).unwrap_or_default();
        for (id, node) in &self.nodes {
            graph.nodes.insert(id.clone(), node.clone());
        }

        let mut edge_set: BTreeSet<(String, String, String)> = graph
            .edges
            .iter()
            .map(|edge| (edge.from.clone(), edge.to.clone(), edge.kind.clone()))
            .collect();
        for edge in &self.edges {
            edge_set.insert((edge.from.clone(), edge.to.clone(), edge.kind.clone()));
        }
        graph.edges = edge_set
            .into_iter()
            .map(|(from, to, kind)| SymbolEdge { from, to, kind })
            .collect();

        let nodes_path = self.store_root.join("nodes.jsonl");
        let edges_path = self.store_root.join("edges.jsonl");
        write_jsonl(&nodes_path, graph.nodes.values())?;
        write_jsonl(&edges_path, graph.edges.iter())?;
        Ok(())
    }

    fn record_generic_file(&mut self, path: &Path) -> Result<(), GraphError> {
        let relative = relative_file(path);
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");
        let language = format!("generic::{extension}");
        let signature = format!("file://{relative}");
        let stable_id = stable_symbol_id(&language, &relative, "file", &signature);
        self.nodes.entry(stable_id.clone()).or_insert(SymbolNode {
            stable_id,
            language,
            name: relative.clone(),
            kind: "file".to_string(),
            file: relative,
            signature,
            span: (1, 1),
        });
        Ok(())
    }
}

fn write_jsonl<'a, I, T>(path: &Path, items: I) -> Result<(), GraphError>
where
    I: IntoIterator<Item = &'a T>,
    T: Serialize + 'a,
{
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    for item in items {
        let line = serde_json::to_string(item)?;
        writeln!(file, "{}", line)?;
    }
    file.flush()?;
    Ok(())
}

fn relative_file(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&cwd) {
            return relative.to_string_lossy().to_string();
        }
    }
    path.to_string_lossy().to_string()
}

fn language_for(path: &Path) -> Option<(&'static str, Language)> {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "rs" => Some(("rust", tree_sitter_rust::language())),
        "ts" | "tsx" => Some(("typescript", tree_sitter_typescript::language_typescript())),
        _ => None,
    }
}

fn collect_symbol(
    node: Node,
    source: &str,
    path: &Path,
    language_id: &str,
    nodes: &mut HashMap<String, SymbolNode>,
    edges: &mut Vec<SymbolEdge>,
) -> Result<(), GraphError> {
    match language_id {
        "rust" => collect_rust_symbol(node, source, path, nodes),
        "typescript" => collect_typescript_symbol(node, source, path, nodes),
        _ => Ok(()),
    }?;

    if node.kind() == "call_expression" {
        if let Some(name) = extract_identifier(language_id, node, source) {
            // Find the enclosing function/method node and use its stable_id as the caller
            if let Some((enclosing_name, enclosing_kind, enclosing_signature)) =
                find_enclosing_function(node, source, language_id)
            {
                let caller = stable_symbol_id(
                    language_id,
                    &enclosing_name,
                    &enclosing_kind,
                    &enclosing_signature,
                );
                let callee_id = stable_symbol_id(language_id, &name, "call", &name);
                edges.push(SymbolEdge {
                    from: caller,
                    to: callee_id,
                    kind: "call".to_string(),
                });
            }
        }
    }

    Ok(())
}

fn collect_rust_symbol(
    node: Node,
    source: &str,
    path: &Path,
    nodes: &mut HashMap<String, SymbolNode>,
) -> Result<(), GraphError> {
    let kind = match node.kind() {
        "function_item" => "function",
        "struct_item" => "struct",
        _ => return Ok(()),
    };

    let name = extract_identifier("rust", node, source).unwrap_or_else(|| "anonymous".into());
    let signature = normalise_signature("rust", node, source);
    let stable_id = stable_symbol_id("rust", &name, kind, &signature);
    let file = relative_file(path);
    nodes.insert(
        stable_id.clone(),
        SymbolNode {
            stable_id,
            language: "rust".to_string(),
            name,
            kind: kind.to_string(),
            file,
            signature,
            span: (node.start_position().row + 1, node.end_position().row + 1),
        },
    );
    Ok(())
}

fn collect_typescript_symbol(
    node: Node,
    source: &str,
    path: &Path,
    nodes: &mut HashMap<String, SymbolNode>,
) -> Result<(), GraphError> {
    let kind = match node.kind() {
        "function_declaration" => "function",
        "class_declaration" => "class",
        _ => return Ok(()),
    };

    let name = extract_identifier("typescript", node, source).unwrap_or_else(|| "anonymous".into());
    let signature = normalise_signature("typescript", node, source);
    let stable_id = stable_symbol_id("typescript", &name, kind, &signature);
    let file = relative_file(path);
    nodes.insert(
        stable_id.clone(),
        SymbolNode {
            stable_id,
            language: "typescript".to_string(),
            name,
            kind: kind.to_string(),
            file,
            signature,
            span: (node.start_position().row + 1, node.end_position().row + 1),
        },
    );
    Ok(())
}

fn find_enclosing_function(
    node: Node,
    source: &str,
    _language: &str,
) -> Option<(String, String, String)> {
    let mut current = node;
    loop {
        if let Some(parent) = current.parent() {
            match parent.kind() {
                "function_definition"
                | "function_declaration"
                | "method_definition"
                | "arrow_function" => {
                    if let Some(name) = extract_identifier("rust", parent, source) {
                        let signature = normalise_signature("rust", parent, source);
                        let kind = parent.kind().to_string();
                        return Some((name, kind, signature));
                    }
                }
                _ => {}
            }
            current = parent;
        } else {
            return None;
        }
    }
}

fn extract_identifier(language: &str, node: Node, source: &str) -> Option<String> {
    match language {
        "rust" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                }
            }
            None
        }
        "typescript" => node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn normalise_signature(language: &str, node: Node, source: &str) -> String {
    match language {
        "rust" => {
            let mut params = Vec::new();
            if let Some(param_list) = node.child_by_field_name("parameters") {
                let mut cursor = param_list.walk();
                for child in param_list.children(&mut cursor) {
                    params.push(
                        child
                            .utf8_text(source.as_bytes())
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    );
                }
            }
            params.join(",")
        }
        "typescript" => {
            let mut params = Vec::new();
            if let Some(param_list) = node.child_by_field_name("parameters") {
                let mut cursor = param_list.walk();
                for child in param_list.children(&mut cursor) {
                    params.push(
                        child
                            .utf8_text(source.as_bytes())
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    );
                }
            }
            params.join(",")
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn builds_symbol_graph_for_rust_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("lib.rs");
        fs::write(
            &file,
            r#"pub fn hello(name: &str) -> String { format!(\"Hello {}\", name) }"#,
        )
        .unwrap();

        let builder = SymbolGraphBuilder::new(dir.path());
        let graph = builder.index().unwrap();
        assert!(!graph.nodes.is_empty());
        let node = graph
            .nodes
            .values()
            .find(|node| node.name == "hello")
            .unwrap();
        assert_eq!(node.kind, "function");
    }

    #[test]
    fn stable_ids_survive_file_moves() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("nested/b.rs");
        fs::create_dir_all(file_b.parent().unwrap()).unwrap();
        let source = "pub fn compute(value: i32) -> i32 { value + 1 }";
        fs::write(&file_a, source).unwrap();
        fs::write(&file_b, source).unwrap();

        let builder_a = SymbolGraphBuilder::new(dir.path());
        let graph_a = builder_a.index().unwrap();
        let id_a = graph_a
            .nodes
            .values()
            .find(|node| node.name == "compute")
            .unwrap()
            .stable_id
            .clone();

        let nodes = dir
            .path()
            .join(".workspace/indexes/symbol_graph/nodes.jsonl");
        let edges = dir
            .path()
            .join(".workspace/indexes/symbol_graph/edges.jsonl");
        if nodes.exists() {
            fs::remove_file(&nodes).unwrap();
        }
        if edges.exists() {
            fs::remove_file(&edges).unwrap();
        }

        let builder_b = SymbolGraphBuilder::new(dir.path());
        let graph_b = builder_b.index().unwrap();
        let id_b = graph_b
            .nodes
            .values()
            .find(|node| node.name == "compute")
            .unwrap()
            .stable_id
            .clone();

        assert_eq!(id_a, id_b);
    }
}
