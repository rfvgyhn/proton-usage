use super::{parse_vdf_keys, AppId, KeyParser};
use derive_more::{From, Into, IntoIterator};
use std::collections::hash_map::{Entry, Values};
use std::collections::HashMap;

#[derive(IntoIterator, Into, From, Default)]
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

fn parse_tool_name(line: &str, app_id: &AppId, map: &mut CompatToolMapping) {
    let version = line
        .split_whitespace()
        .last()
        .map_or("", |s| s.trim_matches('"'));
    if !version.is_empty() {
        map.entry(version.to_string())
            .or_insert_with(Vec::new)
            .push(*app_id);
    }
}

pub fn parse_compat_tool_mapping(config_lines: impl Iterator<Item = String>) -> CompatToolMapping {
    let parsers = HashMap::from([("name", parse_tool_name as KeyParser<CompatToolMapping>)]);

    parse_vdf_keys("CompatToolMapping", config_lines, &parsers)
}
