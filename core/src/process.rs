//! Process management subsystem

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, OnceLock,
};

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

static PROCESS_TABLE: OnceLock<Mutex<HashMap<ProcessId, Process>>> = OnceLock::new();
static NEXT_PID: AtomicU64 = AtomicU64::new(1);

fn process_table() -> &'static Mutex<HashMap<ProcessId, Process>> {
    PROCESS_TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Initialize process management
pub fn init() -> Result<(), &'static str> {
    println!("[PROCESS] Initializing process manager...");
    Ok(())
}

fn create_process_inner(name: String) -> Result<ProcessId, &'static str> {
    let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);

    let process = Process {
        id: pid,
        name,
        state: ProcessState::Ready,
    };

    let mut table = process_table().lock().unwrap();
    table.insert(pid, process);

    Ok(pid)
}

fn get_process_inner(pid: ProcessId) -> Option<Process> {
    let table = process_table().lock().unwrap();
    table.get(&pid).cloned()
}

/// Capability handle wrapping process-management operations.
#[derive(Clone, Default)]
pub struct ProcessService;

impl ProcessService {
    /// Create a new process through the kernel-managed capability.
    pub fn create_process(&self, name: String) -> Result<ProcessId, &'static str> {
        create_process_inner(name)
    }

    /// Fetch a process record by identifier.
    pub fn get_process(&self, pid: ProcessId) -> Option<Process> {
        get_process_inner(pid)
    }

    /// List all tracked processes.
    pub fn list_processes(&self) -> Vec<Process> {
        let table = process_table().lock().unwrap();
        table.values().cloned().collect()
    }
}

/// Create a new process.
pub fn create_process(name: String) -> Result<ProcessId, &'static str> {
    ProcessService::default().create_process(name)
}

/// Get process by ID.
pub fn get_process(pid: ProcessId) -> Option<Process> {
    ProcessService::default().get_process(pid)
}
