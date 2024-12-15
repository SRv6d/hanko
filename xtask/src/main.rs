use clap::{self, CommandFactory, ValueEnum};
use hanko::cli::Cli;
use std::{env, error::Error, path::PathBuf};

const MAN_DEFAULT_CONFIG: &str = "~/.config/hanko/config.toml";

fn main() -> Result<(), Box<dyn Error>> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("completions") => {
            let output_dir = env::args().nth(2).expect("output directory not specified");
            create_completions(output_dir.into())?
        }
        Some("manpages") => {
            let output_dir =
                PathBuf::from(env::args().nth(2).expect("output directory not specified"));
            create_manpages(output_dir)?
        }
        _ => print_help(),
    }
    Ok(())
}

/// Create command completions for all shells supported by clap_complete.
fn create_completions(dir: clap::builder::OsStr) -> Result<(), Box<dyn Error>> {
    let mut cmd = Cli::command();
    for &shell in clap_complete::Shell::value_variants() {
        clap_complete::generate_to(shell, &mut cmd, "hanko", &dir)?;
    }

    Ok(())
}

/// Create manpages for all commands and subcommands.
fn create_manpages(dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut cmd = Cli::command()
        // Since manpages are static, but some default values are adjusted to the user environment at
        // runtime, we set appropriate static values here.
        .mut_arg("config", |a| a.default_value(MAN_DEFAULT_CONFIG))
        .mut_arg("file", |a| a.default_value(None));

    clap_mangen::generate_to(cmd, dir)?;

    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

manpages            generate manpages
completions         generate shell completions
"
    )
}
