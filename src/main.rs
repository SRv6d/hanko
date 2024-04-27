use clap::Parser;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use hanko::{cli::Cli, Config};

fn main() {
    let cli = Cli::parse();
    let _config: Config = Figment::from(Serialized::defaults(Config::default()))
        .admerge(Toml::file(cli.config))
        .extract()
        .unwrap();
}
