use super::{Result, SteamId64};
use crate::open_text_config;
use std::path::Path;

fn parse_display_name(id: &SteamId64, config_lines: impl Iterator<Item = String>) -> String {
    config_lines
        .skip_while(|line| line.trim_start() != format!("\"{}\"", id))
        .find(|line| line.trim_start().starts_with("\"PersonaName"))
        .and_then(|line| line.rsplit_terminator('"').next().map(|s| s.to_string()))
        .unwrap_or_else(|| id.to_string())
}

pub fn get_display_name(steam_home: &Path, id: &SteamId64) -> Result<String> {
    const CONFIG_PATH: &str = "root/config/loginusers.vdf";
    let lines = open_text_config(steam_home.join(CONFIG_PATH))
        .map_err(|e| format!("Couldn't open file '{}': {}", CONFIG_PATH, e))?;

    Ok(parse_display_name(id, lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: SteamId64 = SteamId64(12345678901234567);

    #[test]
    fn can_parse_display_name() {
        let lines = r#"
            "users"
            {
                "123"
                {
                    "AccountName"		"Account Name1"
                    "PersonaName"		"Display Name1"
                    "RememberPassword"		"1"
                    "WantsOfflineMode"		"0"
                    "SkipOfflineModeWarning"		"0"
                    "AllowAutoLogin"		"1"
                    "mostrecent"		"1"
                    "Timestamp"		"123456789"
                }
                "12345678901234567"
                {
                    "AccountName"		"Account Name1"
                    "PersonaName"		"Display Name1"
                    "RememberPassword"		"1"
                    "WantsOfflineMode"		"0"
                    "SkipOfflineModeWarning"		"0"
                    "AllowAutoLogin"		"1"
                    "mostrecent"		"1"
                    "Timestamp"		"123456789"
                }
            }
            "#
        .lines()
        .map(|s| s.to_string());

        let display_name = parse_display_name(&ID, lines);

        assert_eq!(display_name, "Display Name1", "")
    }

    #[test]
    fn defaults_to_id_if_no_persona_name() {
        let lines = r#"
            "users"
            {
                "12345678901234567"
                {
                    "AccountName"		"Account Name"
                    "RememberPassword"		"1"
                    "WantsOfflineMode"		"0"
                    "SkipOfflineModeWarning"		"0"
                    "AllowAutoLogin"		"1"
                    "mostrecent"		"1"
                    "Timestamp"		"123456789"
                }
            }
            "#
        .lines()
        .map(|s| s.to_string());

        let display_name = parse_display_name(&ID, lines);

        assert_eq!(display_name, ID.to_string(), "")
    }

    #[test]
    fn defaults_to_id_if_no_id_match() {
        let lines = r#"
            "users"
            {
                "123"
                {
                    "AccountName"		"Account Name"
                    "PersonaName"		"Display Name"
                    "RememberPassword"		"1"
                    "WantsOfflineMode"		"0"
                    "SkipOfflineModeWarning"		"0"
                    "AllowAutoLogin"		"1"
                    "mostrecent"		"1"
                    "Timestamp"		"123456789"
                }
            }
            "#
        .lines()
        .map(|s| s.to_string());

        let display_name = parse_display_name(&ID, lines);

        assert_eq!(display_name, ID.to_string(), "")
    }
}
