use serde_json::Value;

use crate::cli::ProjectsSubcommand;
use crate::error::AgentError;
use crate::redmine_client::RedmineClient;

pub(super) async fn run(
    client: &RedmineClient,
    command: ProjectsSubcommand,
) -> Result<Value, AgentError> {
    match command {
        ProjectsSubcommand::List { limit } => client.projects_list(limit).await,
        ProjectsSubcommand::Get { project_id } => client.project_get(&project_id).await,
    }
}
