use super::{parse_vdf_keys, AppId, KeyParser, DEFAULT_PROTON_APP_ID};
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
    if app_id != &DEFAULT_PROTON_APP_ID {
        map.entry(tool_name.to_string()).or_default().push(*app_id);
    }
}

pub fn parse_compat_tool_mapping(config_lines: impl Iterator<Item = String>) -> CompatToolMapping {
    let parsers = HashMap::from([("name", parse_tool_name as KeyParser<CompatToolMapping>)]);

    parse_vdf_keys("CompatToolMapping", config_lines, &parsers, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tool_name_excludes_app_zero() {
        let mut map = CompatToolMapping::new();

        parse_tool_name("name1", &AppId(0), &mut map);
        parse_tool_name("name2", &AppId(1), &mut map);

        assert_eq!(map.0.len(), 1);
        assert!(map.0.contains_key("name2"));
    }
}
