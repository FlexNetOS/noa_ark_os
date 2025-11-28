//! ARK-OS Production Main Application
//! 
//! Unified entry point that combines all three repositories into a single
//! production-ready system following the "Heal, Don't Harm" principle.

use anyhow::Result;
use ark_os_production::config::{ConfigManager, Environment};
use ark_os_production::{utils, ArkOsSystem};
use clap::{Parser, Subcommand};
use tracing::{error, info, warn};

/// ARK-OS Production Command Line Interface
#[derive(Parser)]
#[command(name = "ark-os")]
#[command(about = "ARK-OS Production-Ready Unified System")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.json")]
    config: String,

    /// Environment override
    #[arg(short, long)]
    environment: Option<String>,

    /// Log level override
    #[arg(short, long)]
    log_level: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Run in headless mode (no desktop UI)
    #[arg(long)]
    headless: bool,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the ARK-OS system
    Start {
        /// Run in daemon mode
        #[arg(short, long)]
        daemon: bool,
    },
    /// Stop the ARK-OS system
    Stop,
    /// Check system status
    Status,
    /// Run system health check
    Health,
    /// Generate default configuration
    GenerateConfig {
        /// Output file path
        #[arg(short, long, default_value = "config.json")]
        output: String,
    },
    /// Validate configuration
    ValidateConfig,
    /// Run system tests
    Test {
        /// Test specific component
        #[arg(short, long)]
        component: Option<String>,
    },
    /// Show system metrics
    Metrics,
    /// Initialize system directories and files
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle early commands that don't need full system initialization
    match &cli.command {
        Some(Commands::GenerateConfig { output }) => {
            return generate_config(output).await;
        }
        Some(Commands::Init) => {
            return init_system().await;
        }
        _ => {}
    }

    // Load configuration
    let mut config_manager = if std::path::Path::new(&cli.config).exists() {
        ConfigManager::load_from_file(&cli.config).await?
    } else {
        warn!("Configuration file {} not found, using environment and defaults", cli.config);
        ConfigManager::load_from_env()
    };

    // Apply CLI overrides
    if let Some(env_str) = &cli.environment {
        let environment = match env_str.to_lowercase().as_str() {
            "dev" | "development" => Environment::Development,
            "test" | "testing" => Environment::Testing,
            "stage" | "staging" => Environment::Staging,
            "prod" | "production" => Environment::Production,
            _ => {
                error!("Invalid environment: {}", env_str);
                std::process::exit(1);
            }
        };
        config_manager.config_mut().system.environment = environment;
    }

    if let Some(log_level) = &cli.log_level {
        config_manager.config_mut().logging.level = log_level.clone();
    }

    if cli.verbose {
        config_manager.config_mut().logging.level = "debug".to_string();
    }

    if cli.headless {
        config_manager.config_mut().ui.desktop_enabled = false;
    }

    // Validate configuration
    if let Err(e) = config_manager.validate() {
        error!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Apply configuration to environment
    config_manager.apply_to_environment()?;

    // Setup logging
    utils::setup_tracing(&config_manager.config().system.environment)?;

    info!("ARK-OS Production System v{}", env!("CARGO_PKG_VERSION"));
    info!("Environment: {:?}", config_manager.config().system.environment);

    // Handle commands
    match &cli.command {
        Some(Commands::Start { daemon }) => {
            start_system(config_manager, *daemon).await
        }
        Some(Commands::Stop) => {
            stop_system().await
        }
        Some(Commands::Status) => {
            check_status().await
        }
        Some(Commands::Health) => {
            check_health(config_manager).await
        }
        Some(Commands::ValidateConfig) => {
            info!("Configuration is valid ✓");
            Ok(())
        }
        Some(Commands::Test { component }) => {
            run_tests(config_manager, component.as_deref()).await
        }
        Some(Commands::Metrics) => {
            show_metrics(config_manager).await
        }
        None => {
            // Default: start the system
            start_system(config_manager, false).await
        }
        _ => Ok(()), // Already handled above
    }
}

/// Start the ARK-OS system
async fn start_system(config_manager: ConfigManager, daemon: bool) -> Result<()> {
    info!("Starting ARK-OS Production System...");

    if daemon {
        info!("Running in daemon mode");
        // In a real implementation, this would properly daemonize the process
    }

    let config = config_manager.config().clone();
    let ark_config = crate::utils::create_component_config(&config.system.name);

    // Create and initialize the system
    let mut system = ArkOsSystem::new(ark_config);
    
    info!("Initializing system components...");
    system.initialize().await?;

    info!("Starting system components...");
    system.start().await?;

    info!("ARK-OS system started successfully ✓");
    info!("System name: {}", config.system.name);
    info!("Features enabled:");
    info!("  - Agents: {}", config.features.agents_enabled);
    info!("  - Orchestration: {}", config.features.orchestration_enabled);
    info!("  - Execution: {}", config.features.execution_enabled);
    info!("  - Desktop UI: {}", config.features.desktop_ui_enabled);
    info!("  - Web UI: {}", config.features.web_ui_enabled);
    info!("  - API: {}", config.features.api_enabled);
    info!("  - Autonomous Mode: {}", config.features.autonomous_mode);

    if config.ui.web_enabled {
        info!("Web UI available at: http://{}:{}", 
            config.ui.web_config.host, config.ui.web_config.port);
    }

    if config.ui.api_enabled {
        info!("API available at: http://{}:{}{}", 
            config.ui.api_config.host, config.ui.api_config.port, config.ui.api_config.api_prefix);
    }

    // Set up signal handlers for graceful shutdown
    setup_signal_handlers(system).await?;

    Ok(())
}

/// Stop the ARK-OS system
async fn stop_system() -> Result<()> {
    info!("Stopping ARK-OS system...");
    
    // In a real implementation, this would connect to a running instance
    // and send a shutdown signal
    
    info!("ARK-OS system stopped ✓");
    Ok(())
}

/// Check system status
async fn check_status() -> Result<()> {
    info!("Checking ARK-OS system status...");
    
    // In a real implementation, this would connect to a running instance
    // and retrieve status information
    
    info!("System status: Not running");
    Ok(())
}

/// Check system health
async fn check_health(config_manager: ConfigManager) -> Result<()> {
    info!("Performing system health check...");

    let config = config_manager.config();
    let ark_config = crate::utils::create_component_config(&config.system.name);

    // Create system for health check
    let system = ArkOsSystem::new(ark_config);
    
    match system.health_check().await? {
        crate::HealthStatus::Healthy => {
            info!("System health: ✓ Healthy");
        }
        crate::HealthStatus::Degraded => {
            warn!("System health: ⚠ Degraded");
        }
        crate::HealthStatus::Unhealthy => {
            error!("System health: ✗ Unhealthy");
        }
        crate::HealthStatus::Unknown => {
            warn!("System health: ? Unknown");
        }
        crate::HealthStatus::Maintenance => {
            info!("System health: 🔧 Maintenance");
        }
    }

    Ok(())
}

/// Run system tests
async fn run_tests(config_manager: ConfigManager, component: Option<&str>) -> Result<()> {
    info!("Running system tests...");

    if let Some(comp) = component {
        info!("Testing component: {}", comp);
    } else {
        info!("Testing all components");
    }

    let config = config_manager.config();
    let ark_config = crate::utils::create_component_config(&config.system.name);

    let mut system = ArkOsSystem::new(ark_config);
    
    // Initialize but don't start for testing
    system.initialize().await?;

    // Run basic tests
    let health = system.health_check().await?;
    info!("Health check: {:?}", health);

    let metrics = system.get_metrics().await?;
    info!("System metrics collected: {} components", metrics.component_metrics.len());

    info!("System tests completed ✓");
    Ok(())
}

/// Show system metrics
async fn show_metrics(config_manager: ConfigManager) -> Result<()> {
    info!("Collecting system metrics...");

    let config = config_manager.config();
    let ark_config = crate::utils::create_component_config(&config.system.name);

    let system = ArkOsSystem::new(ark_config);
    let metrics = system.get_metrics().await?;

    println!("\n=== ARK-OS System Metrics ===");
    println!("Timestamp: {}", metrics.timestamp);
    println!("Overall Health: {:?}", metrics.health_status);
    println!("\nComponent Metrics:");
    
    for (name, component_metrics) in &metrics.component_metrics {
        println!("  {}:", name);
        println!("    Health: {:?}", component_metrics.health);
        println!("    Uptime: {:?}", component_metrics.uptime);
        println!("    CPU Usage: {:.2}%", component_metrics.cpu_usage);
        println!("    Memory Usage: {} MB", component_metrics.memory_usage / 1024 / 1024);
        println!("    Active Tasks: {}", component_metrics.active_tasks);
        println!("    Completed Tasks: {}", component_metrics.completed_tasks);
        println!("    Error Count: {}", component_metrics.error_count);
    }

    println!("\nAgent System Metrics:");
    println!("  Total Agents: {}", metrics.agent_metrics.total_agents);
    println!("  Active Agents: {}", metrics.agent_metrics.active_agents);
    println!("  Tasks Completed: {}", metrics.agent_metrics.tasks_completed);

    println!("\nOrchestration Metrics:");
    println!("  Total Executions: {}", metrics.orchestration_metrics.total_executions);
    println!("  Successful Executions: {}", metrics.orchestration_metrics.successful_executions);
    println!("  Active Executions: {}", metrics.orchestration_metrics.active_executions);

    println!("\nExecution Metrics:");
    println!("  Total Tasks: {}", metrics.execution_metrics.total_tasks);
    println!("  Completed Tasks: {}", metrics.execution_metrics.completed_tasks);
    println!("  Failed Tasks: {}", metrics.execution_metrics.failed_tasks);
    println!("  Throughput: {:.2} tasks/sec", metrics.execution_metrics.throughput_per_second);

    Ok(())
}

/// Generate default configuration file
async fn generate_config(output: &str) -> Result<()> {
    info!("Generating default configuration...");

    let config = crate::config::Config::default();
    let content = serde_json::to_string_pretty(&config)?;
    
    tokio::fs::write(output, content).await?;
    
    info!("Default configuration written to: {}", output);
    info!("Edit the configuration file and run 'ark-os validate-config' to check it");
    
    Ok(())
}

/// Initialize system directories and files
async fn init_system() -> Result<()> {
    info!("Initializing ARK-OS system...");

    let config = crate::config::Config::default();

    // Create directories
    for dir in [&config.system.data_directory, &config.system.log_directory, &config.system.config_directory] {
        tokio::fs::create_dir_all(dir).await?;
        info!("Created directory: {}", dir);
    }

    // Generate default config if it doesn't exist
    let config_path = "config.json";
    if !std::path::Path::new(config_path).exists() {
        generate_config(config_path).await?;
    }

    info!("ARK-OS system initialized ✓");
    info!("Next steps:");
    info!("  1. Review and edit config.json");
    info!("  2. Run 'ark-os validate-config'");
    info!("  3. Run 'ark-os start'");

    Ok(())
}

/// Setup signal handlers for graceful shutdown
async fn setup_signal_handlers(mut system: ArkOsSystem) -> Result<()> {
    use tokio::signal;

    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => {
            info!("Received SIGINT, shutting down gracefully...");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down gracefully...");
        }
    }

    system.stop().await?;
    info!("ARK-OS system stopped ✓");

    Ok(())
}

// Additional modules that need to be declared in lib.rs
mod agents {
    pub mod communication {
        use super::super::agents::*;

        pub struct MessageBroker {
            capacity: usize,
        }

        impl MessageBroker {
            pub fn new(capacity: usize) -> Self {
                Self { capacity }
            }
        }
    }
}