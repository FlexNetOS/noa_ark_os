fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let capnp_path = std::path::Path::new(&manifest_dir)
        .join("..").join("..").join("contracts").join("inference.capnp");
    println!("cargo:rerun-if-changed={}", capnp_path.display());
    capnpc::CompilerCommand::new()
        .file(capnp_path.to_str().unwrap())
        .run()
        .expect("capnp compile failed");
}
