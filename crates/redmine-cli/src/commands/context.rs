use serde_json::Value;

use crate::config::{load_config, Profile};
use crate::error::AgentError;
use crate::output::CommandResult;
use crate::redmine_client::RedmineClient;

pub(super) struct ClientContext {
    pub(super) client: RedmineClient,
    pub(super) profile: Profile,
    profile_name: String,
}

pub(super) struct CommandOutput {
    data: Value,
    profile: Option<String>,
    redmine_url: Option<String>,
}

impl ClientContext {
    pub(super) fn load(
        profile: Option<&str>,
        timeout_ms: u64,
        ssl_no_revoke: bool,
    ) -> Result<Self, AgentError> {
        let config = load_config()?;
        let (profile_name, profile) = config.select_profile(profile)?;
        let client = RedmineClient::new(&profile, timeout_ms, ssl_no_revoke)?;

        Ok(Self {
            client,
            profile,
            profile_name,
        })
    }

    pub(super) fn output(&self, data: Value) -> CommandOutput {
        CommandOutput {
            data,
            profile: Some(self.profile_name.clone()),
            redmine_url: Some(self.client.redmine_url()),
        }
    }
}

impl CommandOutput {
    pub(super) const fn local(data: Value) -> Self {
        Self {
            data,
            profile: None,
            redmine_url: None,
        }
    }

    pub(super) fn into_result(self, operation: impl Into<String>) -> CommandResult {
        CommandResult::success(operation, self.data, self.profile, self.redmine_url)
    }
}
