use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::io::BufRead;
use std::path::Path;
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize};

pub struct CompatToolConfig(BTreeMap<String, Vec<String>>);

impl Display for CompatToolConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (compat_tool, app_names) in self.0.iter() {
            writeln!(f, "{}", compat_tool)?;
            for name in app_names {
                writeln!(f, "    {}", name)?;
            }
        }
        
        Ok(())
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
///     "name"        "[tool_name]"
///     "config"      "ignored"
///     "Priority"    "ignored"
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
            if !version.is_empty() {
                map.entry(version.to_string())
                    .or_insert_with(Vec::new)
                    .push(game_id.unwrap());
            }
            game_id = None;
        }
    }

    map
}

pub async fn parse_steam_config(path: &Path) -> Result<CompatToolConfig, Box<dyn std::error::Error>> {
    let lines = open_config(path)?;

    log::debug!("Parsing {}", path.display());
    let tool_mapping = parse_compat_tool_mapping(lines);
    
    let ids = tool_mapping.values().flatten().collect::<Vec<_>>();

    log::info!("Fetching app names");
    let names = fetch_app_names(&ids).await?;
    log::debug!("Found {} app names", names.len());
    
    let config = tool_mapping.iter()
        .map(|(tool, ids)| {
            let names = ids.iter()
                .map(|id| names.get(id).cloned().unwrap_or_else(|| format!("Unknown (Id: {})", id)))
                .collect();
            (tool.clone(), names)
        })
        .collect();
    
    Ok(CompatToolConfig(config))
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

async fn fetch_app_names(app_ids: &[&u32]) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
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
            let id = *r.0.keys().next().unwrap();
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