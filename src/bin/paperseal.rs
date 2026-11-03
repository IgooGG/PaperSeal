fn main() {
    if let Err(error) = paperseal::gui::run() {
        eprintln!("Paperseal could not start: {error}");
        std::process::exit(1);
    }
}
