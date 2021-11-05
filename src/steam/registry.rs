use super::{parse_vdf_keys, AppId, InstallState, KeyParser};
use derive_more::{From, Into, IntoIterator};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct RegistryEntry {
    pub name: Option<String>,
    pub install_state: InstallState,
}

#[derive(IntoIterator, Into, From, Default, Debug)]
pub struct Registry(HashMap<AppId, RegistryEntry>);
impl Registry {
    pub fn entry(&mut self, key: AppId) -> Entry<'_, AppId, RegistryEntry> {
        self.0.entry(key)
    }
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

fn parse_installed(value: &str, app_id: &AppId, map: &mut Registry) {
    let state = match value {
        "0" => InstallState::NotInstalled,
        "1" => InstallState::Installed,
        _ => InstallState::Unknown,
    };

    map.entry(*app_id)
        .or_insert_with(RegistryEntry::default)
        .install_state = state;
}

fn parse_name(name: &str, app_id: &AppId, map: &mut Registry) {
    map.entry(*app_id)
        .or_insert_with(RegistryEntry::default)
        .name = Some(name.to_string());
}

pub fn parse_registry(config_lines: impl Iterator<Item = String>) -> Registry {
    let parsers = HashMap::from([
        ("installed", parse_installed as KeyParser<Registry>),
        ("name", parse_name as KeyParser<Registry>),
    ]);

    parse_vdf_keys("apps", config_lines, &parsers)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn treats_0_as_not_installed() {
        let app_id = AppId::new(12345);
        let lines = r#"
            "apps"
            {
                "12345"
                {
                    "installed"		"0"
                    "Updating"		"0"
                    "Running"		"0"
                }
            }"#
        .lines()
        .map(|s| s.to_string());

        let registry = super::parse_registry(lines);
        let entry = registry.0.get(&app_id);

        assert_eq!(entry.is_some(), true, "Entry should be Some");
        assert_eq!(
            entry.unwrap().install_state,
            super::InstallState::NotInstalled
        );
    }

    #[test]
    fn treats_1_as_installed() {
        let app_id = AppId::new(12345);
        let lines = r#"
            "apps"
            {
                "12345"
                {
                    "installed"		"1"
                    "Updating"		"0"
                    "Running"		"0"
                }
            }"#
        .lines()
        .map(|s| s.to_string());

        let registry = super::parse_registry(lines);
        let entry = registry.0.get(&app_id);

        assert_eq!(entry.is_some(), true, "Entry should be Some");
        assert_eq!(entry.unwrap().install_state, super::InstallState::Installed);
    }

    #[test]
    fn name_is_some_if_kvp_present() {
        let app_id = AppId::new(12345);
        let lines = r#"
            "apps"
            {
                "12345"
                {
                    "installed"		"1"
                    "Updating"		"0"
                    "Running"		"0"
                    "name"  	"asdf"
                }
            }"#
        .lines()
        .map(|s| s.to_string());

        let registry = super::parse_registry(lines);
        let entry = registry.0.get(&app_id);

        assert_eq!(entry.is_some(), true, "Entry should be Some");
        assert_eq!(entry.unwrap().name, Some("asdf".to_string()));
    }

    #[test]
    fn name_is_none_if_kvp_not_present() {
        let app_id = AppId::new(12345);
        let lines = r#"
            "apps"
            {
                "12345"
                {
                    "installed"		"1"
                    "Updating"		"0"
                    "Running"		"0"
                }
            }"#
        .lines()
        .map(|s| s.to_string());

        let registry = super::parse_registry(lines);
        let entry = registry.0.get(&app_id);

        assert_eq!(entry.is_some(), true, "Entry should be Some");
        assert_eq!(entry.unwrap().name, None);
    }
}