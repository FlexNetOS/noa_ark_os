use std::env;
use std::process;

use noa_core::world::{Reconciler, WorldGraph, WorldGraphError};

fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => process::exit(1),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}

fn run() -> Result<bool, WorldGraphError> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("verify");

    let graph = WorldGraph::load_default()?;
    let reconciler = Reconciler::new(graph);
    let repo_root = WorldGraph::repo_root();
    let report = reconciler.diff(&repo_root);

    match command {
        "verify" => handle_verify(report),
        "fix" => handle_fix(report),
        "plan" => handle_plan(report),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(true)
        }
        _ => {
            eprintln!("unknown command: {command}");
            print_help();
            Ok(false)
        }
    }
}

fn handle_verify(report: noa_core::world::ReconciliationReport) -> Result<bool, WorldGraphError> {
    if report.is_clean() {
        println!("✅ world graph is in sync with the repository");
        Ok(true)
    } else {
        println!(
            "❌ world graph drift detected ({} issues).",
            report.drifts.len()
        );
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        Ok(false)
    }
}

fn handle_fix(report: noa_core::world::ReconciliationReport) -> Result<bool, WorldGraphError> {
    if report.is_clean() {
        println!("✅ no drift detected – nothing to fix");
        return Ok(true);
    }

    println!(
        "🛠️ remediation plan for {} drift entries:",
        report.drifts.len()
    );
    for (index, step) in report.remediation.iter().enumerate() {
        println!(
            "{}. [{}] {} → {}",
            index + 1,
            step.action,
            step.relative_path,
            step.description
        );
    }
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    Ok(false)
}

fn handle_plan(report: noa_core::world::ReconciliationReport) -> Result<bool, WorldGraphError> {
    if report.remediation.is_empty() {
        println!("✅ world graph is in sync – no remediation steps required");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&report.remediation).unwrap()
        );
    }
    Ok(report.is_clean())
}

fn print_help() {
    println!("noa-world – world graph reconciler");
    println!("Usage: noa_world <command>");
    println!("Commands:");
    println!("  verify   Validate the repository against the world graph");
    println!("  fix      Show remediation plan for any detected drift");
    println!("  plan     Output remediation steps as JSON");
}
