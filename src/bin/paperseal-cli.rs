use std::{env, io::{self, Write}, path::Path};

fn main() {
    let mut args = env::args().skip(1);
    let Some(dest) = args.next() else {
        eprintln!("Usage: paperseal-cli <output.pdf> [pages]");
        std::process::exit(2);
    };
    let pages = match args.next() {
        Some(value) => match value.parse::<usize>() { Ok(n) => n, Err(_) => { eprintln!("Pages must be an integer from 1 to 100"); std::process::exit(2); } },
        None => 1,
    };
    if args.next().is_some() {
        eprintln!("Usage: paperseal-cli <output.pdf> [pages]");
        std::process::exit(2);
    }
    if Path::new(&dest).exists() {
        eprint!("Warning: file '{dest}' already exists and will be overwritten. Continue? [y/N] ");
        let _ = io::stderr().flush();
        let mut answer = String::new();
        if io::stdin().read_line(&mut answer).is_err() || !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            eprintln!("Cancelled; existing file was kept.");
            std::process::exit(1);
        }
    }
    match paperseal::generate_pages(&dest, pages) {
        Ok(ids) => println!("Created {} page(s) in {dest}\nPage IDs: {}\nPrint at 100% / Actual Size; use a stamp in each marked box.", ids.len(), ids.join(", ")),
        Err(error) => { eprintln!("Could not create PDF: {error}"); std::process::exit(1); }
    }
}
