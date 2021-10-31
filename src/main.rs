use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fs;
use std::io::BufRead;
use std::path::Path;
use futures::{stream, StreamExt};
use reqwest::Client;
use tokio;
use serde::{Deserialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = dirs::home_dir()
        .map(|home| home.join(".steam/root/config/config.vdf"))
        .ok_or("Couldn't find Steam root config")?;

    println!("Parsing {}", config_path.display());
    if let Ok(config) = parse_steam_config(&config_path) {
        let ids = config.values().cloned().flatten().collect::<Vec<_>>();

        println!("Fetching app names");
        let names = fetch_app_names(&ids).await?;

        println!();
        print_config(&config, &names);
        Ok(())
    } else {
        Err(Box::try_from("Couldn't parse Steam root config").unwrap())
    }
}

fn print_config(config: &BTreeMap<String, Vec<u32>>, names: &HashMap<u32, String>) -> () {
    for (compat_tool, app_ids) in config {
        println!("{}", compat_tool);
        for app_id in app_ids {
            if let Some(name) = names.get(app_id) {
                println!("    {}", name);
            } else {
                println!("    Unknown (Id: {})", app_id);
            }
        }
    }
}

fn open_config<P>(path: P) -> Result<impl Iterator<Item=String>, Box<dyn std::error::Error>> where P: AsRef<Path> {
    let file = fs::File::open(path)?;
    let lines = std::io::BufReader::new(file)
        .lines()
        .filter_map(Result::ok);
    Ok(lines)
}

/// Super fragile parsing. Expects a well formed config.vdf in the form of
/// ```
/// ...
/// "CompatToolMapping"
/// {
///     ...
///     "[app_id]"
///     {
///     "name"		"[tool_name]"
///     "config"		"ignored"
///     "Priority"	"ignored"
///     }
///     ...
/// }
/// ...
/// ```
fn parse_compat_tool_mapping(config_lines: impl Iterator<Item=String>) -> BTreeMap<String, Vec<u32>> {
    let mut lines = config_lines.skip_while(|l| l.trim() != "\"CompatToolMapping\"")
        .skip(2);
    let mut map = BTreeMap::new();
    let mut brace_count = 0;
    let mut game_id: Option<u32> = None;

    while let (Some(line), true) = (lines.next(), brace_count > -1) {
        let line = line.trim();
        if line == "{" {
            brace_count += 1;
        }
        else if line == "}" {
            brace_count -= 1;
        }
        else if let Ok(id) = line.trim_matches('"').parse() {
            game_id = Some(id);
        }
        else if line.starts_with("\"name") && game_id.is_some() {
            let version = line.split_whitespace()
                .last()
                .map_or("", |s| s.trim_matches('"'));
            if version != "" {
                map.entry(version.to_string())
                    .or_insert_with(Vec::new)
                    .push(game_id.unwrap());
            }
            game_id = None;
        }
    }

    map
}

pub fn parse_steam_config(path: &Path) -> Result<BTreeMap<String, Vec<u32>>, Box<dyn std::error::Error>> {
    let lines = open_config(path)?;
    let tool_mapping = parse_compat_tool_mapping(lines);

    Ok(tool_mapping)
}

pub async fn fetch_app_names(app_ids: &Vec<u32>) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let urls = app_ids.iter()
        .map(|id| format!("https://store.steampowered.com/api/appdetails?filters=basic&appids={}", id))
        .collect::<Vec<String>>();
    let client = Client::new();
    
    let names = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                client.get(url).send().await?.json::<AppDetailsResponse>().await
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|r| {
            let id = r.0.keys().next().unwrap().clone();
            let details = r.0.values().into_iter().next().unwrap();
            if details.success {
                Some((id, details.data.name.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<u32, String>>();
    
    Ok(names)
}

#[derive(Deserialize, Debug)]
struct AppDetails {
    name: String
}

#[derive(Deserialize, Debug)]
struct AppDetailsData {
    data: AppDetails,
    success: bool
}

#[derive(Deserialize, Debug)]
struct AppDetailsResponse(HashMap<u32, AppDetailsData>);
