use super::{parse_names_from_bin_vdf, AppId, Result};
use log::error;
use std::collections::HashMap;
use std::path::Path;

const POSSIBLE_KEYS: [&str; 2] = ["appname", "AppName"];

pub fn parse_names(steam_home: &Path, app_ids: &[&AppId]) -> Result<HashMap<AppId, String>> {
    let mut result = HashMap::new();
    let userdata_path = steam_home.join("userdata");

    std::fs::read_dir(userdata_path)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.path().is_dir().then(|| entry.path()))
        .filter_map(|dir| {
            dir.components()
                .last()
                .map(|component| component.as_os_str().to_str().unwrap_or(""))
                .map(|str| str.parse::<u32>().unwrap_or(0))
                .filter(|user_id| *user_id != 0)
                .map(|_| dir)
        })
        .for_each(|userdata_dir| {
            let shortcuts_path = userdata_dir.join("config/shortcuts.vdf");
            if shortcuts_path.exists() {
                match std::fs::read(&shortcuts_path) {
                    Ok(contents) => {
                        let shortcuts =
                            parse_names_from_bin_vdf(&contents, &POSSIBLE_KEYS, app_ids);
                        result.extend(shortcuts);
                    }
                    Err(e) => error!("Failed to read '{}': {}", shortcuts_path.display(), e),
                }
            }
        });

    Ok(result)
}
