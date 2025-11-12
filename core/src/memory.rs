//! Memory management subsystem

use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED_MEMORY: AtomicUsize = AtomicUsize::new(0);

/// Initialize memory management
pub fn init() -> Result<(), &'static str> {
    println!("[MEMORY] Initializing memory manager...");
    Ok(())
}

fn allocate_inner(size: usize) {
    ALLOCATED_MEMORY.fetch_add(size, Ordering::SeqCst);
}

fn deallocate_inner(size: usize) {
    ALLOCATED_MEMORY.fetch_sub(size, Ordering::SeqCst);
}

fn allocated_inner() -> usize {
    ALLOCATED_MEMORY.load(Ordering::SeqCst)
}

/// Capability wrapper for kernel-managed memory operations.
#[derive(Clone, Default)]
pub struct MemoryManager;

impl MemoryManager {
    /// Reserve memory pages from the global allocator tracking.
    pub fn allocate(&self, size: usize) -> Result<(), &'static str> {
        allocate_inner(size);
        Ok(())
    }

    /// Release memory back to the allocator tracking.
    pub fn deallocate(&self, size: usize) -> Result<(), &'static str> {
        deallocate_inner(size);
        Ok(())
    }

    /// Query total allocated memory.
    pub fn total_allocated(&self) -> usize {
        allocated_inner()
    }
}

/// Track memory allocation.
pub fn allocate(size: usize) -> Result<(), &'static str> {
    MemoryManager::default().allocate(size)
}

/// Track memory deallocation.
pub fn deallocate(size: usize) -> Result<(), &'static str> {
    MemoryManager::default().deallocate(size)
}

/// Get total allocated memory.
pub fn get_allocated() -> usize {
    MemoryManager::default().total_allocated()
}
