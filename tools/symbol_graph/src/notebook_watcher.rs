use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde_json::to_string_pretty;

use noa_symbol_graph::notebook::NotebookMetadataDiff;
use noa_symbol_graph::{SymbolGraph, SymbolGraphBuilder};

fn main() -> Result<()> {
    let (root, once) = parse_args(env::args().skip(1));
    if once {
        run_once(&root)
    } else {
        run_watch(root)
    }
}

fn parse_args<I>(args: I) -> (PathBuf, bool)
where
    I: IntoIterator<Item = String>,
{
    let mut root = PathBuf::from(".");
    let mut once = false;
    for arg in args {
        if arg == "--once" {
            once = true;
        } else if arg.starts_with("--") {
            eprintln!("[symbol-graph] ignoring unknown flag {arg}");
        } else {
            root = PathBuf::from(arg);
        }
    }
    (root, once)
}

fn run_once(root: &Path) -> Result<()> {
    let mut previous = load_existing_graph(root)?;
    let new_graph = rebuild_graph(root)?;
    let diff = NotebookMetadataDiff::from_graphs(&previous, &new_graph);
    if diff.has_changes() {
        write_diff(root, &diff)?;
    }
    previous = new_graph;
    persist_state(root, &previous)?;
    Ok(())
}

fn run_watch(root: PathBuf) -> Result<()> {
    let mut previous = load_existing_graph(&root)?;
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .with_context(|| "failed to start filesystem watcher")?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", root.display()))?;

    for event in rx {
        match event {
            Ok(event) => {
                if should_process(&event) {
                    if let Err(err) = handle_event(&root, &mut previous) {
                        eprintln!("[symbol-graph] watcher error: {err}");
                    }
                }
            }
            Err(err) => eprintln!("[symbol-graph] watcher channel error: {err}"),
        }
    }
    Ok(())
}

fn handle_event(root: &Path, previous: &mut SymbolGraph) -> Result<()> {
    let new_graph = rebuild_graph(root)?;
    let diff = NotebookMetadataDiff::from_graphs(previous, &new_graph);
    if diff.has_changes() {
        write_diff(root, &diff)?;
    }
    *previous = new_graph;
    persist_state(root, previous)?;
    Ok(())
}

fn should_process(event: &Event) -> bool {
    if matches!(event.kind, EventKind::Other) {
        return false;
    }
    if matches!(event.kind, EventKind::Modify(ModifyKind::Metadata(_))) {
        return false;
    }
    event
        .paths
        .iter()
        .any(|path| is_relevant_file(path) && !is_workspace_path(path))
}

fn is_relevant_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "ts" | "tsx")
    )
}

fn is_workspace_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some(".workspace" | "target" | "node_modules")
        )
    })
}

fn rebuild_graph(root: &Path) -> Result<SymbolGraph> {
    SymbolGraphBuilder::new(root)
        .index()
        .with_context(|| format!("failed to rebuild symbol graph for {}", root.display()))
}

fn load_existing_graph(root: &Path) -> Result<SymbolGraph> {
    let store_root = symbol_store_root(root);
    Ok(SymbolGraph::load(&store_root).unwrap_or_default())
}

fn persist_state(root: &Path, graph: &SymbolGraph) -> Result<()> {
    let store_root = symbol_store_root(root);
    std::fs::create_dir_all(&store_root).with_context(|| {
        format!(
            "failed to create symbol graph store at {}",
            store_root.display()
        )
    })?;
    let nodes_path = store_root.join("notebook_state.json");
    std::fs::write(&nodes_path, to_string_pretty(graph)?).with_context(|| {
        format!(
            "failed to record notebook state at {}",
            nodes_path.display()
        )
    })
}

fn write_diff(root: &Path, diff: &NotebookMetadataDiff) -> Result<PathBuf> {
    let diff_root = root.join(".workspace").join("notebook_sync").join("diffs");
    std::fs::create_dir_all(&diff_root)
        .with_context(|| format!("failed to create diff directory at {}", diff_root.display()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut candidate = diff_root.join(format!("diff-{timestamp}.json"));
    let mut counter = 0u32;
    while candidate.exists() {
        counter += 1;
        candidate = diff_root.join(format!("diff-{timestamp}-{counter}.json"));
    }
    std::fs::write(&candidate, to_string_pretty(diff)?).with_context(|| {
        format!(
            "failed to write notebook metadata diff to {}",
            candidate.display()
        )
    })?;
    Ok(candidate)
}

fn symbol_store_root(root: &Path) -> PathBuf {
    root.join(".workspace").join("indexes").join("symbol_graph")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_args_supports_root_and_once() {
        let (root, once) = parse_args(vec!["./workspace".into(), "--once".into()]);
        assert_eq!(root, PathBuf::from("./workspace"));
        assert!(once);
    }

    #[test]
    fn write_diff_creates_unique_files() {
        let temp = tempdir().unwrap();
        let diff = NotebookMetadataDiff::empty();
        let first = write_diff(temp.path(), &diff).unwrap();
        let second = write_diff(temp.path(), &diff).unwrap();
        assert!(first.exists());
        assert!(second.exists());
        assert_ne!(first, second);
    }

    #[test]
    fn handle_event_writes_diff_when_changes_detected() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn example() {}").unwrap();

        let mut previous = SymbolGraph::default();
        handle_event(root, &mut previous).unwrap();
        let diff_root = root.join(".workspace/notebook_sync/diffs");
        assert!(diff_root.exists());
        let entries: Vec<_> = fs::read_dir(diff_root).unwrap().collect();
        assert!(!entries.is_empty());
    }
}
