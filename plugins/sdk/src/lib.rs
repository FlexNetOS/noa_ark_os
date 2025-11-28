//! NOA Ark OS Plugin SDK
//!
//! Provides utilities for inspecting the shared tool registry so plugins and
//! automation runtimes can dynamically discover capabilities exposed by the
//! `noa` CLI, REST, and gRPC surfaces.

pub mod registry;

pub use registry::{
    ApiSurface, AutomationSurface, BudgetSpec, CliMapping, NetworkClass, OutputSpec, ParameterSpec,
    RestSurface, SideEffectSpec, ToolDescriptor, ToolRegistry,
};
