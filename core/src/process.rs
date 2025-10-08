//! Process management subsystem

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ProcessId = u64;

#[derive(Debug, Clone)]
pub struct Process {
    pub id: ProcessId,
    pub name: String,
    pub state: ProcessState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

lazy_static::lazy_static! {
    static ref PROCESS_TABLE: Arc<Mutex<HashMap<ProcessId, Process>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

static mut NEXT_PID: ProcessId = 1;

/// Initialize process management
pub fn init() -> Result<(), &'static str> {
    println!("[PROCESS] Initializing process manager...");
    Ok(())
}

/// Create a new process
pub fn create_process(name: String) -> Result<ProcessId, &'static str> {
    let pid = unsafe {
        let pid = NEXT_PID;
        NEXT_PID += 1;
        pid
    };
    
    let process = Process {
        id: pid,
        name,
        state: ProcessState::Ready,
    };
    
    let mut table = PROCESS_TABLE.lock().unwrap();
    table.insert(pid, process);
    
    Ok(pid)
}

/// Get process by ID
pub fn get_process(pid: ProcessId) -> Option<Process> {
    let table = PROCESS_TABLE.lock().unwrap();
    table.get(&pid).cloned()
}
