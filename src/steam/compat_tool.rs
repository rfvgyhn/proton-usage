use super::{parse_vdf_keys, AppId, KeyParser};
use derive_more::{From, Into, IntoIterator};
use std::collections::hash_map::{Entry, Values};
use std::collections::{HashMap, HashSet};

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
    pub fn apps(&self) -> HashSet<&AppId> {
        self.0.values().flatten().collect()
    }
}

fn parse_tool_name(tool_name: &str, app_id: &AppId, map: &mut CompatToolMapping) {
    map.entry(tool_name.to_string())
        .or_insert_with(Vec::new)
        .push(*app_id);
}

pub fn parse_compat_tool_mapping(config_lines: impl Iterator<Item = String>) -> CompatToolMapping {
    let parsers = HashMap::from([("name", parse_tool_name as KeyParser<CompatToolMapping>)]);

    parse_vdf_keys("CompatToolMapping", config_lines, &parsers, None)
}
