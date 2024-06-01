fn main() {
    hanko::cli::entrypoint().unwrap_or_else(|err| eprintln!("error: {err}"))
}
