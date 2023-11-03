use clap::{Parser, Subcommand, ArgAction};
use proton_usage::{parse_launch_options, parse_tool_mapping};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Opts {
    /// Path to the Steam home directory. Default: ~/.steam
    #[clap(short, long)]
    steam_path: Option<PathBuf>,

    /// Output verbosity (-v, -vv, -vvv, etc)
    #[clap(short, long, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Lists apps with a specific compatibility tool (default)
    Proton,

    /// Lists apps with overridden launch options
    LaunchOptions
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity((opts.verbose + 1).into())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    let steam_path = opts
        .steam_path
        .or_else(|| dirs::home_dir().map(|home| home.join(".steam")))
        .ok_or("Couldn't find Steam directory")?;

    match &opts.command {
        None | Some(Command::Proton) => {
            let config = parse_tool_mapping(&steam_path)?;
            println!("{}", &config);
        }
        Some(Command::LaunchOptions) => {
            let config = parse_launch_options(&steam_path)?;
            println!("{}", &config);
        },
    };

    Ok(())
}
