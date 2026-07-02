use std::ffi::OsStr;
use std::io::{Cursor, IsTerminal, Write};
use std::path::{Path, PathBuf};

use reqwest::header::{ACCEPT, USER_AGENT};
use semver::Version;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::cli::UpdateCommand;
use crate::error::AgentError;

const USER_AGENT_VALUE: &str = concat!("redmine-cli/", env!("CARGO_PKG_VERSION"));
const GITHUB_API_BASE: &str = "https://api.github.com/repos";

pub(super) async fn run(command: UpdateCommand, timeout_ms: u64) -> Result<Value, AgentError> {
    update(
        command.repo,
        command.tag,
        command.force,
        command.dry_run,
        command.confirm,
        timeout_ms,
    )
    .await
}

async fn update(
    repo: Option<String>,
    tag: Option<String>,
    force: bool,
    dry_run: bool,
    confirm: bool,
    timeout_ms: u64,
) -> Result<Value, AgentError> {
    let repo = match repo {
        Some(repo) => validate_repo(&repo)?,
        None => default_repo()?,
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()?;
    let release = fetch_release(&client, &repo, tag.as_deref()).await?;
    let package_asset_name = package_asset_name()?;
    let package_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == package_asset_name)
        .ok_or_else(|| {
            AgentError::InvalidInput(format!(
                "Release `{}` does not contain `{}`.",
                release.tag_name, package_asset_name
            ))
        })?;
    let checksum_asset_name = format!("{}.sha256", package_asset.name);
    let checksum_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == checksum_asset_name)
        .ok_or_else(|| {
            AgentError::InvalidInput(format!(
                "Release `{}` does not contain `{checksum_asset_name}`.",
                release.tag_name
            ))
        })?;

    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| {
        AgentError::InvalidConfig(format!("Current package version is invalid: {error}"))
    })?;
    let release_version = parse_release_version(&release.tag_name)?;
    let up_to_date = release_version <= current_version && !force;

    if dry_run || up_to_date {
        return Ok(json!({
            "dryRun": dry_run,
            "updated": false,
            "upToDate": up_to_date,
            "currentVersion": env!("CARGO_PKG_VERSION"),
            "releaseTag": release.tag_name,
            "repo": repo,
            "asset": package_asset.name,
            "checksumAsset": checksum_asset.name,
            "target": current_target(),
        }));
    }

    confirm_update(confirm, &release.tag_name, &package_asset.name)?;

    let package = download(&client, &package_asset.browser_download_url).await?;
    let checksum = download_text(&client, &checksum_asset.browser_download_url).await?;
    verify_checksum(&package, &checksum, &package_asset.name)?;

    let temp_dir = tempfile::Builder::new()
        .prefix("redmine-cli-update-")
        .tempdir()?;
    let replacement_path = extract_binary(&package, temp_dir.path())?;
    let current_exe = std::env::current_exe()?;
    self_replace::self_replace(&replacement_path)?;

    #[cfg(windows)]
    {
        std::mem::forget(temp_dir);
    }

    Ok(json!({
        "dryRun": false,
        "updated": true,
        "upToDate": false,
        "currentVersion": env!("CARGO_PKG_VERSION"),
        "releaseTag": release.tag_name,
        "repo": repo,
        "asset": package_asset.name,
        "checksumAsset": checksum_asset.name,
        "target": current_target(),
        "installedPath": current_exe,
    }))
}

fn confirm_update(confirm: bool, release_tag: &str, asset_name: &str) -> Result<(), AgentError> {
    if confirm {
        return Ok(());
    }

    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return Err(AgentError::InvalidInput(
            "`update` modifies the installed binary; pass `--confirm` in non-interactive environments."
                .to_string(),
        ));
    }

    eprint!("Update redmine-cli to {release_tag} using {asset_name}? [y/N] ");
    std::io::stderr().flush()?;

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(()),
        _ => Err(AgentError::InvalidInput("Update cancelled.".to_string())),
    }
}

async fn fetch_release(
    client: &reqwest::Client,
    repo: &str,
    tag: Option<&str>,
) -> Result<GithubRelease, AgentError> {
    let url = tag.map_or_else(
        || format!("{GITHUB_API_BASE}/{repo}/releases/latest"),
        |tag| format!("{GITHUB_API_BASE}/{repo}/releases/tags/{tag}"),
    );

    Ok(client
        .get(url)
        .header(USER_AGENT, USER_AGENT_VALUE)
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

async fn download(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, AgentError> {
    Ok(client
        .get(url)
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}

async fn download_text(client: &reqwest::Client, url: &str) -> Result<String, AgentError> {
    let bytes = download(client, url).await?;
    String::from_utf8(bytes)
        .map_err(|error| AgentError::InvalidInput(format!("Checksum is not UTF-8: {error}")))
}

fn verify_checksum(
    package: &[u8],
    checksum_text: &str,
    asset_name: &str,
) -> Result<(), AgentError> {
    let expected = checksum_text
        .split_whitespace()
        .next()
        .ok_or_else(|| AgentError::InvalidInput("Checksum file is empty.".to_string()))?;
    if expected.len() != 64
        || !expected
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(AgentError::InvalidInput(
            "Checksum file does not start with a SHA256 hex digest.".to_string(),
        ));
    }

    let actual = format!("{:x}", Sha256::digest(package));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(AgentError::InvalidInput(format!(
            "Checksum mismatch for `{asset_name}`."
        )));
    }

    Ok(())
}

#[cfg(windows)]
fn extract_binary(package: &[u8], output_dir: &Path) -> Result<PathBuf, AgentError> {
    let reader = Cursor::new(package);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|error| AgentError::InvalidInput(format!("Invalid zip archive: {error}")))?;

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| AgentError::InvalidInput(format!("Invalid zip entry: {error}")))?;
        if file
            .enclosed_name()
            .is_some_and(|path| path.file_name() == Some(OsStr::new(binary_name())))
        {
            let output_path = output_dir.join(binary_name());
            let mut output = std::fs::File::create(&output_path)?;
            std::io::copy(&mut file, &mut output)?;
            return Ok(output_path);
        }
    }

    Err(AgentError::InvalidInput(format!(
        "Archive does not contain `{}`.",
        binary_name()
    )))
}

#[cfg(not(windows))]
fn extract_binary(package: &[u8], output_dir: &Path) -> Result<PathBuf, AgentError> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(package));
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.file_name() == Some(OsStr::new(binary_name())) {
            let output_path = output_dir.join(binary_name());
            entry.unpack(&output_path)?;
            set_executable(&output_path)?;
            return Ok(output_path);
        }
    }

    Err(AgentError::InvalidInput(format!(
        "Archive does not contain `{}`.",
        binary_name()
    )))
}

#[cfg(not(windows))]
fn set_executable(path: &Path) -> Result<(), AgentError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn validate_repo(repo: &str) -> Result<String, AgentError> {
    let parts: Vec<_> = repo.split('/').collect();
    if parts.len() == 2 && parts.iter().all(|part| !part.is_empty()) {
        Ok(repo.to_string())
    } else {
        Err(AgentError::InvalidInput(
            "`--repo` must be in `owner/name` format.".to_string(),
        ))
    }
}

fn default_repo() -> Result<String, AgentError> {
    let repository = env!("CARGO_PKG_REPOSITORY");
    let path = repository
        .strip_prefix("https://github.com/")
        .ok_or_else(|| {
            AgentError::InvalidConfig(
                "Package repository is not a GitHub URL; pass `--repo owner/name`.".to_string(),
            )
        })?
        .trim_end_matches(".git");

    if path.starts_with("your-org/") {
        return Err(AgentError::InvalidConfig(
            "Package repository is a placeholder; pass `--repo owner/name`.".to_string(),
        ));
    }

    validate_repo(path)
}

fn parse_release_version(tag: &str) -> Result<Version, AgentError> {
    Version::parse(tag.trim_start_matches('v'))
        .map_err(|error| AgentError::InvalidInput(format!("Release tag is not semver: {error}")))
}

const fn current_target() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else {
        "unsupported"
    }
}

fn package_asset_name() -> Result<&'static str, AgentError> {
    match current_target() {
        "x86_64-pc-windows-msvc" => Ok("redmine-cli-x86_64-pc-windows-msvc.zip"),
        "x86_64-unknown-linux-gnu" => Ok("redmine-cli-x86_64-unknown-linux-gnu.tar.gz"),
        "x86_64-apple-darwin" => Ok("redmine-cli-x86_64-apple-darwin.tar.gz"),
        target => Err(AgentError::InvalidInput(format!(
            "Self update is not supported for target `{target}`."
        ))),
    }
}

const fn binary_name() -> &'static str {
    #[cfg(windows)]
    {
        "redmine-cli.exe"
    }
    #[cfg(not(windows))]
    {
        "redmine-cli"
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_repo_accepts_owner_name() {
        assert_eq!(validate_repo("owner/project").unwrap(), "owner/project");
    }

    #[test]
    fn validate_repo_rejects_url() {
        assert!(validate_repo("https://github.com/owner/project").is_err());
    }

    #[test]
    fn parse_release_version_accepts_v_prefix() {
        assert_eq!(
            parse_release_version("v1.2.3").unwrap(),
            Version::new(1, 2, 3)
        );
    }
}
