fn main() {
    tonic_build::configure()
        .compile(&["proto/ui_schema.proto"], &["proto"])
        .expect("failed to compile ui schema proto");
}
