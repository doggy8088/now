use crate::config::{NowConfig, ProviderKind};
use anyhow::{Context, Result, bail};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProviderCommand {
    pub provider: ProviderKind,
    pub program: String,
    pub args: Vec<String>,
    pub required_cli: String,
    pub required_env: Vec<String>,
    pub env_mappings: Vec<(String, String)>,
    pub cwd: Option<PathBuf>,
    display_program: String,
    display_args: Vec<String>,
}

impl ProviderCommand {
    pub fn new(
        provider: ProviderKind,
        program: impl Into<String>,
        args: Vec<String>,
        required_cli: impl Into<String>,
    ) -> Self {
        let program = program.into();
        Self {
            provider,
            display_program: program.clone(),
            display_args: args.clone(),
            program,
            args,
            required_cli: required_cli.into(),
            required_env: Vec::new(),
            env_mappings: Vec::new(),
            cwd: None,
        }
    }

    pub fn with_required_env(mut self, names: Vec<String>) -> Self {
        self.required_env = names;
        self
    }

    pub fn with_env_mapping(
        mut self,
        target_name: impl Into<String>,
        source_name: impl Into<String>,
    ) -> Self {
        self.env_mappings
            .push((target_name.into(), source_name.into()));
        self
    }

    pub fn with_display(mut self, program: impl Into<String>, args: Vec<String>) -> Self {
        self.display_program = program.into();
        self.display_args = args;
        self
    }

    pub fn display_line(&self) -> String {
        shell_join(
            std::iter::once(self.display_program.as_str())
                .chain(self.display_args.iter().map(String::as_str)),
        )
    }

    pub fn validate_environment(&self) -> Result<()> {
        let missing = self
            .required_env
            .iter()
            .filter(|name| {
                env::var(name.as_str())
                    .map(|value| value.is_empty())
                    .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            bail!(
                "missing required environment variable(s) for {}: {}",
                self.provider,
                missing.join(", ")
            );
        }

        Ok(())
    }

    pub fn apply_environment(&self, process: &mut std::process::Command) -> Result<()> {
        for (target_name, source_name) in &self.env_mappings {
            let value = env::var(source_name)
                .with_context(|| format!("missing required environment variable: {source_name}"))?;
            process.env(target_name, value);
        }
        Ok(())
    }
}

pub fn build_provider_command(
    provider: ProviderKind,
    config: &NowConfig,
    source: &Path,
) -> Result<ProviderCommand> {
    match provider {
        ProviderKind::Firebase => firebase_command(config),
        ProviderKind::AzureBlob => {
            bail!(
                "provider Azure Storage Blob uses built-in SAS URL upload and does not require a provider CLI"
            )
        }
        ProviderKind::AzureSwa => azure_swa_command(config, source),
        ProviderKind::Ftp => ftp_command(config, source),
    }
}

fn firebase_command(config: &NowConfig) -> Result<ProviderCommand> {
    let only = config
        .firebase
        .site
        .as_deref()
        .map(|site| format!("hosting:{site}"))
        .unwrap_or_else(|| "hosting".to_owned());

    let mut args = vec!["deploy".to_owned(), "--only".to_owned(), only];
    if let Some(project) = non_empty(config.firebase.project.as_deref()) {
        args.extend(["--project".to_owned(), project.to_owned()]);
    }

    Ok(ProviderCommand::new(
        ProviderKind::Firebase,
        "firebase",
        args,
        "firebase",
    ))
}

fn azure_swa_command(config: &NowConfig, source: &Path) -> Result<ProviderCommand> {
    let environment = non_empty(config.azure_swa.environment.as_deref()).unwrap_or("production");
    let token_env = non_empty(config.azure_swa.deployment_token_env.as_deref())
        .unwrap_or("SWA_CLI_DEPLOYMENT_TOKEN");

    let mut args = vec![
        "deploy".to_owned(),
        source.display().to_string(),
        "--env".to_owned(),
        environment.to_owned(),
    ];

    if let Some(app_name) = non_empty(config.azure_swa.app_name.as_deref()) {
        args.extend(["--app-name".to_owned(), app_name.to_owned()]);
    }

    Ok(
        ProviderCommand::new(ProviderKind::AzureSwa, "swa", args, "swa")
            .with_required_env(vec![token_env.to_owned()])
            .with_env_mapping("SWA_CLI_DEPLOYMENT_TOKEN", token_env),
    )
}

fn ftp_command(config: &NowConfig, source: &Path) -> Result<ProviderCommand> {
    let host = non_empty(config.ftp.host.as_deref())
        .context("ftp.host is required for provider Any Website (FTP)")?;
    let remote_dir = non_empty(config.ftp.remote_dir.as_deref()).unwrap_or("/");
    let username_env = non_empty(config.ftp.username_env.as_deref()).unwrap_or("NOW_FTP_USERNAME");
    let password_env = non_empty(config.ftp.password_env.as_deref()).unwrap_or("NOW_FTP_PASSWORD");

    let mirror_command = format!(
        "set net:max-retries 2; mirror -R --only-newer {} {}; bye",
        shell_quote(&source.display().to_string()),
        shell_quote(remote_dir)
    );
    let script = format!(
        "lftp -u \"${username_env}\",\"${password_env}\" {} -e {}",
        shell_quote(host),
        shell_quote(&mirror_command)
    );

    #[cfg(windows)]
    let (program, args) = ("cmd".to_owned(), vec!["/C".to_owned(), script.clone()]);
    #[cfg(not(windows))]
    let (program, args) = ("sh".to_owned(), vec!["-c".to_owned(), script.clone()]);

    Ok(
        ProviderCommand::new(ProviderKind::Ftp, program, args, "lftp")
            .with_required_env(vec![username_env.to_owned(), password_env.to_owned()])
            .with_display(
                "lftp",
                vec!["-e".to_owned(), mirror_command, host.to_owned()],
            ),
    )
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub fn provider_install_hint(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Firebase => "Install Firebase CLI with: npm install -g firebase-tools",
        ProviderKind::AzureBlob => {
            "Azure Storage Blob uses built-in SAS URL upload and does not require Azure CLI"
        }
        ProviderKind::AzureSwa => "Install SWA CLI with: npm install -g @azure/static-web-apps-cli",
        ProviderKind::Ftp => "Install lftp with your system package manager",
    }
}

pub fn program_available(program: &str) -> bool {
    if program.contains(std::path::MAIN_SEPARATOR) {
        return Path::new(program).is_file();
    }

    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    let candidates = executable_names(program);
    env::split_paths(&paths).any(|dir| candidates.iter().any(|name| dir.join(name).is_file()))
}

fn executable_names(program: &str) -> Vec<OsString> {
    #[cfg(windows)]
    {
        let path = Path::new(program);
        if path.extension().is_some() {
            return vec![OsString::from(program)];
        }

        let pathext = env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
        return env::split_paths(&pathext)
            .map(|extension| {
                let extension = extension.to_string_lossy();
                OsString::from(format!("{program}{extension}"))
            })
            .chain(std::iter::once(OsString::from(program)))
            .collect();
    }

    #[cfg(not(windows))]
    {
        vec![OsString::from(program)]
    }
}

fn shell_join<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    parts
        .into_iter()
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_./:$=@%+".contains(character))
    {
        return value.to_owned();
    }

    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FtpConfig, NowConfig};

    #[test]
    fn firebase_command_uses_hosting_only() {
        let config = NowConfig::default();
        let command =
            build_provider_command(ProviderKind::Firebase, &config, Path::new("public")).unwrap();

        assert_eq!(command.program, "firebase");
        assert_eq!(command.args, ["deploy", "--only", "hosting"]);
    }

    #[test]
    fn azure_blob_command_builder_is_not_used_for_native_upload() {
        let config = NowConfig::default();
        let error = build_provider_command(ProviderKind::AzureBlob, &config, Path::new("public"))
            .unwrap_err()
            .to_string();

        assert!(error.contains("built-in SAS URL upload"));
    }

    #[test]
    fn ftp_command_display_does_not_include_plaintext_secret() {
        let config = NowConfig {
            ftp: FtpConfig {
                host: Some("example.com".to_owned()),
                remote_dir: Some("/www".to_owned()),
                username_env: Some("NOW_FTP_USERNAME".to_owned()),
                password_env: Some("NOW_FTP_PASSWORD".to_owned()),
                base_url: None,
            },
            ..NowConfig::default()
        };
        let command =
            build_provider_command(ProviderKind::Ftp, &config, Path::new("public")).unwrap();

        assert!(!command.display_line().contains("super-secret"));
        assert!(command.display_line().contains("lftp"));
        assert_eq!(
            command.required_env,
            ["NOW_FTP_USERNAME", "NOW_FTP_PASSWORD"]
        );
    }

    #[test]
    fn azure_swa_maps_configured_token_env_to_cli_env() {
        let config = NowConfig {
            azure_swa: crate::config::AzureSwaConfig {
                deployment_token_env: Some("AZURE_STATIC_WEB_APPS_API_TOKEN".to_owned()),
                ..Default::default()
            },
            ..NowConfig::default()
        };

        let command =
            build_provider_command(ProviderKind::AzureSwa, &config, Path::new("public")).unwrap();

        assert_eq!(command.required_env, ["AZURE_STATIC_WEB_APPS_API_TOKEN"]);
        assert_eq!(
            command.env_mappings,
            [(
                "SWA_CLI_DEPLOYMENT_TOKEN".to_owned(),
                "AZURE_STATIC_WEB_APPS_API_TOKEN".to_owned()
            )]
        );
        assert!(!command.display_line().contains("TOKEN"));
    }
}
