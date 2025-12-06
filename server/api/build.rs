fn main() {
    tonic_build::configure()
        .build_client(false)
        .compile(&["proto/noa_api.proto"], &["proto"])
        .expect("failed to compile noa api protos");
}
