use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct BudgetConfig {
    #[serde(default)]
    pub lints: Option<HashMap<String, String>>,
}

/// Parse a `BudgetConfig` from the file at `config_path`.
///
/// Returns the default configuration when:
/// - the file does not exist
/// - the file is empty or contains only whitespace
/// - the file contains invalid TOML
/// - the `budget` table or `lints` field is missing
pub fn parse_config(config_path: &Path) -> BudgetConfig {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return BudgetConfig::default(),
    };

    let content = content.trim();
    if content.is_empty() {
        return BudgetConfig::default();
    }

    parse_config_str(content)
}

/// Parse a `BudgetConfig` from a raw TOML string, falling back to defaults
/// on any error.
pub fn parse_config_str(raw: &str) -> BudgetConfig {
    let raw = raw.trim();
    if raw.is_empty() {
        return BudgetConfig::default();
    }

    if let Ok(wrapper) = toml::from_str::<BudgetConfigWrapper>(raw) {
        if let Some(budget) = wrapper.budget {
            return budget;
        }
    }
    toml::from_str::<BudgetConfig>(raw).unwrap_or_default()
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
struct BudgetConfigWrapper {
    #[serde(default)]
    budget: Option<BudgetConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_file(path: &Path, contents: &str) {
        let mut f = File::create(path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
    }

    #[test]
    fn empty_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.toml");
        write_file(&path, "");

        let config = parse_config(&path);
        assert!(config.lints.is_none());
    }

    #[test]
    fn whitespace_only_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("whitespace.toml");
        write_file(&path, "   \n\t\n  ");

        let config = parse_config(&path);
        assert!(config.lints.is_none());
    }

    #[test]
    fn missing_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");

        let config = parse_config(&path);
        assert!(config.lints.is_none());
    }

    #[test]
    fn invalid_toml_returns_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        write_file(&path, "this is not [[valid toml");

        let config = parse_config(&path);
        assert!(config.lints.is_none());
    }

    #[test]
    fn empty_string_returns_defaults() {
        let config = parse_config_str("");
        assert!(config.lints.is_none());
    }

    #[test]
    fn whitespace_only_string_returns_defaults() {
        let config = parse_config_str("   \n\t  ");
        assert!(config.lints.is_none());
    }

    #[test]
    fn invalid_toml_string_returns_defaults() {
        let config = parse_config_str("not valid {{{ toml");
        assert!(config.lints.is_none());
    }

    #[test]
    fn missing_budget_table_returns_defaults() {
        let config = parse_config_str("[lints]\nsoroban_storage_in_loop = \"deny\"");
        let lints = config.lints.as_ref().unwrap();
        assert_eq!(lints.get("soroban_storage_in_loop").unwrap(), "deny");
    }

    #[test]
    fn missing_lints_field_returns_defaults() {
        let config = parse_config_str("[budget]\nother = 1");
        assert!(config.lints.is_none());
    }

    #[test]
    fn valid_config_parses_correctly() {
        let toml_str = r#"
[budget]
lints = { "soroban_storage_in_loop" = "deny" }
"#;
        let config = parse_config_str(toml_str);
        let lints = config.lints.as_ref().unwrap();
        assert_eq!(lints.len(), 1);
        assert_eq!(lints.get("soroban_storage_in_loop").unwrap(), "deny");
    }

    #[test]
    fn valid_config_file_parses_correctly() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("good.toml");
        write_file(
            &path,
            "[budget]\nlints = { \"redundant_env_clone\" = \"warn\" }\n",
        );

        let config = parse_config(&path);
        let lints = config.lints.as_ref().unwrap();
        assert_eq!(lints.len(), 1);
        assert_eq!(lints.get("redundant_env_clone").unwrap(), "warn");
    }
}
