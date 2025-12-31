//! Kernel-managed runtime bootstrapper.
//!
//! The runtime manager is responsible for activating language runtimes defined
//! in the kernel manifest. Each runtime is modeled as a plugin with explicit
//! dependencies so that startup ordering is deterministic and reproducible.

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;

use crate::capabilities::{CapabilityError, CapabilityResult};
use crate::config::manifest::{RuntimeKind, RuntimeManifestEntry};
use crate::kernel::{self, AiControlLoop, MachineRemediationDirective};
use crate::metrics::AggregatedTelemetry;

/// Runtime plugin state tracked by the kernel.
#[derive(Debug, Clone)]
pub struct RuntimePlugin {
    pub name: String,
    pub kind: RuntimeKind,
    pub version: String,
    pub entrypoint: String,
    pub depends_on: Vec<String>,
    pub assets: Vec<String>,
    pub status: RuntimeStatus,
}

/// Lifecycle phases of a runtime plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Registered,
    Bootstrapped,
    Running,
}

impl RuntimePlugin {
    fn from_manifest(entry: &RuntimeManifestEntry) -> Self {
        Self {
            name: entry.name.clone(),
            kind: entry.kind,
            version: entry.version.clone(),
            entrypoint: entry.entrypoint.clone(),
            depends_on: entry.depends_on.clone(),
            assets: entry.assets.clone(),
            status: RuntimeStatus::Registered,
        }
    }
}

/// Manages runtime plugins declared in the kernel manifest.
#[derive(Debug, Default)]
pub struct RuntimeManager {
    plugins: RwLock<HashMap<String, RuntimePlugin>>,
    boot_order: RwLock<Vec<String>>,
}

/// Execution policy derived from kernel telemetry for runtime schedulers.
#[derive(Debug, Clone)]
pub struct MachineExecutionPolicy {
    pub directive: MachineRemediationDirective,
    pub telemetry: Option<AggregatedTelemetry>,
    pub active_runtimes: Vec<RuntimePlugin>,
}

impl MachineExecutionPolicy {
    pub fn prefer_machine(&self) -> bool {
        self.directive.prefer_machine()
    }
}

/// Trait bridging runtime managers with the kernel control loop interface.
pub trait RuntimeControlLoop {
    fn machine_execution_policy(&self) -> MachineExecutionPolicy;
}

impl RuntimeControlLoop for RuntimeManager {
    fn machine_execution_policy(&self) -> MachineExecutionPolicy {
        let (directive, telemetry) = if let Some(handle) = kernel::handle() {
            let snapshot = handle.agent_health_snapshot();
            (snapshot.directive(), snapshot.telemetry)
        } else {
            (MachineRemediationDirective::default(), None)
        };

        let active_runtimes = self
            .all_runtimes()
            .into_iter()
            .filter(|runtime| runtime.status == RuntimeStatus::Running)
            .collect();

        MachineExecutionPolicy {
            directive,
            telemetry,
            active_runtimes,
        }
    }
}

impl RuntimeManager {
    /// Construct a runtime manager from manifest entries.
    pub fn from_manifest(entries: &[RuntimeManifestEntry]) -> CapabilityResult<Self> {
        let mut plugins = HashMap::new();
        for entry in entries {
            if plugins.contains_key(&entry.name) {
                return Err(CapabilityError::ManifestError(format!(
                    "duplicate runtime name {} in manifest",
                    entry.name
                )));
            }
            plugins.insert(entry.name.clone(), RuntimePlugin::from_manifest(entry));
        }
        Ok(Self {
            plugins: RwLock::new(plugins),
            boot_order: RwLock::new(Vec::new()),
        })
    }

    /// Boot all runtimes respecting dependency ordering.
    pub fn bootstrap(&self) -> CapabilityResult<()> {
        let order = self.compute_boot_order()?;
        {
            let mut boot_order = self.boot_order.write().unwrap();
            *boot_order = order.clone();
        }

        for runtime_name in order {
            {
                let mut plugins = self.plugins.write().unwrap();
                if let Some(runtime) = plugins.get_mut(&runtime_name) {
                    runtime.status = match runtime.status {
                        RuntimeStatus::Registered => RuntimeStatus::Bootstrapped,
                        RuntimeStatus::Bootstrapped | RuntimeStatus::Running => runtime.status,
                    };
                }
            }

            println!("[RUNTIME] Bootstrapping plugin: {runtime_name}");

            let mut plugins = self.plugins.write().unwrap();
            if let Some(runtime) = plugins.get_mut(&runtime_name) {
                runtime.status = RuntimeStatus::Running;
            }
        }

        Ok(())
    }

    /// Retrieve a runtime plugin definition.
    pub fn runtime(&self, name: &str) -> CapabilityResult<RuntimePlugin> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .get(name)
            .cloned()
            .ok_or_else(|| CapabilityError::ManifestError(format!("runtime {name} not found")))
    }

    /// Snapshot all registered runtimes.
    pub fn all_runtimes(&self) -> Vec<RuntimePlugin> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    /// Report the runtime boot order computed from manifest dependencies.
    pub fn boot_order(&self) -> Vec<String> {
        self.boot_order.read().unwrap().clone()
    }

    fn compute_boot_order(&self) -> CapabilityResult<Vec<String>> {
        let plugins = self.plugins.read().unwrap();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for plugin in plugins.values() {
            in_degree.entry(plugin.name.clone()).or_insert(0);
            for dependency in &plugin.depends_on {
                if !plugins.contains_key(dependency) {
                    return Err(CapabilityError::ManifestError(format!(
                        "runtime {} depends on missing runtime {}",
                        plugin.name, dependency
                    )));
                }
                *in_degree.entry(plugin.name.clone()).or_insert(0) += 1;
                adjacency
                    .entry(dependency.clone())
                    .or_default()
                    .push(plugin.name.clone());
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter_map(|(runtime, degree)| {
                if *degree == 0 {
                    Some(runtime.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut ordered = Vec::new();
        while let Some(runtime) = queue.pop_front() {
            ordered.push(runtime.clone());
            if let Some(children) = adjacency.get(&runtime) {
                for child in children {
                    if let Some(degree) = in_degree.get_mut(child) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(child.clone());
                        }
                    }
                }
            }
        }

        if ordered.len() != plugins.len() {
            return Err(CapabilityError::DependencyCycle(
                plugins.keys().cloned().collect(),
            ));
        }

        Ok(ordered)
    }
}
