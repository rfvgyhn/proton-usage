use super::{parse_names_from_bin_vdf, AppId, Result};
use log::error;
use std::collections::HashMap;
use std::path::Path;

const POSSIBLE_KEYS: [&str; 1] = ["name"];

pub fn parse_names(file_path: &Path, app_ids: &[&AppId]) -> Result<HashMap<AppId, String>> {
    let mut result = HashMap::new();

    match std::fs::read(&file_path) {
        Ok(contents) => {
            let names = parse_names_from_bin_vdf(&contents, &POSSIBLE_KEYS, app_ids);
            result.extend(names);
        }
        Err(e) => error!("Failed to read '{}': {}", file_path.display(), e),
    }

    Ok(result)
}
