//! Integration test for NOA ARK OS components

#[cfg(test)]
mod integration_tests {
    use noa_core;
    use std::path::Path;
    
    #[test]
    fn test_full_system_init() {
        let result = noa_core::init();
        assert!(result.is_ok(), "Core OS should initialize successfully");
        noa_core::kernel::shutdown();
    }

    #[test]
    fn test_kernel_running() {
        noa_core::kernel::init().unwrap();
        assert!(noa_core::kernel::is_running(), "Kernel should be running");
        noa_core::kernel::shutdown();
        assert!(!noa_core::kernel::is_running(), "Kernel should be stopped");
    }
    
    #[test]
    fn test_process_management() {
        let handle = noa_core::init().unwrap();
        let process_service = handle
            .request::<noa_core::process::ProcessService>(
                noa_core::config::manifest::CAPABILITY_PROCESS,
            )
            .unwrap();
        let pid = process_service
            .create_process("test_process".to_string())
            .unwrap();
        let process = process_service.get_process(pid);
        assert!(process.is_some(), "Process should exist");
        noa_core::kernel::shutdown();
    }

    #[test]
    fn test_indexer_outputs_are_persisted() {
        let _ = noa_core::init().unwrap();
        let indexes = Path::new(".workspace/indexes");
        assert!(indexes.join("ast_graph.json").exists());
        assert!(indexes.join("ownership_graph.json").exists());
        assert!(indexes.join("config_graph.json").exists());
        noa_core::kernel::shutdown();
    }
}
