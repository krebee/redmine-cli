use serde::Serialize;

use crate::cli::OutputFormat;
use crate::error::AgentError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    ok: bool,
    operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
    meta: ResultMeta,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultMeta {
    profile: Option<String>,
    redmine_url: Option<String>,
    truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    code: String,
    message: String,
    status: Option<u16>,
    retryable: bool,
    details: Vec<String>,
    hint: Option<String>,
}

impl CommandResult {
    pub fn success(
        operation: impl Into<String>,
        data: serde_json::Value,
        profile: Option<String>,
        redmine_url: Option<String>,
    ) -> Self {
        Self {
            ok: true,
            operation: operation.into(),
            data: Some(data),
            error: None,
            meta: ResultMeta {
                profile,
                redmine_url,
                truncated: false,
            },
        }
    }

    pub fn failure(operation: impl Into<String>, error: &AgentError) -> Self {
        Self {
            ok: false,
            operation: operation.into(),
            data: None,
            error: Some(ErrorBody {
                code: error.code().to_string(),
                message: error.user_message(),
                status: error.status(),
                retryable: error.retryable(),
                details: error.details(),
                hint: error.hint(),
            }),
            meta: ResultMeta {
                profile: None,
                redmine_url: None,
                truncated: false,
            },
        }
    }
}

pub fn print_result(result: &CommandResult, format: &OutputFormat) -> Result<(), AgentError> {
    println!("{}", render_result(result, format)?);
    Ok(())
}

fn render_result(result: &CommandResult, format: &OutputFormat) -> Result<String, AgentError> {
    match format {
        OutputFormat::Json | OutputFormat::Text | OutputFormat::Table => {
            Ok(serde_json::to_string_pretty(result)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_success_uses_stable_agent_shape() {
        let result = CommandResult::success(
            "issues.get",
            serde_json::json!({ "issue": { "id": 123 } }),
            Some("default".to_string()),
            Some("https://redmine.example.com".to_string()),
        );

        let rendered = render_result(&result, &OutputFormat::Json).expect("result should render");
        let value: serde_json::Value =
            serde_json::from_str(&rendered).expect("rendered result should be JSON");

        assert_eq!(value["ok"], true);
        assert_eq!(value["operation"], "issues.get");
        assert_eq!(value["data"]["issue"]["id"], 123);
        assert_eq!(value["meta"]["profile"], "default");
    }

    #[test]
    fn render_failure_includes_retry_hint_fields() {
        let error = AgentError::MissingConfig;
        let result = CommandResult::failure("config.show", &error);
        let rendered = render_result(&result, &OutputFormat::Json).expect("result should render");
        let value: serde_json::Value =
            serde_json::from_str(&rendered).expect("rendered result should be JSON");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "CONFIG_NOT_FOUND");
        assert_eq!(value["error"]["retryable"], false);
        assert!(value["error"]["hint"].is_string());
    }
}
