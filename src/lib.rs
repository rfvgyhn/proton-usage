mod steam;

use std::collections::{BTreeMap, HashSet};
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

pub fn parse_steam_config(steam_home: &Path) -> Result<CompatToolConfig> {
    let config_path = steam_home.join("root/config/config.vdf");
    log::debug!("Parsing {}", config_path.display());
    let config_lines = open_text_config(config_path)?;
    let tool_mapping = steam::parse_compat_tool_mapping(config_lines);
    let mut unique_apps = tool_mapping.apps();
    unique_apps.remove(&steam::AppId::new(0));

    let registry_path = steam_home.join("registry.vdf");
    log::debug!("Parsing {}", registry_path.display());
    let registry_lines = open_text_config(registry_path)?;
    let registry = steam::parse_registry(registry_lines, &unique_apps);

    let mut app_names = registry.to_name_map();
    log::debug!("Found {} name(s) from registry.vdf", app_names.len());

    if app_names.len() != unique_apps.len() {
        let appinfo_path = steam_home.join("root/appcache/appinfo.vdf");
        log::debug!("Parsing {}", appinfo_path.display());
        let missing_names = unique_apps
            .difference(&HashSet::from_iter(app_names.keys()))
            .copied()
            .collect::<Vec<&steam::AppId>>();
        let names = steam::app_info::parse_names(&appinfo_path, &missing_names)?;
        log::debug!("Found {} name(s) from appinfo.vdf", names.len());
        app_names.extend(names);
    }

    if app_names.len() != unique_apps.len() {
        log::debug!("Parsing shortcuts");
        let missing_names = unique_apps
            .difference(&HashSet::from_iter(app_names.keys()))
            .copied()
            .collect::<Vec<&steam::AppId>>();
        let shortcuts = steam::shortcuts::parse_names(&steam_home.join("steam"), &missing_names)?;
        log::debug!("Found {} name(s) from shortcuts.vdf", shortcuts.len());
        app_names.extend(shortcuts);
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
