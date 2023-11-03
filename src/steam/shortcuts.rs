use super::{get_userdata_file, parse_names_from_bin_vdf, AppId, Result};
use log::error;
use std::collections::HashMap;
use std::path::Path;

const POSSIBLE_KEYS: [&str; 2] = ["appname", "AppName"];

pub fn parse_names(steam_home: &Path, app_ids: &[&AppId]) -> Result<HashMap<AppId, String>> {
    let mut result = HashMap::new();
    get_userdata_file(steam_home, "config/shortcuts.vdf")?
        .into_iter()
        .for_each(|userdata_dir| match std::fs::read(&userdata_dir.path) {
            Ok(contents) => {
                let shortcuts = parse_names_from_bin_vdf(&contents, &POSSIBLE_KEYS, app_ids);
                result.extend(shortcuts);
            }
            Err(e) => error!(
                "Failed to parse names from '{}': {}",
                userdata_dir.path.display(),
                e
            ),
        });

    Ok(result)
}
