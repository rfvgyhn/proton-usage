use clap::Parser;
use proton_usage::parse_steam_config;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version, about)]
struct Opts {
    /// Path to the Steam home directory. Default: ~/.steam
    #[clap(short, long)]
    steam_path: Option<PathBuf>,

    /// Output verbosity (-v, -vv, -vvv, etc)
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(opts.verbose + 1)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    let steam_path = opts
        .steam_path
        .or_else(|| dirs::home_dir().map(|home| home.join(".steam")))
        .ok_or("Couldn't find Steam directory")?;

    let config = parse_steam_config(&steam_path)?;
    println!("{}", &config);
    Ok(())
}
