use clap::{self, CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use hanko::cli::Cli;
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("completions") => {
            let output_dir = env::args().nth(2).expect("output directory not specified");
            create_completions(output_dir.into())?
        }
        _ => print_help(),
    }
    Ok(())
}

/// Create command completions for all shells supported by clap_complete.
fn create_completions(dir: clap::builder::OsStr) -> Result<(), Box<dyn Error>> {
    let mut cmd = Cli::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "hanko", &dir)?;
    }

    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

completions         generate shell completions
"
    )
}
