use super::{parse_vdf_keys, AppId, KeyParser};
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct Registry {
    pub app_names: HashMap<AppId, String>,
    installed_apps: HashSet<AppId>,
    uninstalled_apps: HashSet<AppId>,
}
impl Registry {
    pub fn app_is_installed(&self, app_id: &AppId) -> bool {
        self.installed_apps.contains(app_id)
    }
}

fn parse_installed(value: &str, app_id: &AppId, registry: &mut Registry) {
    if value == "1" {
        registry.installed_apps.insert(*app_id);
    }
}

fn parse_name(name: &str, app_id: &AppId, registry: &mut Registry) {
    registry.app_names.insert(*app_id, name.to_string());
}

pub fn parse_registry(
    config_lines: impl Iterator<Item = String>,
    whitelist: &HashSet<&AppId>,
) -> Registry {
    let parsers = HashMap::from([
        ("installed", parse_installed as KeyParser<Registry>),
        ("name", parse_name as KeyParser<Registry>),
    ]);

    parse_vdf_keys("apps", config_lines, &parsers, Some(whitelist))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn treats_0_as_not_installed() {
        let app_id = AppId::new(12345);
        let filter = HashSet::from([&app_id]);
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

        let registry = super::parse_registry(lines, &filter);
        let installed_entry = registry.installed_apps.get(&app_id);
        let uninstalled_entry = registry.uninstalled_apps.get(&app_id);

        assert!(installed_entry.is_none());
        assert!(uninstalled_entry.is_some());
    }

    #[test]
    fn treats_1_as_installed() {
        let app_id = AppId::new(12345);
        let filter = HashSet::from([&app_id]);
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

        let registry = super::parse_registry(lines, &filter);
        let installed_entry = registry.installed_apps.get(&app_id);
        let uninstalled_entry = registry.uninstalled_apps.get(&app_id);

        assert!(installed_entry.is_some());
        assert!(uninstalled_entry.is_none());
    }

    #[test]
    fn name_is_some_if_kvp_present() {
        let app_id = AppId::new(12345);
        let filter = HashSet::from([&app_id]);
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

        let registry = super::parse_registry(lines, &filter);
        let entry = registry.app_names.get(&app_id);

        assert!(entry.is_some());
        assert_eq!(entry.unwrap(), "asdf");
    }

    #[test]
    fn name_is_none_if_kvp_not_present() {
        let app_id = AppId::new(12345);
        let filter = HashSet::from([&app_id]);
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

        let registry = super::parse_registry(lines, &filter);
        let entry = registry.app_names.get(&app_id);

        assert!(entry.is_none());
    }
}
