fn main() {
    if let Err(error) = now::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
