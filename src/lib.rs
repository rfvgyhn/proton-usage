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

pub async fn parse_steam_config(path: &Path) -> Result<CompatToolConfig> {
    let lines = open_text_config(path)?;

    log::debug!("Parsing {}", path.display());
    let tool_mapping = steam::parse_compat_tool_mapping(lines);

    let ids = tool_mapping.values().flatten().collect::<Vec<_>>();

    log::info!("Fetching app names");
    let app_names = steam::fetch_app_names(&ids).await?;
    log::debug!("Found {} app names", app_names.len());

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
            (tool.clone(), names)
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
