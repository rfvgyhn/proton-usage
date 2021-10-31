use clap::Parser;
use proton_usage::parse_steam_config;
use std::path::PathBuf;
use tokio;

#[derive(Parser)]
#[clap(version, about)]
struct Opts {
    /// Path to the config.vdf file
    #[clap(short, long)]
    config_path: Option<PathBuf>,

    /// Output verbosity (-v, -vv, -vvv, etc)
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(opts.verbose + 1)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    let config_path = opts
        .config_path
        .or_else(|| dirs::home_dir().map(|home| home.join(".steam/root/config/config.vdf")))
        .ok_or("Couldn't find Steam root config")?;

    let config = parse_steam_config(&config_path).await?;
    println!("{}", &config);
    Ok(())
}
