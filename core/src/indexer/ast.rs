use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::indexer::{should_skip, IndexerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstGraph {
    pub generated_at: u128,
    pub nodes: Vec<AstNode>,
    pub edges: Vec<AstEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub id: String,
    pub path: String,
    pub functions: usize,
    pub structs: usize,
    pub enums: usize,
    pub traits: usize,
    pub impls: usize,
    pub dependencies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

impl AstGraph {
    pub fn build(root: impl AsRef<Path>) -> Result<Self, IndexerError> {
        let root = root.as_ref();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(root)
                .unwrap_or(entry.path())
                .to_path_buf();
            if should_skip(&relative) {
                continue;
            }
            let node = build_node(entry.path(), relative, &mut edges)?;
            nodes.push(node);
        }

        Ok(Self {
            generated_at: crate::utils::current_timestamp_millis(),
            nodes,
            edges,
        })
    }
}

fn build_node(
    path: &Path,
    relative: PathBuf,
    edges: &mut Vec<AstEdge>,
) -> Result<AstNode, IndexerError> {
    let source = fs::read_to_string(path)?;
    let syntax = syn::parse_file(&source).map_err(|err| {
        // Attach file path to make upstream kernel errors actionable when parsing fails.
        syn::Error::new(err.span(), format!("{}: {}", path.display(), err))
    })?;
    let module_id = relative
        .to_string_lossy()
        .replace('\\', "/")
        .replace(".rs", "");
    let mut functions = 0;
    let mut structs = 0;
    let mut enums = 0;
    let mut traits = 0;
    let mut impls = 0;
    let mut dependencies = 0;

    for item in syntax.items {
        match item {
            syn::Item::Fn(_) => functions += 1,
            syn::Item::Struct(_) => structs += 1,
            syn::Item::Enum(_) => enums += 1,
            syn::Item::Trait(_) => traits += 1,
            syn::Item::Impl(_) => impls += 1,
            syn::Item::Use(item_use) => {
                let targets = flatten_use_tree(&item_use.tree);
                dependencies += targets.len();
                for target in targets {
                    edges.push(AstEdge {
                        from: module_id.clone(),
                        to: target,
                        relation: "use".into(),
                    });
                }
            }
            syn::Item::Mod(item_mod) => {
                let name = item_mod.ident.to_string();
                edges.push(AstEdge {
                    from: module_id.clone(),
                    to: format!("{}::{}", module_id, name),
                    relation: "module".into(),
                });
            }
            _ => {}
        }
    }

    Ok(AstNode {
        id: module_id.clone(),
        path: relative.to_string_lossy().to_string(),
        functions,
        structs,
        enums,
        traits,
        impls,
        dependencies,
    })
}

fn flatten_use_tree(tree: &syn::UseTree) -> Vec<String> {
    // Iterative approach to avoid stack overflow on deep/circular use trees
    use syn::UseTree;
    let mut result = Vec::new();
    // Stack holds (current tree, accumulated path prefix)
    let mut stack = vec![(tree, String::new())];
    while let Some((node, prefix)) = stack.pop() {
        match node {
            UseTree::Path(path) => {
                let new_prefix = if prefix.is_empty() {
                    path.ident.to_string()
                } else {
                    format!("{}::{}", prefix, path.ident)
                };
                stack.push((&path.tree, new_prefix));
            }
            UseTree::Name(name) => {
                let full = if prefix.is_empty() {
                    name.ident.to_string()
                } else {
                    format!("{}::{}", prefix, name.ident)
                };
                result.push(full);
            }
            UseTree::Rename(rename) => {
                let full = if prefix.is_empty() {
                    rename.ident.to_string()
                } else {
                    format!("{}::{}", prefix, rename.ident)
                };
                result.push(full);
            }
            UseTree::Glob(_) => {
                let full = if prefix.is_empty() {
                    "*".to_string()
                } else {
                    format!("{}::*", prefix)
                };
                result.push(full);
            }
            UseTree::Group(group) => {
                for item in group.items.iter().rev() {
                    stack.push((item, prefix.clone()));
                }
            }
        }
    }
    result
}
