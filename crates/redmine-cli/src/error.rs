use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("configuration is missing")]
    MissingConfig,

    #[error("profile was not found: {0}")]
    MissingProfile(String),

    #[error("required environment variable is missing: {0}")]
    MissingEnv(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Redmine API error")]
    Redmine {
        status: u16,
        message: String,
        details: Vec<String>,
    },
}

impl AgentError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MissingConfig => "CONFIG_NOT_FOUND",
            Self::MissingProfile(_) => "CONFIG_PROFILE_NOT_FOUND",
            Self::MissingEnv(_) => "CONFIG_ENV_MISSING",
            Self::InvalidConfig(_) => "CONFIG_INVALID",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::Io(_) => "IO_ERROR",
            Self::TomlSer(_) | Self::TomlDe(_) => "CONFIG_PARSE_ERROR",
            Self::Json(_) => "JSON_ERROR",
            Self::Http(_) => "HTTP_ERROR",
            Self::Redmine { status, .. } if *status == 422 => "REDMINE_VALIDATION_ERROR",
            Self::Redmine { status, .. } if *status == 401 || *status == 403 => {
                "REDMINE_AUTH_ERROR"
            }
            Self::Redmine { .. } => "REDMINE_API_ERROR",
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::MissingConfig => "No redmine-cli config file was found.".to_string(),
            Self::MissingProfile(profile) => format!("Profile `{profile}` was not found."),
            Self::MissingEnv(name) => format!("Environment variable `{name}` is not set."),
            Self::InvalidConfig(message)
            | Self::InvalidInput(message)
            | Self::Redmine { message, .. } => message.clone(),
            Self::Io(error) => error.to_string(),
            Self::TomlSer(error) => error.to_string(),
            Self::TomlDe(error) => error.to_string(),
            Self::Json(error) => error.to_string(),
            Self::Http(error) => error.to_string(),
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Redmine { status, .. } => Some(*status),
            Self::Http(error) => error.status().map(|status| status.as_u16()),
            _ => None,
        }
    }

    pub fn retryable(&self) -> bool {
        match self {
            Self::Http(error) => error.is_timeout() || error.is_connect(),
            Self::Redmine { status, .. } => *status == 429 || *status >= 500,
            _ => false,
        }
    }

    pub fn details(&self) -> Vec<String> {
        match self {
            Self::Redmine { details, .. } => details.clone(),
            _ => Vec::new(),
        }
    }

    pub fn hint(&self) -> Option<String> {
        match self {
            Self::MissingConfig => Some(
                "Run `redmine-cli config init --url https://redmine.example.com` first."
                    .to_string(),
            ),
            Self::MissingEnv(name) => Some(format!(
                "Set `{name}` in the environment before running Redmine API commands."
            )),
            Self::Redmine { status: 422, .. } => {
                Some("Fetch Redmine metadata and retry with instance-specific IDs.".to_string())
            }
            Self::Redmine {
                status: 401 | 403, ..
            } => Some("Check the configured Redmine URL and API key.".to_string()),
            Self::Http(error) if is_likely_tls_error(error) => Some(
                "TLS certificate validation failed. Check the Redmine certificate chain and root CA, or retry with `--ssl-no-revoke` if revocation checks are failing.".to_string(),
            ),
            _ => None,
        }
    }
}

fn is_likely_tls_error(error: &reqwest::Error) -> bool {
    message_looks_like_tls_error(&error.to_string())
}

fn message_looks_like_tls_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();

    message.contains("tls")
        || message.contains("certificate")
        || message.contains("cert")
        || message.contains("schannel")
        || message.contains("rustls")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_error_detection_matches_common_messages() {
        let samples = [
            "error sending request for url: error trying to connect: invalid peer certificate",
            "builder error: invalid TLS identity",
            "schannel: failed to verify certificate revocation",
            "rustls error: invalid certificate",
        ];

        for sample in samples {
            assert!(message_looks_like_tls_error(sample), "{sample}");
        }
    }
}
