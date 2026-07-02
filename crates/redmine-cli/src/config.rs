use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AgentError;

pub const DEFAULT_API_KEY_ENV: &str = "REDMINE_API_KEY";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    pub url: String,
    pub api_key_env: Option<String>,
    pub default_project: Option<String>,
    #[serde(default)]
    pub ssl_no_revoke: bool,
}

impl Profile {
    pub fn api_key_env_name(&self) -> &str {
        self.api_key_env.as_deref().unwrap_or(DEFAULT_API_KEY_ENV)
    }
}

impl Config {
    pub fn new(profile_name: String, profile: Profile) -> Self {
        let mut profiles = BTreeMap::new();
        profiles.insert(profile_name.clone(), profile);

        Self {
            default_profile: Some(profile_name),
            profiles,
        }
    }

    pub fn select_profile(&self, profile: Option<&str>) -> Result<(String, Profile), AgentError> {
        let profile_name = profile
            .map(ToOwned::to_owned)
            .or_else(|| env::var("REDMINE_PROFILE").ok())
            .or_else(|| self.default_profile.clone())
            .unwrap_or_else(|| "default".to_string());

        let profile = self
            .profiles
            .get(&profile_name)
            .cloned()
            .ok_or_else(|| AgentError::MissingProfile(profile_name.clone()))?;

        Ok((profile_name, profile))
    }
}

pub fn config_path() -> Result<PathBuf, AgentError> {
    if let Some(path) = env_path("REDMINE_CLI_CONFIG") {
        return Ok(path);
    }

    if let Some(path) = env_path("REDMINE_AGENT_CONFIG") {
        return Ok(path);
    }

    if let Some(path) = env_path("XDG_CONFIG_HOME") {
        return Ok(config_file_under(&path));
    }

    if let Some(path) = env_path("APPDATA") {
        return Ok(config_file_under(&path));
    }

    if let Some(path) = env_path("HOME") {
        return Ok(path.join(".config").join("redmine-cli").join("config.toml"));
    }

    Err(AgentError::InvalidConfig(
        "Could not determine config path. Set REDMINE_CLI_CONFIG.".to_string(),
    ))
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var(name).ok().map(PathBuf::from)
}

fn config_file_under(base: &Path) -> PathBuf {
    base.join("redmine-cli").join("config.toml")
}

pub fn load_config() -> Result<Config, AgentError> {
    let path = config_path()?;
    if !path.exists() {
        return Err(AgentError::MissingConfig);
    }

    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

pub fn save_config(config: &Config) -> Result<PathBuf, AgentError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, toml::to_string_pretty(config)?)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_uses_default_api_key_env_when_omitted() {
        let profile = Profile {
            url: "https://redmine.example.com".to_string(),
            api_key_env: None,
            default_project: None,
            ssl_no_revoke: false,
        };

        assert_eq!(profile.api_key_env_name(), DEFAULT_API_KEY_ENV);
    }

    #[test]
    fn select_profile_uses_explicit_profile_first() {
        let config = Config::new(
            "default".to_string(),
            Profile {
                url: "https://redmine.example.com".to_string(),
                api_key_env: None,
                default_project: None,
                ssl_no_revoke: false,
            },
        );

        let (name, profile) = config
            .select_profile(Some("default"))
            .expect("profile should exist");

        assert_eq!(name, "default");
        assert_eq!(profile.url, "https://redmine.example.com");
    }

    #[test]
    fn profile_defaults_ssl_no_revoke_to_false() {
        let profile: Profile = toml::from_str(
            r#"
url = "https://redmine.example.com"
"#,
        )
        .expect("profile should deserialize");

        assert!(!profile.ssl_no_revoke);
    }
}
