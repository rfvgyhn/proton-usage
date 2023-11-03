use super::{
    get_userdata_file, parse_vdf_keys, AppId, KeyParser, Result, SteamId64, DEFAULT_PROTON_APP_ID,
};
use crate::open_text_config;
use log::warn;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

pub struct LaunchOptions {
    pub app_id: AppId,
    pub options: String,
}

fn parse_launch_options(options: &str, app_id: &AppId, map: &mut Vec<LaunchOptions>) {
    if app_id != &DEFAULT_PROTON_APP_ID {
        map.push(LaunchOptions {
            app_id: app_id.clone(),
            options: options.to_string(),
        });
    }
}
pub fn parse_launch_options_mapping(
    steam_home: &Path,
) -> Result<BTreeMap<SteamId64, Vec<LaunchOptions>>> {
    let mut result = BTreeMap::new();
    let parsers = HashMap::from([(
        "LaunchOptions",
        parse_launch_options as KeyParser<Vec<LaunchOptions>>,
    )]);

    get_userdata_file(steam_home, "config/localconfig.vdf")?
        .into_iter()
        .for_each(|userdata_dir| {
            if let Ok(config_lines) = open_text_config(&userdata_dir.path) {
                let options = parse_vdf_keys("apps", config_lines, &parsers, None);
                result.insert(userdata_dir.user_id.into(), options);
            } else {
                warn!(
                    "Couldn't open file '{}'",
                    userdata_dir.path.to_string_lossy()
                )
            }
        });

    Ok(result)
}
