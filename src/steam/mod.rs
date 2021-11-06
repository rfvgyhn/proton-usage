mod compat_tool;
mod registry;

pub use self::compat_tool::parse_compat_tool_mapping;
pub use self::registry::parse_registry;
use derive_more::{Constructor, Display, FromStr};
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Constructor, Display, FromStr, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct AppId(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstallState {
    NotInstalled,
    Installed,
    Shortcut,
    Unknown,
}
impl Default for InstallState {
    fn default() -> Self {
        InstallState::Unknown
    }
}

#[derive(Deserialize, Debug)]
struct AppDetails {
    name: String,
}

#[derive(Deserialize, Debug)]
struct AppDetailsData {
    data: AppDetails,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct AppDetailsResponse(HashMap<u64, AppDetailsData>);

pub async fn fetch_app_names(app_ids: &[&AppId]) -> Result<HashMap<AppId, String>> {
    let urls = app_ids
        .iter()
        .map(|id| {
            format!(
                "https://store.steampowered.com/api/appdetails?filters=basic&appids={}",
                id
            )
        })
        .collect::<Vec<String>>();
    let client = Client::new();

    let names = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                client
                    .get(url)
                    .send()
                    .await?
                    .json::<AppDetailsResponse>()
                    .await
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter_map(|r| {
            let id = *r.0.keys().next().unwrap();
            let details = r.0.values().into_iter().next().unwrap();
            if details.success {
                Some((AppId::new(id), details.data.name.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<AppId, String>>();

    Ok(names)
}

type KeyParser<T> = fn(&str, &AppId, &mut T);

/// Super fragile parsing. May have unexpected results if the vdf is malformed.
/// ```vdf
/// ...
/// "[section]"
/// {
///     ...
///     "[app_id]"
///     {
///     "[key_1]"   "[value_1]"
///     "[key_2]"   "[value_2]"
///     }
///     ...
/// }
/// ...
/// ```
fn parse_vdf_keys<T>(
    section: &str,
    config_lines: impl Iterator<Item = String>,
    key_parsers: &HashMap<&str, KeyParser<T>>,
) -> T
where
    T: Default,
{
    let mut lines = config_lines
        .skip_while(|l| !l.trim().eq_ignore_ascii_case(&format!("\"{}\"", section)))
        .skip(2);
    let mut result = T::default();
    let mut depth = 0;
    let mut app_id: Option<AppId> = None;

    while let (Some(line), true) = (lines.next(), depth > -1) {
        let line = line.trim();
        if line == "{" {
            depth += 1;
        } else if line == "}" {
            depth -= 1;
            app_id = None;
        } else if let Ok(id) = line.trim_matches('"').parse() {
            app_id = Some(id);
        } else if let Some(app_id) = app_id {
            key_parsers
                .iter()
                .filter(|(key, _)| {
                    line.to_lowercase()
                        .starts_with(&format!("\"{}\"", key.to_lowercase()))
                })
                .filter_map(|(_, parse)| {
                    line.split('"')
                        .nth(3)
                        .map(|s| (s, parse))
                        .filter(|(s, _)| !s.is_empty())
                })
                .for_each(|(value, parse)| {
                    parse(value, &app_id, &mut result);
                });
        }
    }

    result
}
