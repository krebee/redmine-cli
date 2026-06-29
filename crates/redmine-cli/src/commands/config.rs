use serde_json::{json, Value};

use crate::cli::ConfigSubcommand;
use crate::config::{config_path, load_config, save_config, Config, Profile};
use crate::error::AgentError;

pub(super) fn run(command: ConfigSubcommand) -> Result<Value, AgentError> {
    match command {
        ConfigSubcommand::Init {
            url,
            api_key_env,
            profile,
            default_project,
            dry_run,
        } => init(url, api_key_env, profile, default_project, dry_run),
        ConfigSubcommand::Show => show(),
    }
}

fn init(
    url: String,
    api_key_env: String,
    profile: String,
    default_project: Option<String>,
    dry_run: bool,
) -> Result<Value, AgentError> {
    let config = Config::new(
        profile,
        Profile {
            url,
            api_key_env: Some(api_key_env),
            default_project,
        },
    );

    if dry_run {
        Ok(json!({
            "dryRun": true,
            "config": config,
        }))
    } else {
        let path = save_config(&config)?;
        Ok(json!({
            "path": path,
            "config": config,
        }))
    }
}

fn show() -> Result<Value, AgentError> {
    let config = load_config()?;

    Ok(json!({
        "path": config_path()?,
        "config": config,
    }))
}
