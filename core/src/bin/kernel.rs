fn main() {
    match noa_core::init() {
        Ok(_) => println!("Kernel initialized."),
        Err(err) => eprintln!("Kernel initialization failed: {err}"),
    }
}
