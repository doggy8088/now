use crate::config::{
    DEFAULT_AZURE_BLOB_SAS_URL_ENV, DEFAULT_AZURE_SWA_DEPLOYMENT_TOKEN_ENV, ProviderKind,
    default_config, get_key, local_config_path, merge_values, read_json_file, remove_key, set_key,
    write_json_file,
};
use crate::env_file::{local_env_path, validate_env_name, write_env_value};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::io::{BufRead, Write};
use std::path::Path;

pub fn run_first_run_setup<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> Result<ProviderKind> {
    let path = local_config_path(root);
    writeln!(output, "No provider is configured for now.")?;
    writeln!(
        output,
        "This first-time setup writes settings to .now.json."
    )?;
    writeln!(
        output,
        "Keep tokens, passwords, and account keys in provider login state or environment variables."
    )?;
    writeln!(output)?;

    let provider = run_setup(root, &path, input, output)?;
    writeln!(output, "Continuing with deployment.")?;

    Ok(provider)
}

pub fn run_init_setup<R: BufRead, W: Write>(
    root: &Path,
    path: &Path,
    input: &mut R,
    output: &mut W,
) -> Result<ProviderKind> {
    writeln!(output, "Configure now interactively.")?;
    writeln!(
        output,
        "Keep tokens, passwords, and account keys in provider login state or environment variables."
    )?;
    writeln!(output)?;

    let provider = run_setup(root, path, input, output)?;
    writeln!(output, "Configuration complete. No deployment was started.")?;

    Ok(provider)
}

fn run_setup<R: BufRead, W: Write>(
    root: &Path,
    path: &Path,
    input: &mut R,
    output: &mut W,
) -> Result<ProviderKind> {
    let config_existed = path.exists();
    let existing = read_json_file(path)?;
    let mut config = default_config();
    merge_values(&mut config, existing);
    let configured_provider = get_string(&config, "provider")
        .as_deref()
        .and_then(ProviderKind::parse);

    let provider = prompt_provider(input, output, configured_provider)?;
    set_key(
        &mut config,
        "provider",
        Value::String(provider.as_str().to_owned()),
    )?;

    if provider != ProviderKind::AzureBlob {
        prompt_common_settings(input, output, &mut config)?;
    }
    let mut env_secret = None;
    match provider {
        ProviderKind::Firebase => prompt_firebase(input, output, &mut config)?,
        ProviderKind::AzureBlob => {
            env_secret = Some(prompt_azure_blob(root, input, output, &mut config)?);
        }
        ProviderKind::AzureSwa => {
            env_secret = Some(prompt_azure_swa(root, input, output, &mut config)?);
        }
        ProviderKind::Ftp => prompt_ftp(input, output, &mut config)?,
    }

    write_json_file(path, &config)?;
    if let Some((env_name, secret_value)) = env_secret {
        let env_path = local_env_path(root);
        let exists = env_path.exists();
        write_env_value(&env_path, &env_name, &secret_value)?;
        if exists {
            writeln!(output, "Updated {}", env_path.display())?;
        } else {
            writeln!(output, "Created {}", env_path.display())?;
        }
    }
    writeln!(output)?;
    let action = if config_existed { "Updated" } else { "Created" };
    writeln!(output, "{action} {}", path.display())?;

    Ok(provider)
}

fn prompt_provider<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    configured: Option<ProviderKind>,
) -> Result<ProviderKind> {
    writeln!(output, "Choose a provider:")?;
    writeln!(output, "  1. Firebase Hosting")?;
    writeln!(output, "  2. Azure Storage Blob")?;
    writeln!(output, "  3. Azure Static Web App")?;
    writeln!(output, "  4. Any Website (FTP)")?;

    loop {
        let default = configured.unwrap_or(ProviderKind::Firebase);
        let prompt_text = format!("Provider [{}]: ", default.display_name());
        let answer = prompt(input, output, &prompt_text)?;
        let answer = answer.as_deref().map(str::trim).unwrap_or_default();
        if answer.is_empty() {
            return Ok(default);
        }
        if let Some(provider) = parse_provider_choice(answer) {
            return Ok(provider);
        }
        writeln!(
            output,
            "Enter 1, 2, 3, 4, firebase-hosting, Firebase Hosting, azure-storage-blob, Azure Storage Blob, azure-static-web-app, Azure Static Web App, any-website-ftp, or Any Website (FTP)."
        )?;
    }
}

fn prompt_common_settings<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &mut Value,
) -> Result<()> {
    let base_url_default = get_string(config, "base_url");
    let base_url = prompt_optional(input, output, "Base URL", base_url_default.as_deref())?;
    set_optional_string(config, "base_url", base_url)?;

    let default_url_default = get_string(config, "default_url");
    let default_url = prompt_optional(
        input,
        output,
        "Default URL override",
        default_url_default.as_deref(),
    )?;
    set_optional_string(config, "default_url", default_url)?;

    Ok(())
}

fn prompt_firebase<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &mut Value,
) -> Result<()> {
    let project = prompt_optional(
        input,
        output,
        "Firebase project",
        get_string(config, "firebase.project").as_deref(),
    )?;
    set_optional_string(config, "firebase.project", project)?;

    let site = prompt_optional(
        input,
        output,
        "Firebase hosting site",
        get_string(config, "firebase.site").as_deref(),
    )?;
    set_optional_string(config, "firebase.site", site)?;

    Ok(())
}

fn prompt_azure_blob<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
    config: &mut Value,
) -> Result<(String, String)> {
    writeln!(
        output,
        "Azure Storage Blob SAS URL includes upload credentials. It will be saved to .env, while .now.json only stores the environment variable name."
    )?;
    let env_name = prompt_optional(
        input,
        output,
        "Azure Storage Blob SAS URL environment variable",
        get_string(config, "azure_blob.sas_url_env")
            .as_deref()
            .or(Some(DEFAULT_AZURE_BLOB_SAS_URL_ENV)),
    )?
    .unwrap_or_else(|| DEFAULT_AZURE_BLOB_SAS_URL_ENV.to_owned());
    validate_env_name(&env_name)?;

    let local_env = crate::env_file::read_local_env(root).ok();
    let existing_sas_url = crate::env_file::env_value(&env_name, local_env.as_ref());

    let sas_url = if let Some(existing) = &existing_sas_url {
        let masked =
            crate::azure_blob::mask_sas_url(existing).unwrap_or_else(|_| "<invalid>".to_owned());
        let prompt_text = format!("Azure Storage Blob container SAS URL [{masked}]: ");
        let answer = prompt(input, output, &prompt_text)?;
        match answer {
            Some(answer) if !answer.trim().is_empty() => answer.trim().to_owned(),
            _ => existing.clone(),
        }
    } else {
        prompt_required(input, output, "Azure Storage Blob container SAS URL", None)?
    };

    set_key(
        config,
        "azure_blob.sas_url_env",
        Value::String(env_name.clone()),
    )?;
    remove_key(config, "azure_blob.sas_url");

    let prefix = prompt_optional(
        input,
        output,
        "Azure Storage Blob folder prefix",
        get_string(config, "azure_blob.prefix").as_deref(),
    )?;
    set_optional_string(config, "azure_blob.prefix", prefix)?;

    Ok((env_name, sas_url))
}

fn prompt_azure_swa<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
    config: &mut Value,
) -> Result<(String, String)> {
    writeln!(
        output,
        "Azure Static Web App deployment token will be saved to .env, while .now.json only stores the environment variable name."
    )?;
    let app_name = prompt_optional(
        input,
        output,
        "Azure Static Web App app name",
        get_string(config, "azure_swa.app_name").as_deref(),
    )?;
    set_optional_string(config, "azure_swa.app_name", app_name)?;

    let environment = prompt_optional(
        input,
        output,
        "Azure Static Web App environment",
        get_string(config, "azure_swa.environment")
            .as_deref()
            .or(Some("production")),
    )?
    .unwrap_or_else(|| "production".to_owned());
    set_key(config, "azure_swa.environment", Value::String(environment))?;

    let token_env = prompt_optional(
        input,
        output,
        "Azure Static Web App token environment variable",
        get_string(config, "azure_swa.deployment_token_env")
            .as_deref()
            .or(Some(DEFAULT_AZURE_SWA_DEPLOYMENT_TOKEN_ENV)),
    )?
    .unwrap_or_else(|| DEFAULT_AZURE_SWA_DEPLOYMENT_TOKEN_ENV.to_owned());
    validate_env_name(&token_env)?;
    set_key(
        config,
        "azure_swa.deployment_token_env",
        Value::String(token_env.clone()),
    )?;

    let local_env = crate::env_file::read_local_env(root).ok();
    let existing_token = crate::env_file::env_value(&token_env, local_env.as_ref());
    let token = if let Some(existing) = existing_token {
        let answer = prompt(
            input,
            output,
            "Azure Static Web App deployment token [configured]: ",
        )?;
        match answer {
            Some(answer) if !answer.trim().is_empty() => answer.trim().to_owned(),
            _ => existing,
        }
    } else {
        prompt_required(input, output, "Azure Static Web App deployment token", None)?
    };

    Ok((token_env, token))
}

fn prompt_ftp<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    config: &mut Value,
) -> Result<()> {
    let host = prompt_required(
        input,
        output,
        "FTP host",
        get_string(config, "ftp.host").as_deref(),
    )?;
    set_key(config, "ftp.host", Value::String(host))?;

    let remote_dir = prompt_optional(
        input,
        output,
        "FTP remote directory",
        get_string(config, "ftp.remote_dir")
            .as_deref()
            .or(Some("/")),
    )?
    .unwrap_or_else(|| "/".to_owned());
    set_key(config, "ftp.remote_dir", Value::String(remote_dir))?;

    let username_env = prompt_optional(
        input,
        output,
        "FTP username environment variable",
        get_string(config, "ftp.username_env")
            .as_deref()
            .or(Some("NOW_FTP_USERNAME")),
    )?
    .unwrap_or_else(|| "NOW_FTP_USERNAME".to_owned());
    set_key(config, "ftp.username_env", Value::String(username_env))?;

    let password_env = prompt_optional(
        input,
        output,
        "FTP password environment variable",
        get_string(config, "ftp.password_env")
            .as_deref()
            .or(Some("NOW_FTP_PASSWORD")),
    )?
    .unwrap_or_else(|| "NOW_FTP_PASSWORD".to_owned());
    set_key(config, "ftp.password_env", Value::String(password_env))?;

    Ok(())
}

fn prompt_optional<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    label: &str,
    default: Option<&str>,
) -> Result<Option<String>> {
    let prompt_text = match default {
        Some(default) if !default.trim().is_empty() => format!("{label} [{default}]: "),
        _ => format!("{label} (optional): "),
    };
    let answer = prompt(input, output, &prompt_text)?;
    match answer {
        Some(answer) if !answer.trim().is_empty() => Ok(Some(answer.trim().to_owned())),
        _ => Ok(default
            .map(str::to_owned)
            .filter(|value| !value.trim().is_empty())),
    }
}

fn prompt_required<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    label: &str,
    default: Option<&str>,
) -> Result<String> {
    loop {
        let prompt_text = match default {
            Some(default) if !default.trim().is_empty() => format!("{label} [{default}]: "),
            _ => format!("{label}: "),
        };
        let answer = prompt(input, output, &prompt_text)?;
        if answer.is_none() && default.is_none() {
            bail!("{label} is required");
        }
        if let Some(answer) = answer {
            let answer = answer.trim();
            if !answer.is_empty() {
                return Ok(answer.to_owned());
            }
        }
        if let Some(default) = default.filter(|value| !value.trim().is_empty()) {
            return Ok(default.to_owned());
        }
        writeln!(output, "{label} is required.")?;
    }
}

fn prompt<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    prompt_text: &str,
) -> Result<Option<String>> {
    write!(output, "{prompt_text}")?;
    output.flush()?;

    let mut answer = String::new();
    let bytes = input
        .read_line(&mut answer)
        .context("failed to read setup answer")?;
    if bytes == 0 {
        return Ok(None);
    }
    Ok(Some(answer.trim_end_matches(['\r', '\n']).to_owned()))
}

fn parse_provider_choice(value: &str) -> Option<ProviderKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "1" | "firebase-hosting" | "firebase hosting" | "firebase" => {
            Some(ProviderKind::Firebase)
        }
        "2" | "azure-storage-blob" | "azure storage blob" | "azure-blob" | "azure_blob" => {
            Some(ProviderKind::AzureBlob)
        }
        "3" | "azure-static-web-app" | "azure static web app" | "azure-swa" | "azure_swa" => {
            Some(ProviderKind::AzureSwa)
        }
        "4" | "any-website-ftp" | "any website (ftp)" | "any website ftp" | "ftp" => {
            Some(ProviderKind::Ftp)
        }
        _ => None,
    }
}

fn get_string(value: &Value, key: &str) -> Option<String> {
    get_key(value, key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn set_optional_string(config: &mut Value, key: &str, value: Option<String>) -> Result<()> {
    if let Some(value) = value {
        set_key(config, key, Value::String(value))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use serde_json::json;
    use std::io::Cursor;

    #[test]
    fn first_run_setup_writes_firebase_config_without_secrets() {
        let temp = TempDir::new().unwrap();
        let answers = b"1\nhttps://example.web.app\n\nmy-project\n\n";
        let mut input = Cursor::new(answers.as_slice());
        let mut output = Vec::new();

        let provider = run_first_run_setup(temp.path(), &mut input, &mut output).unwrap();
        let config = read_json_file(&local_config_path(temp.path())).unwrap();

        assert_eq!(provider, ProviderKind::Firebase);
        assert_eq!(
            get_key(&config, "provider"),
            Some(&json!("firebase-hosting"))
        );
        assert_eq!(
            get_key(&config, "base_url"),
            Some(&json!("https://example.web.app"))
        );
        assert_eq!(
            get_key(&config, "firebase.project"),
            Some(&json!("my-project"))
        );
        assert!(String::from_utf8(output).unwrap().contains(".now.json"));
    }

    #[test]
    fn first_run_setup_writes_azure_blob_sas_url_to_env_file() {
        let temp = TempDir::new().unwrap();
        let answers = b"2\n\nhttps://acct.blob.core.windows.net/$web?sv=1&sig=secret\nmy-prefix\n";
        let mut input = Cursor::new(answers.as_slice());
        let mut output = Vec::new();

        let provider = run_first_run_setup(temp.path(), &mut input, &mut output).unwrap();
        let config = read_json_file(&local_config_path(temp.path())).unwrap();

        assert_eq!(provider, ProviderKind::AzureBlob);
        assert_eq!(
            get_key(&config, "azure_blob.sas_url_env"),
            Some(&json!("NOW_AZURE_BLOB_SAS_URL"))
        );
        assert_eq!(
            get_key(&config, "azure_blob.prefix"),
            Some(&json!("my-prefix"))
        );
        assert_eq!(get_key(&config, "azure_blob.sas_url"), None);

        let env_text = std::fs::read_to_string(temp.path().join(".env")).unwrap();
        assert!(env_text.contains("NOW_AZURE_BLOB_SAS_URL="));
        assert!(env_text.contains("sig=secret"));

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("SAS URL"));
        assert!(output.contains(".env"));
        assert!(!output.contains("sig=secret"));
    }

    #[test]
    fn first_run_setup_uses_existing_sas_url_from_env_file_as_default() {
        let temp = TempDir::new().unwrap();
        let env_path = temp.path().join(".env");
        std::fs::write(
            &env_path,
            "NOW_AZURE_BLOB_SAS_URL=\"https://acct.blob.core.windows.net/$web?sv=1&sig=secret\"\n",
        )
        .unwrap();

        let answers = b"2\n\n\nmy-prefix\n";
        let mut input = Cursor::new(answers.as_slice());
        let mut output = Vec::new();

        let provider = run_first_run_setup(temp.path(), &mut input, &mut output).unwrap();
        let config = read_json_file(&local_config_path(temp.path())).unwrap();

        assert_eq!(provider, ProviderKind::AzureBlob);
        assert_eq!(
            get_key(&config, "azure_blob.sas_url_env"),
            Some(&json!("NOW_AZURE_BLOB_SAS_URL"))
        );
        assert_eq!(
            get_key(&config, "azure_blob.prefix"),
            Some(&json!("my-prefix"))
        );

        let env_text = std::fs::read_to_string(&env_path).unwrap();
        assert!(env_text.contains("NOW_AZURE_BLOB_SAS_URL="));
        assert!(env_text.contains("sig=secret"));

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("https://acct.blob.core.windows.net/$web?<redacted>"));
    }

    #[test]
    fn first_run_setup_writes_azure_swa_deployment_token_to_env_file() {
        let temp = TempDir::new().unwrap();
        let answers = b"3\n\n\nmy-app\n\n\nsecret-deployment-token\n";
        let mut input = Cursor::new(answers.as_slice());
        let mut output = Vec::new();

        let provider = run_first_run_setup(temp.path(), &mut input, &mut output).unwrap();
        let config = read_json_file(&local_config_path(temp.path())).unwrap();

        assert_eq!(provider, ProviderKind::AzureSwa);
        assert_eq!(
            get_key(&config, "azure_swa.deployment_token_env"),
            Some(&json!("SWA_CLI_DEPLOYMENT_TOKEN"))
        );
        assert_eq!(get_key(&config, "azure_swa.deployment_token"), None);

        let env_text = std::fs::read_to_string(temp.path().join(".env")).unwrap();
        assert!(env_text.contains("SWA_CLI_DEPLOYMENT_TOKEN="));
        assert!(env_text.contains("secret-deployment-token"));

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("deployment token"));
        assert!(output.contains(".env"));
        assert!(!output.contains("secret-deployment-token"));
    }
}
