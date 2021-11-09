pub mod app_info;
mod compat_tool;
mod registry;
pub mod shortcuts;

pub use self::compat_tool::parse_compat_tool_mapping;
pub use self::registry::parse_registry;
use derive_more::{Constructor, Display, FromStr};
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

fn parse_names_from_bin_vdf(
    file_contents: &[u8],
    possible_keys: &[&str],
    whitelist: &[&AppId],
) -> HashMap<AppId, String> {
    let mut map = HashMap::new();
    const ID_KEY: &[u8; 6] = b"appid\0";

    for app_id in whitelist.iter() {
        let id = ID_KEY
            .iter()
            .chain(&(app_id.0 as u32).to_le_bytes())
            .copied()
            .collect::<Vec<u8>>();

        let name = &file_contents
            .windows(id.len())
            .position(|window| window == id)
            .map(|i| {
                let start = i + id.len() + 1; // index + "appname\0appid"
                let current = &file_contents[start..];

                possible_keys
                    .iter()
                    .map(|&key| (key.to_owned() + "\0").as_bytes().to_owned())
                    .collect::<Vec<_>>()
                    .iter()
                    .filter_map(|key| {
                        current
                            .windows(key.len())
                            .position(|window| window == key)
                            .map(|i| {
                                let start = i + key.len(); // index + "value\0"
                                let current = &current[start..];
                                let end = current.iter().position(|&b| b == 0).unwrap();
                                let name = &current[..end];
                                String::from_utf8_lossy(name).into_owned()
                            })
                    })
                    .next()
            })
            .flatten();

        if let Some(name) = name {
            map.insert(*app_id.to_owned(), name.to_owned());
        }
    }

    map
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_vdf_uses_first_potential_key() {
        let contents = [
            0x00u8, 0x73, 0x68, 0x6F, 0x72, 0x74, 0x63, 0x75, 0x74, 0x73, 0x00, 0x00, 0x30, 0x00,
            0x02, 0x61, 0x70, 0x70, 0x69, 0x64, 0x00, // appid\0
            0x6E, 0xB1, 0xFE, 0x99, 0x01, // 2583605614 (little endian u32)
            0x61, 0x70, 0x70, 0x6E, 0x61, 0x6D, 0x65, 0x00, // appname\0
            0x54, 0x68, 0x65, 0x20, 0x4e, 0x61, 0x6d, 0x65, 0x31, 0x00, // The Name1\0
            0x01, 0x65, 0x78, 0x65, 0x00, // .exe\0
            0x02, 0x61, 0x70, 0x70, 0x69, 0x64, 0x00, // appid\0
            0x6E, 0xB1, 0xFE, 0x98, 0x01, // 2566828398 (little endian u32)
            0x41, 0x70, 0x70, 0x4E, 0x61, 0x6D, 0x65, 0x00, // AppName\0
            0x54, 0x68, 0x65, 0x20, 0x4e, 0x61, 0x6d, 0x65, 0x32, 0x00, // The Name2\0
            0x01, 0x65, 0x78, 0x65, 0x00, // .exe\0
        ];
        let app_id1 = &AppId::new(2583605614);
        let app_id2 = &AppId::new(2566828398);
        let keys = ["appname", "AppName"];

        let result = parse_names_from_bin_vdf(&contents, &keys, &[app_id1, app_id2]);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get(app_id1).unwrap(), "The Name1");
        assert_eq!(result.get(app_id2).unwrap(), "The Name2");
    }

    #[test]
    fn text_vdf_parsing_is_case_insensitive() {
        let lines = r#"
            "Section"
            {
                "12345"
                {
                    "Asdf" "0"
                }
            }"#
        .lines()
        .map(|s| s.to_string());
        fn parse(_: &str, _: &AppId, result: &mut u32) {
            *result = *result + 1;
        }
        let parsers = HashMap::from([("aSdF", parse as KeyParser<u32>)]);

        let result = parse_vdf_keys("sEcTiOn", lines, &parsers);

        assert_eq!(
            result, 1,
            "Parsing section and keys should be case insensitive"
        );
    }
}
