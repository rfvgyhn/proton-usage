mod steam;

use crate::steam::registry::Registry;
use crate::steam::AppId;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::path::Path;
use std::{fmt, fs};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const INDENT_WIDTH: usize = 4;

pub struct CompatToolConfig(BTreeMap<String, Vec<App>>);
impl Display for CompatToolConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, (compat_tool, apps)) in self.0.iter().enumerate() {
            writeln!(f, "{}", compat_tool)?;

            for app in apps {
                write!(f, "{:i$}", "", i = INDENT_WIDTH)?;
                app.fmt(f)?;
            }

            if i < self.0.len() - 1 {
                writeln!(f, "")?;
            }
        }

        Ok(())
    }
}

pub struct App {
    pub name: String,
    pub install_state: InstallState,
}

impl Display for App {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.install_state != InstallState::Installed {
            writeln!(f, "{} ({})", self.name, self.install_state)?;
        } else {
            writeln!(f, "{}", self.name)?;
        }

        Ok(())
    }
}

pub struct LaunchOptions {
    pub app: App,
    pub value: String,
}

pub struct LaunchOptionsConfig(BTreeMap<String, Vec<LaunchOptions>>);
impl Display for LaunchOptionsConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (user_id, apps) in self.0.iter() {
            let mut indent = 0;
            if self.0.len() != 1 {
                writeln!(f, "{}", user_id)?;
                indent += 1;
            }

            for (i, options) in apps.iter().enumerate() {
                write!(f, "{:i$}", "", i = indent * INDENT_WIDTH)?;
                options.app.fmt(f)?;
                writeln!(
                    f,
                    "{:i$}{}",
                    "",
                    options.value.replace("\\\"", "\""),
                    i = (indent + 1) * INDENT_WIDTH
                )?;

                if i < apps.len() - 1 {
                    writeln!(f, "")?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstallState {
    NotInstalled,
    Installed,
    Shortcut,
    Unknown,
}

impl Display for InstallState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallState::NotInstalled => write!(f, "Not Installed"),
            InstallState::Installed => write!(f, "Installed"),
            InstallState::Shortcut => write!(f, "Shortcut"),
            InstallState::Unknown => write!(f, "Unknown Install State"),
        }
    }
}

pub fn parse_launch_options(steam_home: &Path) -> Result<LaunchOptionsConfig> {
    let launch_options = steam::parse_launch_options_mapping(steam_home)?;
    let unique_apps = launch_options
        .values()
        .flat_map(|ids| ids.iter().map(|o| &o.app_id))
        .collect();
    let registry = get_registry(steam_home, &unique_apps)?;
    let (app_names, shortcuts) = get_app_names(steam_home, &unique_apps, &registry)?;

    let mut result = BTreeMap::new();
    for (id, options) in launch_options {
        let o = options
            .iter()
            .map(|l| {
                let app = to_app(&l.app_id, &app_names, &registry, &shortcuts);

                LaunchOptions {
                    app,
                    value: (&l.options).to_string(),
                }
            })
            .collect();
        let username = steam::get_display_name(&steam_home, &id)?;
        result.insert(username, o);
    }
    Ok(LaunchOptionsConfig(result))
}

pub fn parse_tool_mapping(steam_home: &Path) -> Result<CompatToolConfig> {
    let config_path = steam_home.join("root/config/config.vdf");
    log::debug!("Parsing {}", config_path.display());
    let config_lines = open_text_config(config_path)?;
    let tool_mapping = steam::parse_compat_tool_mapping(config_lines);
    let mut unique_apps = tool_mapping.apps();
    unique_apps.remove(&steam::AppId::new(0));

    let registry = get_registry(steam_home, &unique_apps)?;
    let (app_names, shortcuts) = get_app_names(steam_home, &unique_apps, &registry)?;

    let config = tool_mapping
        .into_iter()
        .map(|(tool, ids)| {
            let names = ids
                .iter()
                .map(|id| to_app(id, &app_names, &registry, &shortcuts))
                .collect();
            (tool, names)
        })
        .collect();

    Ok(CompatToolConfig(config))
}

fn install_state(
    app_id: &AppId,
    registry: &Registry,
    shortcuts: &HashMap<AppId, String>,
) -> InstallState {
    if registry.app_is_installed(app_id) {
        InstallState::Installed
    } else if shortcuts.contains_key(app_id) {
        InstallState::Shortcut
    } else {
        InstallState::NotInstalled
    }
}

fn get_registry(steam_home: &Path, whitelist: &HashSet<&AppId>) -> Result<Registry> {
    let registry_path = steam_home.join("registry.vdf");
    log::debug!("Parsing {}", registry_path.display());
    let registry_lines = open_text_config(registry_path)?;

    Ok(steam::registry::parse_registry(registry_lines, whitelist))
}

fn get_app_names(
    steam_home: &Path,
    whitelist: &HashSet<&AppId>,
    registry: &Registry,
) -> Result<(HashMap<AppId, String>, HashMap<AppId, String>)> {
    let mut shortcuts = HashMap::new();
    let mut app_names = registry.app_names.clone();
    log::debug!("Found {} name(s) from registry.vdf", app_names.len());

    if app_names.len() != whitelist.len() {
        let appinfo_path = steam_home.join("root/appcache/appinfo.vdf");
        log::debug!("Parsing {}", appinfo_path.display());
        let missing_names = whitelist
            .difference(&HashSet::from_iter(app_names.keys()))
            .copied()
            .collect::<Vec<&AppId>>();
        let names = steam::app_info::parse_names(&appinfo_path, &missing_names)?;
        log::debug!("Found {} name(s) from appinfo.vdf", names.len());
        app_names.extend(names);
    }

    if app_names.len() != whitelist.len() {
        log::debug!("Parsing shortcuts");
        let missing_names = whitelist
            .difference(&HashSet::from_iter(app_names.keys()))
            .copied()
            .collect::<Vec<&AppId>>();
        shortcuts = steam::shortcuts::parse_names(&steam_home, &missing_names)?;
        log::debug!("Found {} name(s) from shortcuts.vdf", shortcuts.len());
        app_names.extend(shortcuts.clone());
    }

    Ok((app_names, shortcuts))
}

fn to_app(
    id: &AppId,
    app_names: &HashMap<AppId, String>,
    registry: &Registry,
    shortcuts: &HashMap<AppId, String>,
) -> App {
    let name = match app_names.get(id) {
        Some(n) => n.to_string(),
        None => {
            log::info!("{} is possibly a deleted shortcut", id);
            format!("Unknown (Id: {})", id)
        }
    };
    let install_state = install_state(id, registry, shortcuts);
    App {
        name,
        install_state,
    }
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
