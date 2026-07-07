use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value, json};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Firebase,
    AzureBlob,
    AzureSwa,
    Ftp,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Firebase => "firebase-hosting",
            Self::AzureBlob => "azure-storage-blob",
            Self::AzureSwa => "azure-static-web-app",
            Self::Ftp => "any-website-ftp",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Firebase => "Firebase Hosting",
            Self::AzureBlob => "Azure Storage Blob",
            Self::AzureSwa => "Azure Static Web App",
            Self::Ftp => "Any Website (FTP)",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match normalize_provider_name(value).as_str() {
            "firebase-hosting" | "firebase" => Some(Self::Firebase),
            "azure-storage-blob" | "azure-blob" => Some(Self::AzureBlob),
            "azure-static-web-app" | "azure-static-web-apps" | "azure-swa" => Some(Self::AzureSwa),
            "any-website-ftp" | "any-website-(ftp)" | "ftp" => Some(Self::Ftp),
            _ => None,
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display_name())
    }
}

impl Serialize for ProviderKind {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ProviderKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unsupported provider {value}; expected firebase-hosting, azure-storage-blob, azure-static-web-app, or any-website-ftp"
            ))
        })
    }
}

fn normalize_provider_name(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct NowConfig {
    pub provider: Option<ProviderKind>,
    pub source: Option<String>,
    pub base_url: Option<String>,
    pub default_url: Option<String>,
    pub firebase: FirebaseConfig,
    pub azure_blob: AzureBlobConfig,
    pub azure_swa: AzureSwaConfig,
    pub ftp: FtpConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FirebaseConfig {
    pub project: Option<String>,
    pub site: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AzureBlobConfig {
    pub sas_url: Option<String>,
    pub account: Option<String>,
    pub container: Option<String>,
    pub destination_path: Option<String>,
    pub overwrite: Option<bool>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AzureSwaConfig {
    pub app_name: Option<String>,
    pub environment: Option<String>,
    pub deployment_token_env: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FtpConfig {
    pub host: Option<String>,
    pub remote_dir: Option<String>,
    pub username_env: Option<String>,
    pub password_env: Option<String>,
    pub base_url: Option<String>,
}

impl NowConfig {
    pub fn provider_base_url(&self, provider: ProviderKind) -> Option<&str> {
        self.base_url
            .as_deref()
            .or(match provider {
                ProviderKind::Firebase => self.firebase.base_url.as_deref(),
                ProviderKind::AzureBlob => self.azure_blob.base_url.as_deref(),
                ProviderKind::AzureSwa => self.azure_swa.base_url.as_deref(),
                ProviderKind::Ftp => self.ftp.base_url.as_deref(),
            })
            .filter(|value| !value.trim().is_empty())
    }
}

pub fn default_config() -> Value {
    json!({
        "provider": null,
        "source": null,
        "base_url": null,
        "default_url": null,
        "firebase": {
            "project": null,
            "site": null
        },
        "azure_blob": {
            "sas_url": null
        },
        "azure_swa": {
            "environment": "production",
            "deployment_token_env": "SWA_CLI_DEPLOYMENT_TOKEN"
        },
        "ftp": {
            "host": null,
            "remote_dir": "/",
            "username_env": "NOW_FTP_USERNAME",
            "password_env": "NOW_FTP_PASSWORD"
        }
    })
}

pub fn local_config_path(root: &Path) -> PathBuf {
    root.join(".now.json")
}

pub fn global_config_path() -> Result<PathBuf> {
    if let Ok(config_home) = env::var("NOW_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("settings.json"));
    }

    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("now").join("settings.json"));
    }

    let home = dirs::home_dir().context("cannot resolve home directory for global config")?;
    Ok(home.join(".config").join("now").join("settings.json"))
}

pub fn read_json_file(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }

    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse JSON config {}", path.display()))?;
    if !value.is_object() {
        bail!("config file {} must contain a JSON object", path.display());
    }
    Ok(value)
}

pub fn write_json_file(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let text = format!("{}\n", serde_json::to_string_pretty(value)?);
    fs::write(path, text).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn merge_values(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                merge_values(base_map.entry(key).or_insert(Value::Null), value);
            }
        }
        (base_slot, overlay_value) => {
            *base_slot = overlay_value;
        }
    }
}

pub fn merged_config_value(root: &Path, cli_provider: Option<ProviderKind>) -> Result<Value> {
    let mut merged = read_json_file(&global_config_path()?)?;
    merge_values(&mut merged, read_json_file(&local_config_path(root))?);
    if let Some(provider) = cli_provider {
        set_key(
            &mut merged,
            "provider",
            Value::String(provider.as_str().to_owned()),
        )?;
    }
    Ok(merged)
}

pub fn parse_config(value: Value) -> Result<NowConfig> {
    let paths = secret_paths(&value);
    if !paths.is_empty() {
        bail!(
            "configuration contains secret-like keys; use provider login state or environment variables instead: {}",
            paths.join(", ")
        );
    }

    serde_json::from_value(value).context("failed to deserialize config")
}

pub fn parse_config_value(raw: &str) -> Value {
    if matches!(raw, "true" | "false" | "null")
        && let Ok(value) = serde_json::from_str(raw)
    {
        return value;
    }

    if (raw.starts_with('{') || raw.starts_with('[') || raw.starts_with('"'))
        && let Ok(value) = serde_json::from_str(raw)
    {
        return value;
    }

    if let Ok(number) = raw.parse::<i64>() {
        return Value::Number(number.into());
    }

    Value::String(raw.to_owned())
}

pub fn get_key<'a>(value: &'a Value, dotted_key: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in dotted_key.split('.') {
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

pub fn set_key(value: &mut Value, dotted_key: &str, new_value: Value) -> Result<()> {
    if dotted_key.trim().is_empty() {
        bail!("config key cannot be empty");
    }
    if is_secret_key(dotted_key) {
        bail!("refusing to write secret-like config key: {dotted_key}");
    }

    if !value.is_object() {
        *value = Value::Object(Map::new());
    }

    let mut current = value;
    let mut parts = dotted_key.split('.').peekable();
    while let Some(part) = parts.next() {
        if part.is_empty() {
            bail!("config key contains an empty path segment: {dotted_key}");
        }

        if parts.peek().is_none() {
            current
                .as_object_mut()
                .expect("current value is an object")
                .insert(part.to_owned(), new_value);
            return Ok(());
        }

        let next = current
            .as_object_mut()
            .expect("current value is an object")
            .entry(part.to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        if !next.is_object() {
            *next = Value::Object(Map::new());
        }
        current = next;
    }

    Ok(())
}

pub fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    if lower.ends_with("_env") || lower.ends_with(".env") || lower.ends_with("-env") {
        return false;
    }

    lower.contains("password")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("account_key")
        || lower.contains("account-key")
        || lower.contains("accountkey")
}

pub fn secret_paths(value: &Value) -> Vec<String> {
    fn walk(prefix: String, value: &Value, output: &mut Vec<String>) {
        if let Value::Object(map) = value {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.to_owned()
                } else {
                    format!("{prefix}.{key}")
                };
                if super::config::is_secret_key(&path) && !child.is_null() {
                    output.push(path.clone());
                }
                walk(path, child, output);
            }
        }
    }

    let mut output = Vec::new();
    walk(String::new(), value, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_nested_values_with_overlay_priority() {
        let mut base = json!({
            "provider": "firebase",
            "firebase": {
                "project": "global",
                "site": "main"
            }
        });
        let overlay = json!({
            "firebase": {
                "project": "local"
            }
        });

        merge_values(&mut base, overlay);

        assert_eq!(get_key(&base, "provider"), Some(&json!("firebase")));
        assert_eq!(get_key(&base, "firebase.project"), Some(&json!("local")));
        assert_eq!(get_key(&base, "firebase.site"), Some(&json!("main")));
    }

    #[test]
    fn set_key_creates_nested_objects() {
        let mut value = json!({});
        set_key(&mut value, "azure_blob.account", json!("account-name")).unwrap();
        assert_eq!(
            get_key(&value, "azure_blob.account"),
            Some(&json!("account-name"))
        );
    }

    #[test]
    fn secret_values_are_rejected_but_env_names_are_allowed() {
        let mut value = json!({});

        assert!(set_key(&mut value, "ftp.password", json!("plain")).is_err());
        assert!(set_key(&mut value, "ftp.password_env", json!("NOW_FTP_PASSWORD")).is_ok());
        assert!(
            set_key(
                &mut value,
                "azure_swa.deployment_token_env",
                json!("TOKEN_ENV")
            )
            .is_ok()
        );
    }

    #[test]
    fn parses_provider_names_from_config() {
        let config = parse_config(json!({ "provider": "firebase-hosting" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::Firebase));

        let config = parse_config(json!({ "provider": "Firebase Hosting" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::Firebase));

        let config = parse_config(json!({ "provider": "firebase" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::Firebase));

        let config = parse_config(json!({ "provider": "azure-storage-blob" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::AzureBlob));

        let config = parse_config(json!({ "provider": "Azure Storage Blob" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::AzureBlob));

        let config = parse_config(json!({ "provider": "azure-blob" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::AzureBlob));

        let config = parse_config(json!({ "provider": "azure-static-web-app" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::AzureSwa));

        let config = parse_config(json!({ "provider": "Azure Static Web App" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::AzureSwa));

        let config = parse_config(json!({ "provider": "azure-swa" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::AzureSwa));

        let config = parse_config(json!({ "provider": "any-website-ftp" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::Ftp));

        let config = parse_config(json!({ "provider": "Any Website (FTP)" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::Ftp));

        let config = parse_config(json!({ "provider": "ftp" })).unwrap();
        assert_eq!(config.provider, Some(ProviderKind::Ftp));
    }
}
