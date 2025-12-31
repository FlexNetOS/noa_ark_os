use std::env;

fn main() {
    // Set build time for version tracking
    println!("cargo:rustc-env=BUILD_TIME={}", std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string()));
    
    // Generate protobuf files if they exist
    if std::path::Path::new("proto").exists() {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile(&[], &["proto"])
            .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));
    }
}