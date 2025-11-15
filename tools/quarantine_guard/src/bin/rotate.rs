use quarantine_guard::rotation;

fn main() {
    if let Err(err) = rotation::run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
