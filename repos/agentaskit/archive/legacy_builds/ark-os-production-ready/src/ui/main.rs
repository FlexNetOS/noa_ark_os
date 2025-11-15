use anyhow::Result;
use ark_os_core::{utils, ArkComponent};
use tauri_app::TauriApp;

fn main() -> Result<()> {
    // Initialize tracing
    utils::setup_tracing()?;
    
    // Create and initialize the Tauri app
    let mut app = TauriApp::new();
    let config = utils::create_component_config("tauri-desktop-app");
    
    app.initialize(config)?;
    
    // Check if we should run in headless mode for testing
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--headless".to_string()) {
        println!("Running in headless test mode...");
        app.run_headless_test()?;
        println!("Headless test completed successfully");
    } else {
        println!("Starting ARK-OS Desktop Application...");
        app.run()?;
    }
    
    Ok(())
}
