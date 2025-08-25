fn main() {
    if let Err(e) = kayton_repl::run_repl() {
        eprintln!("REPL error: {}", e);
        std::process::exit(1);
    }
}
