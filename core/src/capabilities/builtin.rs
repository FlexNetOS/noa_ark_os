//! Built-in capability registrations for the kernel.

use std::sync::Arc;

use super::{
    CapabilityDefinition, CapabilityError, CapabilityRegistry, CapabilityResult, DynCapability,
};
use crate::config::manifest::{
    CAPABILITY_FILESYSTEM, CAPABILITY_GATEWAY, CAPABILITY_IPC, CAPABILITY_MEMORY,
    CAPABILITY_PROCESS, CAPABILITY_RUNTIME_MANAGER, CAPABILITY_SECURITY,
};
use crate::fs::FileSystemService;
use crate::gateway::Gateway;
use crate::ipc::IpcService;
use crate::memory::MemoryManager;
use crate::process::ProcessService;
use crate::runtime::RuntimeManager;
use crate::security::SecurityService;

fn wrap_init_error(id: &str, err: impl ToString) -> CapabilityError {
    CapabilityError::InitializationFailed(id.to_string(), err.to_string())
}

/// Register all built-in capabilities with the provided registry.
pub fn register_default_capabilities(registry: &CapabilityRegistry) -> CapabilityResult<()> {
    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_PROCESS)
            .description("Process management subsystem")
            .init_with(|_| {
                crate::process::init().map_err(|err| wrap_init_error(CAPABILITY_PROCESS, err))?;
                Ok(Arc::new(ProcessService::default()) as DynCapability)
            })
            .build(),
    )?;

    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_MEMORY)
            .description("Memory management subsystem")
            .init_with(|_| {
                crate::memory::init().map_err(|err| wrap_init_error(CAPABILITY_MEMORY, err))?;
                Ok(Arc::new(MemoryManager::default()) as DynCapability)
            })
            .build(),
    )?;

    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_IPC)
            .description("Inter-process communication subsystem")
            .init_with(|_| {
                crate::ipc::init().map_err(|err| wrap_init_error(CAPABILITY_IPC, err))?;
                Ok(Arc::new(IpcService::default()) as DynCapability)
            })
            .build(),
    )?;

    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_FILESYSTEM)
            .description("File system subsystem")
            .init_with(|_| {
                crate::fs::init().map_err(|err| wrap_init_error(CAPABILITY_FILESYSTEM, err))?;
                Ok(Arc::new(FileSystemService::default()) as DynCapability)
            })
            .build(),
    )?;

    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_SECURITY)
            .description("Security subsystem")
            .init_with(|_| {
                crate::security::init().map_err(|err| wrap_init_error(CAPABILITY_SECURITY, err))?;
                Ok(Arc::new(SecurityService::default()) as DynCapability)
            })
            .build(),
    )?;

    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_GATEWAY)
            .description("Service gateway fabric")
            .depends_on([CAPABILITY_PROCESS, CAPABILITY_MEMORY, CAPABILITY_SECURITY])
            .init_with(|_| {
                let gateway = Gateway::default();
                gateway
                    .bootstrap_defaults()
                    .map_err(|err| wrap_init_error(CAPABILITY_GATEWAY, err))?;
                Ok(Arc::new(gateway) as DynCapability)
            })
            .build(),
    )?;

    registry.register_definition(
        CapabilityDefinition::builder(CAPABILITY_RUNTIME_MANAGER)
            .description("Language runtime manager")
            .depends_on([CAPABILITY_PROCESS, CAPABILITY_MEMORY, CAPABILITY_SECURITY])
            .init_with(|context| {
                let manifest = context.manifest();
                let manager = RuntimeManager::from_manifest(&manifest.runtimes)?;
                manager.bootstrap()?;
                Ok(Arc::new(manager) as DynCapability)
            })
            .build(),
    )?;

    Ok(())
}
