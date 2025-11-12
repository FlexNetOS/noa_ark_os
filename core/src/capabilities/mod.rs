//! Capability registry and kernel-facing handles.
//!
//! The capability system exposes every core subsystem as a kernel-managed
//! resource. Higher-level modules can request these resources dynamically
//! instead of importing concrete implementations directly.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::config::manifest::{CapabilityManifestEntry, KernelManifest};

/// Result alias for capability operations.
pub type CapabilityResult<T> = Result<T, CapabilityError>;

/// Dynamic capability object stored inside the registry.
pub type DynCapability = Arc<dyn Any + Send + Sync>;

/// Errors emitted by the capability registry.
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("capability already registered: {0}")]
    AlreadyRegistered(String),
    #[error("capability not registered: {0}")]
    UnknownCapability(String),
    #[error("capability not initialized: {0}")]
    NotInitialized(String),
    #[error("initialization failed for capability {0}: {1}")]
    InitializationFailed(String, String),
    #[error("shutdown failed for capability {0}: {1}")]
    ShutdownFailed(String, String),
    #[error("dependency cycle detected: {0:?}")]
    DependencyCycle(Vec<String>),
    #[error("manifest error: {0}")]
    ManifestError(String),
}

/// Lifecycle states tracked for each capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapabilityState {
    Registered,
    Initializing,
    Ready,
    Failed,
}

/// Definition describing how to create and teardown a capability.
pub struct CapabilityDefinition {
    id: String,
    dependencies: Vec<String>,
    initializer: Arc<dyn Fn(&CapabilityContext) -> CapabilityResult<DynCapability> + Send + Sync>,
    shutdown: Option<
        Arc<dyn Fn(&CapabilityContext, DynCapability) -> CapabilityResult<()> + Send + Sync>,
    >,
    description: Option<String>,
}

impl CapabilityDefinition {
    /// Begin building a new capability definition.
    pub fn builder(id: impl Into<String>) -> CapabilityDefinitionBuilder {
        CapabilityDefinitionBuilder {
            id: id.into(),
            dependencies: Vec::new(),
            description: None,
            initializer: None,
            shutdown: None,
        }
    }

    /// Identifier of the capability.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Additional description for diagnostics.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Builder utility for [`CapabilityDefinition`].
pub struct CapabilityDefinitionBuilder {
    id: String,
    dependencies: Vec<String>,
    description: Option<String>,
    initializer:
        Option<Arc<dyn Fn(&CapabilityContext) -> CapabilityResult<DynCapability> + Send + Sync>>,
    shutdown: Option<
        Arc<dyn Fn(&CapabilityContext, DynCapability) -> CapabilityResult<()> + Send + Sync>,
    >,
}

impl CapabilityDefinitionBuilder {
    /// Document runtime dependencies enforced in addition to the manifest.
    pub fn depends_on(mut self, deps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.dependencies = deps.into_iter().map(Into::into).collect();
        self
    }

    /// Provide a human-readable description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Register the initializer responsible for creating the capability handle.
    pub fn init_with<F>(mut self, initializer: F) -> Self
    where
        F: Fn(&CapabilityContext) -> CapabilityResult<DynCapability> + Send + Sync + 'static,
    {
        self.initializer = Some(Arc::new(initializer));
        self
    }

    /// Register a shutdown hook invoked during kernel teardown.
    pub fn shutdown_with<F>(mut self, shutdown: F) -> Self
    where
        F: Fn(&CapabilityContext, DynCapability) -> CapabilityResult<()> + Send + Sync + 'static,
    {
        self.shutdown = Some(Arc::new(shutdown));
        self
    }

    /// Finalize the capability definition.
    pub fn build(self) -> CapabilityDefinition {
        CapabilityDefinition {
            id: self.id,
            dependencies: self.dependencies,
            initializer: self
                .initializer
                .expect("capability definitions require an initializer"),
            shutdown: self.shutdown,
            description: self.description,
        }
    }
}

struct RegisteredCapability {
    definition: Arc<CapabilityDefinition>,
    state: CapabilityState,
    instance: Option<DynCapability>,
}

/// Context provided to capability hooks during lifecycle changes.
#[derive(Clone)]
pub struct CapabilityContext {
    kernel: KernelHandle,
    capability_id: String,
}

impl CapabilityContext {
    fn new(kernel: KernelHandle, capability_id: impl Into<String>) -> Self {
        Self {
            kernel,
            capability_id: capability_id.into(),
        }
    }

    /// Access the kernel handle for requesting other capabilities.
    pub fn kernel(&self) -> &KernelHandle {
        &self.kernel
    }

    /// Identifier of the capability being processed.
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    /// Access the kernel manifest currently in effect.
    pub fn manifest(&self) -> &KernelManifest {
        self.kernel.manifest()
    }
}

/// Registry storing capability metadata and lifecycle state.
pub struct CapabilityRegistry {
    entries: RwLock<HashMap<String, RegisteredCapability>>,
    init_order: Mutex<Vec<String>>,
}

impl CapabilityRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            init_order: Mutex::new(Vec::new()),
        }
    }

    /// Register a new capability definition.
    pub fn register_definition(&self, definition: CapabilityDefinition) -> CapabilityResult<()> {
        let mut entries = self.entries.write().unwrap();
        let id = definition.id().to_string();
        if entries.contains_key(&id) {
            return Err(CapabilityError::AlreadyRegistered(id));
        }
        entries.insert(
            id,
            RegisteredCapability {
                definition: Arc::new(definition),
                state: CapabilityState::Registered,
                instance: None,
            },
        );
        Ok(())
    }

    /// Ensure the capability identified by `id` is initialized.
    pub fn ensure_initialized(&self, id: &str, kernel: &KernelHandle) -> CapabilityResult<()> {
        let definition = {
            let mut entries = self.entries.write().unwrap();
            let entry = entries
                .get_mut(id)
                .ok_or_else(|| CapabilityError::UnknownCapability(id.to_string()))?;
            match entry.state {
                CapabilityState::Ready => return Ok(()),
                CapabilityState::Initializing => {
                    return Err(CapabilityError::DependencyCycle(vec![id.to_string()]))
                }
                CapabilityState::Failed => {
                    return Err(CapabilityError::InitializationFailed(
                        id.to_string(),
                        "previous initialization attempt failed".to_string(),
                    ))
                }
                CapabilityState::Registered => {
                    entry.state = CapabilityState::Initializing;
                    Arc::clone(&entry.definition)
                }
            }
        };

        let mut all_dependencies = definition.dependencies.clone();
        if let Some(manifest_entry) = kernel.manifest().capability(id) {
            for dependency in &manifest_entry.depends_on {
                if !all_dependencies.contains(dependency) {
                    all_dependencies.push(dependency.clone());
                }
            }
        }

        for dependency in all_dependencies {
            self.ensure_initialized(&dependency, kernel)?;
        }

        let context = CapabilityContext::new(kernel.clone(), definition.id.clone());
        let instance = match (definition.initializer)(&context) {
            Ok(instance) => instance,
            Err(err) => {
                let mut entries = self.entries.write().unwrap();
                if let Some(entry) = entries.get_mut(id) {
                    entry.state = CapabilityState::Failed;
                }
                let message = err.to_string();
                return Err(CapabilityError::InitializationFailed(
                    id.to_string(),
                    message,
                ));
            }
        };

        {
            let mut entries = self.entries.write().unwrap();
            if let Some(entry) = entries.get_mut(id) {
                entry.instance = Some(instance);
                entry.state = CapabilityState::Ready;
            }
        }

        {
            let mut order = self.init_order.lock().unwrap();
            if !order.contains(&definition.id) {
                order.push(definition.id.clone());
            }
        }

        Ok(())
    }

    /// Retrieve an initialized capability instance.
    pub fn instance(&self, id: &str) -> CapabilityResult<DynCapability> {
        let entries = self.entries.read().unwrap();
        let entry = entries
            .get(id)
            .ok_or_else(|| CapabilityError::UnknownCapability(id.to_string()))?;
        entry
            .instance
            .as_ref()
            .cloned()
            .ok_or_else(|| CapabilityError::NotInitialized(id.to_string()))
    }

    /// Shut down all capabilities in reverse initialization order.
    pub fn shutdown_all(&self, kernel: &KernelHandle) -> CapabilityResult<()> {
        let order = self.init_order.lock().unwrap().clone();
        for capability_id in order.into_iter().rev() {
            let shutdown_hook = {
                let entries = self.entries.read().unwrap();
                let Some(entry) = entries.get(&capability_id) else {
                    continue;
                };
                entry
                    .definition
                    .shutdown
                    .as_ref()
                    .map(|hook| (Arc::clone(hook), entry.instance.clone()))
            };

            if let Some((hook, instance)) = shutdown_hook {
                if let Some(instance) = instance {
                    let context = CapabilityContext::new(kernel.clone(), capability_id.clone());
                    if let Err(err) = (hook)(&context, instance) {
                        let message = err.to_string();
                        return Err(CapabilityError::ShutdownFailed(capability_id, message));
                    }
                }
            }
        }
        Ok(())
    }

    /// Initialize every capability flagged for autostart in the manifest.
    pub fn initialize_autostart(&self, kernel: &KernelHandle) -> CapabilityResult<()> {
        for entry in kernel
            .manifest()
            .capabilities
            .iter()
            .filter(|cap| cap.autostart)
        {
            self.ensure_initialized(&entry.id, kernel)?;
        }
        Ok(())
    }

    /// Register a manifest entry that is not explicitly defined yet.
    pub fn declare_manifest_capability(
        &self,
        manifest_entry: &CapabilityManifestEntry,
    ) -> CapabilityResult<()> {
        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(&manifest_entry.id) {
            return Ok(());
        }
        entries.insert(
            manifest_entry.id.clone(),
            RegisteredCapability {
                definition: Arc::new(
                    CapabilityDefinition::builder(&manifest_entry.id)
                        .depends_on(manifest_entry.depends_on.clone())
                        .description("manifest-declared capability placeholder")
                        .init_with(|_| {
                            Err(CapabilityError::ManifestError(
                                "no provider registered for capability".to_string(),
                            ))
                        })
                        .build(),
                ),
                state: CapabilityState::Failed,
                instance: None,
            },
        );
        Err(CapabilityError::ManifestError(format!(
            "no provider registered for capability {}",
            manifest_entry.id
        )))
    }
}

/// Handle exposed to higher layers for requesting capabilities.
#[derive(Clone)]
pub struct KernelHandle {
    registry: Arc<CapabilityRegistry>,
    manifest: Arc<KernelManifest>,
}

impl KernelHandle {
    /// Create a new handle from the registry and manifest.
    pub fn new(registry: Arc<CapabilityRegistry>, manifest: Arc<KernelManifest>) -> Self {
        Self { registry, manifest }
    }

    /// Access the kernel manifest.
    pub fn manifest(&self) -> &KernelManifest {
        &self.manifest
    }

    /// Ensure a capability and its dependencies are initialized.
    pub fn ensure(&self, id: &str) -> CapabilityResult<()> {
        self.registry.ensure_initialized(id, self)
    }

    /// Request a capability handle of type `T`.
    pub fn request<T>(&self, id: &str) -> CapabilityResult<Arc<T>>
    where
        T: Any + Send + Sync + 'static,
    {
        self.ensure(id)?;
        let instance = self.registry.instance(id)?;
        Arc::downcast::<T>(instance).map_err(|_| {
            CapabilityError::ManifestError(format!("capability {id} has unexpected type"))
        })
    }

    /// Shutdown all capabilities registered in the kernel.
    pub fn shutdown(&self) -> CapabilityResult<()> {
        self.registry.shutdown_all(self)
    }

    /// Access to the underlying registry for dynamic extensions.
    pub fn registry(&self) -> &Arc<CapabilityRegistry> {
        &self.registry
    }
}

/// Built-in capability registrations.
pub mod builtin;
