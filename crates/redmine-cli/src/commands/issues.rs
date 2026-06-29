use std::path::PathBuf;

use serde_json::{json, Map, Value};

use crate::cli::IssuesSubcommand;
use crate::config::Profile;
use crate::error::AgentError;
use crate::redmine_client::RedmineClient;

const ISSUE_COLLECTION_PATH: &str = "issues.json";

pub(super) async fn run(
    client: &RedmineClient,
    profile: &Profile,
    command: IssuesSubcommand,
) -> Result<Value, AgentError> {
    match command {
        IssuesSubcommand::Get { issue_id } => client.issue_get(issue_id).await,
        IssuesSubcommand::List {
            project,
            status,
            limit,
        } => list(client, profile, project, status, limit).await,
        IssuesSubcommand::Create {
            project,
            subject,
            description,
            description_file,
            tracker_id,
            status_id,
            priority_id,
            assigned_to_id,
            dry_run,
        } => {
            let request = CreateIssueRequest {
                project,
                subject,
                description,
                description_file,
                tracker_id,
                status_id,
                priority_id,
                assigned_to_id,
            };
            create(client, request, dry_run).await
        }
        IssuesSubcommand::Update {
            issue_id,
            subject,
            description,
            status_id,
            priority_id,
            assigned_to_id,
            notes,
            dry_run,
        } => {
            let request = UpdateIssueRequest {
                subject,
                description,
                status_id,
                priority_id,
                assigned_to_id,
                notes,
            };
            update(client, issue_id, request, dry_run).await
        }
        IssuesSubcommand::Comment {
            issue_id,
            notes,
            dry_run,
        } => comment(client, issue_id, notes, dry_run).await,
    }
}

async fn list(
    client: &RedmineClient,
    profile: &Profile,
    project: Option<String>,
    status: Option<String>,
    limit: u32,
) -> Result<Value, AgentError> {
    let project = project.or_else(|| profile.default_project.clone());
    client
        .issues_list(project.as_deref(), status.as_deref(), limit)
        .await
}

async fn create(
    client: &RedmineClient,
    request: CreateIssueRequest,
    dry_run: bool,
) -> Result<Value, AgentError> {
    let issue = request.into_payload()?;

    if dry_run {
        Ok(issue_preview("POST", ISSUE_COLLECTION_PATH, issue))
    } else {
        client.issue_create(issue).await
    }
}

async fn update(
    client: &RedmineClient,
    issue_id: u64,
    request: UpdateIssueRequest,
    dry_run: bool,
) -> Result<Value, AgentError> {
    let issue = request.into_payload()?;
    let path = issue_path(issue_id);

    if dry_run {
        Ok(issue_preview("PUT", path, issue))
    } else {
        client.issue_update(issue_id, issue).await
    }
}

async fn comment(
    client: &RedmineClient,
    issue_id: u64,
    notes: String,
    dry_run: bool,
) -> Result<Value, AgentError> {
    let issue = IssuePayload::new().string("notes", notes).into_value();
    let path = issue_path(issue_id);

    if dry_run {
        Ok(issue_preview("PUT", path, issue))
    } else {
        client.issue_update(issue_id, issue).await
    }
}

fn issue_path(issue_id: u64) -> String {
    format!("issues/{issue_id}.json")
}

fn issue_preview(method: &str, path: impl Into<String>, issue: Value) -> Value {
    json!({
        "dryRun": true,
        "method": method,
        "path": path.into(),
        "body": { "issue": issue },
    })
}

struct CreateIssueRequest {
    project: String,
    subject: String,
    description: Option<String>,
    description_file: Option<PathBuf>,
    tracker_id: Option<u64>,
    status_id: Option<u64>,
    priority_id: Option<u64>,
    assigned_to_id: Option<u64>,
}

impl CreateIssueRequest {
    fn into_payload(self) -> Result<Value, AgentError> {
        let payload = IssuePayload::new()
            .string("project_id", self.project)
            .string("subject", self.subject)
            .optional_string(
                "description",
                read_description(self.description, self.description_file)?,
            )
            .optional_u64("tracker_id", self.tracker_id)
            .optional_u64("status_id", self.status_id)
            .optional_u64("priority_id", self.priority_id)
            .optional_u64("assigned_to_id", self.assigned_to_id);

        Ok(payload.into_value())
    }
}

struct UpdateIssueRequest {
    subject: Option<String>,
    description: Option<String>,
    status_id: Option<u64>,
    priority_id: Option<u64>,
    assigned_to_id: Option<u64>,
    notes: Option<String>,
}

impl UpdateIssueRequest {
    fn into_payload(self) -> Result<Value, AgentError> {
        IssuePayload::new()
            .optional_string("subject", self.subject)
            .optional_string("description", self.description)
            .optional_u64("status_id", self.status_id)
            .optional_u64("priority_id", self.priority_id)
            .optional_u64("assigned_to_id", self.assigned_to_id)
            .optional_string("notes", self.notes)
            .try_into_value()
    }
}

struct IssuePayload {
    fields: Map<String, Value>,
}

impl IssuePayload {
    fn new() -> Self {
        Self { fields: Map::new() }
    }

    fn string(mut self, key: &str, value: String) -> Self {
        self.fields.insert(key.to_string(), json!(value));
        self
    }

    fn optional_string(self, key: &str, value: Option<String>) -> Self {
        match value {
            Some(value) => self.string(key, value),
            None => self,
        }
    }

    fn optional_u64(mut self, key: &str, value: Option<u64>) -> Self {
        if let Some(value) = value {
            self.fields.insert(key.to_string(), json!(value));
        }

        self
    }

    fn try_into_value(self) -> Result<Value, AgentError> {
        if self.fields.is_empty() {
            Err(AgentError::InvalidInput(
                "At least one update field is required.".to_string(),
            ))
        } else {
            Ok(self.into_value())
        }
    }

    fn into_value(self) -> Value {
        Value::Object(self.fields)
    }
}

fn read_description(
    description: Option<String>,
    description_file: Option<PathBuf>,
) -> Result<Option<String>, AgentError> {
    match (description, description_file) {
        (Some(_), Some(_)) => Err(AgentError::InvalidInput(
            "Use either --description or --description-file, not both.".to_string(),
        )),
        (Some(description), None) => Ok(Some(description)),
        (None, Some(path)) => Ok(Some(std::fs::read_to_string(path)?)),
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_payload_includes_required_and_optional_fields() {
        let payload = CreateIssueRequest {
            project: "ops".to_string(),
            subject: "Investigate outage".to_string(),
            description: Some("Details".to_string()),
            description_file: None,
            tracker_id: Some(1),
            status_id: None,
            priority_id: Some(3),
            assigned_to_id: Some(42),
        }
        .into_payload()
        .expect("payload should be valid");

        assert_eq!(
            payload,
            json!({
                "project_id": "ops",
                "subject": "Investigate outage",
                "description": "Details",
                "tracker_id": 1,
                "priority_id": 3,
                "assigned_to_id": 42,
            })
        );
    }

    #[test]
    fn update_payload_rejects_empty_updates() {
        let error = UpdateIssueRequest {
            subject: None,
            description: None,
            status_id: None,
            priority_id: None,
            assigned_to_id: None,
            notes: None,
        }
        .into_payload()
        .expect_err("empty update should be rejected");

        assert_eq!(error.code(), "INVALID_INPUT");
    }

    #[test]
    fn read_description_rejects_two_sources() {
        let error = read_description(
            Some("inline".to_string()),
            Some(PathBuf::from("description.md")),
        )
        .expect_err("two description sources should be rejected");

        assert_eq!(error.code(), "INVALID_INPUT");
    }

    #[test]
    fn issue_preview_wraps_issue_payload() {
        let preview = issue_preview("PUT", "issues/10.json", json!({ "notes": "Done" }));

        assert_eq!(
            preview,
            json!({
                "dryRun": true,
                "method": "PUT",
                "path": "issues/10.json",
                "body": {
                    "issue": {
                        "notes": "Done",
                    },
                },
            })
        );
    }
}
