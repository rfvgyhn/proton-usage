mod steam;

use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::BufRead;
use std::path::Path;
use std::{fmt, fs};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
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

pub async fn parse_steam_config(steam_home: &Path) -> Result<CompatToolConfig> {
    let config_path = steam_home.join("root/config/config.vdf");
    log::debug!("Parsing {}", config_path.display());
    let config_lines = open_text_config(config_path)?;
    let tool_mapping = steam::parse_compat_tool_mapping(config_lines);
    let unique_apps = tool_mapping.apps();

    let registry_path = steam_home.join("registry.vdf");
    log::debug!("Parsing {}", registry_path.display());
    let registry_lines = open_text_config(registry_path)?;
    let registry = steam::parse_registry(registry_lines, &unique_apps);

    let mut app_names = registry.to_name_map();
    log::debug!("Found {} names from registry.vdf", app_names.len());
    if app_names.len() != unique_apps.len() {
        log::info!("Fetching app names");
        let ids_with_names = app_names.keys().collect();
        let ids = unique_apps
            .difference(&ids_with_names)
            .cloned()
            .collect::<Vec<_>>();
        let api_names = steam::fetch_app_names(&ids).await?;
        log::debug!("Found {} names from Steam API", api_names.len());
        app_names.extend(api_names);
    }

    let config = tool_mapping
        .into_iter()
        .map(|(tool, ids)| {
            let names = ids
                .iter()
                .map(|id| {
                    app_names
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| format!("Unknown (Id: {})", id))
                })
                .collect();
            (tool, names)
        })
        .collect();

    Ok(CompatToolConfig(config))
}

fn open_text_config<P>(path: P) -> Result<impl Iterator<Item = String>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(path)?;
    let lines = std::io::BufReader::new(file)
        .lines()
        .filter_map(std::result::Result::ok);

    Ok(lines)
}
