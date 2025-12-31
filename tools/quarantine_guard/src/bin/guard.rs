use quarantine_guard::guard;

fn main() {
    if let Err(err) = guard::run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
