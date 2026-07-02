use std::time::Duration;

use reqwest::{Client, Method};
use serde_json::{json, Value};

use crate::config::Profile;
use crate::error::AgentError;

const DEFAULT_OFFSET: &str = "0";
const MAX_PAGE_LIMIT: u32 = 100;

#[derive(Clone)]
pub struct RedmineClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl RedmineClient {
    pub fn new(
        profile: &Profile,
        timeout_ms: u64,
        ssl_no_revoke: bool,
    ) -> Result<Self, AgentError> {
        let api_key_env = profile.api_key_env_name();
        let api_key = std::env::var(api_key_env)
            .map_err(|_| AgentError::MissingEnv(api_key_env.to_string()))?;
        let client = build_client(timeout_ms, profile.ssl_no_revoke || ssl_no_revoke)?;

        Ok(Self {
            client,
            base_url: profile.url.trim_end_matches('/').to_string(),
            api_key,
        })
    }

    pub fn redmine_url(&self) -> String {
        self.base_url.clone()
    }

    pub async fn projects_list(&self, limit: u32) -> Result<Value, AgentError> {
        self.request(Method::GET, "projects.json", &page_query(limit), None)
            .await
    }

    pub async fn project_get(&self, project_id: &str) -> Result<Value, AgentError> {
        self.request(
            Method::GET,
            &format!("projects/{project_id}.json"),
            &[],
            None,
        )
        .await
    }

    pub async fn issues_list(
        &self,
        project: Option<&str>,
        status: Option<&str>,
        limit: u32,
    ) -> Result<Value, AgentError> {
        let mut query = page_query(limit);

        if let Some(project) = project {
            query.push(("project_id", project.to_string()));
        }

        if let Some(status) = status {
            query.push(("status_id", status.to_string()));
        }

        self.request(Method::GET, "issues.json", &query, None).await
    }

    pub async fn issue_get(&self, issue_id: u64) -> Result<Value, AgentError> {
        self.request(Method::GET, &format!("issues/{issue_id}.json"), &[], None)
            .await
    }

    pub async fn issue_create(&self, issue: Value) -> Result<Value, AgentError> {
        self.request(
            Method::POST,
            "issues.json",
            &[],
            Some(json!({ "issue": issue })),
        )
        .await
    }

    pub async fn issue_update(&self, issue_id: u64, issue: Value) -> Result<Value, AgentError> {
        self.request(
            Method::PUT,
            &format!("issues/{issue_id}.json"),
            &[],
            Some(json!({ "issue": issue })),
        )
        .await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value, AgentError> {
        let url = format!("{}/{}", self.base_url, path);
        let mut request = self
            .client
            .request(method, url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Accept", "application/json")
            .query(query);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        parse_response(status.as_u16(), status.is_success(), &text)
    }
}

fn build_client(timeout_ms: u64, ssl_no_revoke: bool) -> Result<Client, reqwest::Error> {
    let builder = Client::builder().timeout(Duration::from_millis(timeout_ms));
    let builder = if ssl_no_revoke {
        builder.use_rustls_tls()
    } else {
        builder
    };

    builder.build()
}

fn page_query(limit: u32) -> Vec<(&'static str, String)> {
    vec![
        ("limit", limit.min(MAX_PAGE_LIMIT).to_string()),
        ("offset", DEFAULT_OFFSET.to_string()),
    ]
}

fn parse_response(status: u16, is_success: bool, text: &str) -> Result<Value, AgentError> {
    if !is_success {
        let (message, details) = parse_redmine_error(text);
        return Err(AgentError::Redmine {
            status,
            message,
            details,
        });
    }

    if text.trim().is_empty() {
        Ok(json!({}))
    } else {
        Ok(serde_json::from_str(text)?)
    }
}

fn parse_redmine_error(text: &str) -> (String, Vec<String>) {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        if let Some(errors) = value.get("errors").and_then(Value::as_array) {
            let details = errors
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();

            if !details.is_empty() {
                return ("Redmine rejected the request.".to_string(), details);
            }
        }

        if let Some(error) = value.get("error").and_then(Value::as_str) {
            return (error.to_string(), Vec::new());
        }
    }

    if text.trim().is_empty() {
        (
            "Redmine returned an error without a response body.".to_string(),
            Vec::new(),
        )
    } else {
        (text.trim().to_string(), Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_query_caps_limit() {
        assert_eq!(
            page_query(250),
            vec![
                ("limit", MAX_PAGE_LIMIT.to_string()),
                ("offset", DEFAULT_OFFSET.to_string()),
            ]
        );
    }

    #[test]
    fn parse_response_returns_empty_object_for_empty_success_body() {
        let parsed = parse_response(204, true, "").expect("empty success should parse");

        assert_eq!(parsed, json!({}));
    }

    #[test]
    fn parse_response_normalizes_redmine_validation_errors() {
        let error = parse_response(422, false, r#"{"errors":["Status is invalid"]}"#)
            .expect_err("Redmine error should be normalized");

        assert_eq!(error.code(), "REDMINE_VALIDATION_ERROR");
        assert_eq!(error.details(), vec!["Status is invalid".to_string()]);
    }

    #[test]
    fn parse_redmine_error_falls_back_to_trimmed_body() {
        let (message, details) = parse_redmine_error("  upstream unavailable  ");

        assert_eq!(message, "upstream unavailable");
        assert!(details.is_empty());
    }
}
