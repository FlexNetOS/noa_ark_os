use noa_plugin_sdk::registry::{
    ApiSurface, AutomationSurface, BudgetSpec, CliMapping, NetworkClass, OutputSpec, ParameterSpec,
    RestSurface, SideEffectSpec, ToolDescriptor,
};
use serde_json::json;

/// Return ToolDescriptor entries for core agent-tools capabilities.
pub fn agent_tool_descriptors() -> Vec<ToolDescriptor> {
    let mut out = Vec::new();
    // Helper closures
    let budgets = || BudgetSpec {
        cpu_millis: 200,
        memory_mebibytes: 64,
        storage_mebibytes: 0,
        max_duration_seconds: 30,
        network_class: NetworkClass::Offline,
    };
    let automation = || AutomationSurface {
        events: vec![],
        approvals_required: vec![],
        runbooks: vec![],
    };
    let mk_api = |method: &str, path: &str| ApiSurface {
        grpc: "".to_string(),
        rest: RestSurface {
            method: method.to_string(),
            path: path.to_string(),
            operation_id: path.trim_start_matches('/').replace('/', "_"),
        },
    };

    // run_command
    out.push(ToolDescriptor {
        id: "run_command".into(),
        name: "Run Command".into(),
        category: "execution".into(),
        layer: "workspace".into(),
        summary: "Execute an allowlisted command with arguments".into(),
        description:
            "Runs a single command from the allowlist with optional timeout and environment variables."
                .into(),
        owner: "tools-agent".into(),
        maturity: "beta".into(),
        lifecycle: "active".into(),
        aliases: vec!["exec".into()],
        parameters: vec![
            ParameterSpec {
                name: "command".into(),
                description: "Bare executable name".into(),
                data_type: "string".into(),
                required: true,
                default: None,
                example: Some("cargo".into()),
                source: None,
            },
            ParameterSpec {
                name: "args".into(),
                description: "List of arguments".into(),
                data_type: "string[]".into(),
                required: false,
                default: None,
                example: Some("['--version']".into()),
                source: None,
            },
            ParameterSpec {
                name: "timeout_seconds".into(),
                description: "Max execution time".into(),
                data_type: "int".into(),
                required: false,
                default: Some("30".into()),
                example: Some("10".into()),
                source: None,
            },
        ],
        side_effects: vec![SideEffectSpec {
            impact: "cpu".into(),
            scope: "process".into(),
            description: "May spawn short-lived process".into(),
            mitigation: Some("enforced timeout".into()),
        }],
        budgets: budgets(),
        outputs: vec![OutputSpec {
            format: "json".into(),
            description: "Command result".into(),
            schema: Some(
                json!({
                    "type": "object",
                    "properties": {
                        "exit_code": { "type": "integer" },
                        "stdout": { "type": "string" },
                        "stderr": { "type": "string" }
                    }
                })
                .to_string(),
            ),
        }],
        cli_mappings: vec![CliMapping {
            command: vec!["noa-tools-agent".into(), "run".into()],
            description: "Invoke run_command".into(),
            arguments: vec!["--cmd".into()],
            flags: vec!["--timeout".into()],
            example: Some("noa-tools-agent run --cmd cargo -- --version".into()),
        }],
        api: mk_api("POST", "/run_command"),
        automation: automation(),
    });

    // edit_file
    out.push(ToolDescriptor {
        id: "edit_file".into(),
        name: "Edit File".into(),
        category: "filesystem".into(),
        layer: "workspace".into(),
        summary: "Write contents to a file".into(),
        description: "Creates or overwrites a file at a relative path.".into(),
        owner: "tools-agent".into(),
        maturity: "beta".into(),
        lifecycle: "active".into(),
        aliases: vec!["write_file".into()],
        parameters: vec![
            ParameterSpec {
                name: "path".into(),
                description: "Relative file path".into(),
                data_type: "string".into(),
                required: true,
                default: None,
                example: Some("server/README.md".into()),
                source: None,
            },
            ParameterSpec {
                name: "contents".into(),
                description: "Full file contents".into(),
                data_type: "string".into(),
                required: true,
                default: None,
                example: Some("Hello".into()),
                source: None,
            },
        ],
        side_effects: vec![SideEffectSpec {
            impact: "write".into(),
            scope: "filesystem".into(),
            description: "Overwrites file on disk".into(),
            mitigation: Some("archival pre-write".into()),
        }],
        budgets: budgets(),
        outputs: vec![OutputSpec {
            format: "json".into(),
            description: "Write result".into(),
            schema: None,
        }],
        cli_mappings: vec![CliMapping {
            command: vec!["noa-tools-agent".into(), "edit".into()],
            description: "Edit file".into(),
            arguments: vec!["--path".into()],
            flags: vec![],
            example: None,
        }],
        api: mk_api("POST", "/edit_file"),
        automation: automation(),
    });

    // list_files
    out.push(ToolDescriptor {
        id: "list_files".into(),
        name: "List Files".into(),
        category: "filesystem".into(),
        layer: "workspace".into(),
        summary: "Enumerate directory entries".into(),
        description: "Lists non-recursive entries in a directory.".into(),
        owner: "tools-agent".into(),
        maturity: "beta".into(),
        lifecycle: "active".into(),
        aliases: vec!["ls".into()],
        parameters: vec![ParameterSpec {
            name: "path".into(),
            description: "Directory path".into(),
            data_type: "string".into(),
            required: false,
            default: Some(".".into()),
            example: Some("server".into()),
            source: None,
        }],
        side_effects: vec![],
        budgets: budgets(),
        outputs: vec![OutputSpec {
            format: "json".into(),
            description: "File entries".into(),
            schema: None,
        }],
        cli_mappings: vec![CliMapping {
            command: vec!["noa-tools-agent".into(), "ls".into()],
            description: "List files".into(),
            arguments: vec!["--path".into()],
            flags: vec![],
            example: None,
        }],
        api: mk_api("POST", "/list_files"),
        automation: automation(),
    });

    // read_file
    out.push(ToolDescriptor {
        id: "read_file".into(),
        name: "Read File".into(),
        category: "filesystem".into(),
        layer: "workspace".into(),
        summary: "Return file contents".into(),
        description: "Reads an entire UTF-8 file.".into(),
        owner: "tools-agent".into(),
        maturity: "beta".into(),
        lifecycle: "active".into(),
        aliases: vec!["cat".into()],
        parameters: vec![ParameterSpec {
            name: "path".into(),
            description: "File path".into(),
            data_type: "string".into(),
            required: true,
            default: None,
            example: Some("README.md".into()),
            source: None,
        }],
        side_effects: vec![],
        budgets: budgets(),
        outputs: vec![OutputSpec {
            format: "json".into(),
            description: "File read result".into(),
            schema: None,
        }],
        cli_mappings: vec![CliMapping {
            command: vec!["noa-tools-agent".into(), "read".into()],
            description: "Read file".into(),
            arguments: vec!["--path".into()],
            flags: vec![],
            example: None,
        }],
        api: mk_api("POST", "/read_file"),
        automation: automation(),
    });

    // run_tests
    out.push(ToolDescriptor {
        id: "run_tests".into(),
        name: "Run Tests".into(),
        category: "execution".into(),
        layer: "workspace".into(),
        summary: "Execute test suite".into(),
        description: "Runs the workspace test suite (cargo test).".into(),
        owner: "tools-agent".into(),
        maturity: "alpha".into(),
        lifecycle: "active".into(),
        aliases: vec!["tests".into()],
        parameters: vec![],
        side_effects: vec![SideEffectSpec {
            impact: "cpu".into(),
            scope: "process".into(),
            description: "May spawn compiler jobs".into(),
            mitigation: Some("resource budgets".into()),
        }],
        budgets: budgets(),
        outputs: vec![OutputSpec {
            format: "json".into(),
            description: "Test run summary".into(),
            schema: None,
        }],
        cli_mappings: vec![CliMapping {
            command: vec!["noa-tools-agent".into(), "test".into()],
            description: "Run tests".into(),
            arguments: vec![],
            flags: vec![],
            example: None,
        }],
        api: mk_api("POST", "/run_tests"),
        automation: automation(),
    });

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_have_required_ids() {
        let list = agent_tool_descriptors();
        let mut ids: Vec<_> = list.iter().map(|d| d.id.as_str()).collect();
        ids.sort();
        let expected = [
            "edit_file",
            "list_files",
            "read_file",
            "run_command",
            "run_tests",
        ];
        assert_eq!(ids, expected);
        for d in list {
            assert!(!d.summary.is_empty());
            assert!(!d.description.is_empty());
        }
    }
}
