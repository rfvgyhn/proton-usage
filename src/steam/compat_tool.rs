use super::AppId;
use derive_more::{From, Into, IntoIterator};
use std::collections::hash_map::{Entry, Values};
use std::collections::HashMap;

#[derive(IntoIterator, Into, From)]
pub struct CompatToolMapping(HashMap<String, Vec<AppId>>);
impl CompatToolMapping {
    pub fn values(&self) -> Values<'_, String, Vec<AppId>> {
        self.0.values()
    }
    pub fn entry(&mut self, key: String) -> Entry<'_, String, Vec<AppId>> {
        self.0.entry(key)
    }
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

/// Super fragile parsing. Expects a well formed config.vdf in the form of
/// ```vdf
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
pub fn parse_compat_tool_mapping(config_lines: impl Iterator<Item = String>) -> CompatToolMapping {
    let mut lines = config_lines
        .skip_while(|l| l.trim() != "\"CompatToolMapping\"")
        .skip(2);
    let mut map = CompatToolMapping::new();
    let mut depth = 0;
    let mut app_id: Option<AppId> = None;

    while let (Some(line), true) = (lines.next(), depth > -1) {
        let line = line.trim();
        if line == "{" {
            depth += 1;
        } else if line == "}" {
            depth -= 1;
        } else if let Ok(id) = line.trim_matches('"').parse() {
            app_id = Some(id);
        } else if line.starts_with("\"name") && app_id.is_some() {
            let version = line
                .split_whitespace()
                .last()
                .map_or("", |s| s.trim_matches('"'));
            if !version.is_empty() {
                map.entry(version.to_string())
                    .or_insert_with(Vec::new)
                    .push(app_id.unwrap());
            }
            app_id = None;
        }
    }

    map
}
