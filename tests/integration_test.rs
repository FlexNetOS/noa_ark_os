//! Integration test for NOA ARK OS components

#[cfg(test)]
mod integration_tests {
    use noa_core;
    
    #[test]
    fn test_full_system_init() {
        // Test core initialization
        let result = noa_core::init();
        assert!(result.is_ok(), "Core OS should initialize successfully");
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
        noa_core::process::init().unwrap();
        let pid = noa_core::process::create_process("test_process".to_string()).unwrap();
        let process = noa_core::process::get_process(pid);
        assert!(process.is_some(), "Process should exist");
    }
}
