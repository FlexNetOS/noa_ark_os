fn main() {
    if let Err(err) = noa_core::init() {
        eprintln!("Kernel initialization failed: {err}");
    }
}
