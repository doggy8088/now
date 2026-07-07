use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvFile {
    values: BTreeMap<String, String>,
}

impl EnvFile {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    #[cfg(test)]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        Self {
            values: pairs
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        }
    }
}

pub fn local_env_path(root: &Path) -> PathBuf {
    root.join(".env")
}

pub fn read_local_env(root: &Path) -> Result<EnvFile> {
    read_env_file(&local_env_path(root))
}

pub fn read_env_file(path: &Path) -> Result<EnvFile> {
    if !path.exists() {
        return Ok(EnvFile::default());
    }

    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut values = BTreeMap::new();
    for line in text.lines() {
        if let Some((key, value)) = parse_env_line(line)? {
            values.insert(key, value);
        }
    }

    Ok(EnvFile { values })
}

pub fn env_value(name: &str, env_file: Option<&EnvFile>) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env_file
                .and_then(|file| file.get(name))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
}

pub fn write_env_value(path: &Path, key: &str, value: &str) -> Result<()> {
    validate_env_name(key)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let replacement = format!("{key}={}\n", quote_env_value(value));
    let mut replaced = false;
    let mut output = String::new();

    if path.exists() {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read existing {}", path.display()))?;
        for line in text.lines() {
            if env_line_key(line).as_deref() == Some(key) && !replaced {
                output.push_str(&replacement);
                replaced = true;
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }
        if !text.ends_with('\n') && !output.ends_with('\n') {
            output.push('\n');
        }
    }

    if !replaced {
        output.push_str(&replacement);
    }

    fs::write(path, output).with_context(|| format!("failed to write {}", path.display()))?;
    restrict_owner_permissions(path)?;
    Ok(())
}

pub fn validate_env_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        bail!("environment variable name cannot be empty");
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        bail!("invalid environment variable name: {name}");
    }
    if !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        bail!("invalid environment variable name: {name}");
    }
    Ok(())
}

fn parse_env_line(line: &str) -> Result<Option<(String, String)>> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let line = line.strip_prefix("export ").unwrap_or(line);
    let Some((key, raw_value)) = line.split_once('=') else {
        return Ok(None);
    };
    let key = key.trim();
    if validate_env_name(key).is_err() {
        return Ok(None);
    }
    Ok(Some((
        key.to_owned(),
        parse_env_value(raw_value.trim_start())?,
    )))
}

fn parse_env_value(raw: &str) -> Result<String> {
    if let Some(rest) = raw.strip_prefix('"') {
        return parse_double_quoted(rest);
    }
    if let Some(rest) = raw.strip_prefix('\'') {
        return parse_single_quoted(rest);
    }

    Ok(strip_unquoted_comment(raw).trim_end().to_owned())
}

fn parse_double_quoted(raw: &str) -> Result<String> {
    let mut value = String::new();
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Ok(value),
            '\\' => match chars.next() {
                Some('n') => value.push('\n'),
                Some('r') => value.push('\r'),
                Some('t') => value.push('\t'),
                Some('"') => value.push('"'),
                Some('\\') => value.push('\\'),
                Some(other) => {
                    value.push('\\');
                    value.push(other);
                }
                None => value.push('\\'),
            },
            _ => value.push(ch),
        }
    }
    bail!("unterminated double-quoted .env value")
}

fn parse_single_quoted(raw: &str) -> Result<String> {
    if let Some((value, _)) = raw.split_once('\'') {
        return Ok(value.to_owned());
    }
    bail!("unterminated single-quoted .env value")
}

fn strip_unquoted_comment(raw: &str) -> &str {
    let mut previous_was_space = false;
    for (index, ch) in raw.char_indices() {
        if ch == '#' && previous_was_space {
            return &raw[..index];
        }
        previous_was_space = ch.is_ascii_whitespace();
    }
    raw
}

fn env_line_key(line: &str) -> Option<String> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line);
    let (key, _) = line.split_once('=')?;
    let key = key.trim();
    validate_env_name(key).ok()?;
    Some(key.to_owned())
}

fn quote_env_value(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

#[cfg(unix)]
fn restrict_owner_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to set permissions on {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_owner_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    #[test]
    fn reads_plain_and_quoted_env_values() {
        let temp = TempDir::new().unwrap();
        let env = temp.child(".env");
        env.write_str(
            r#"
# ignored
PLAIN=https://example.com/path?sig=secret
QUOTED="https://example.com/$web?sv=1&sig=secret"
export SINGLE='hello world'
"#,
        )
        .unwrap();

        let file = read_env_file(env.path()).unwrap();

        assert_eq!(
            file.get("PLAIN"),
            Some("https://example.com/path?sig=secret")
        );
        assert_eq!(
            file.get("QUOTED"),
            Some("https://example.com/$web?sv=1&sig=secret")
        );
        assert_eq!(file.get("SINGLE"), Some("hello world"));
    }

    #[test]
    fn writes_or_replaces_env_value() {
        let temp = TempDir::new().unwrap();
        let env = temp.child(".env");
        env.write_str("KEEP=1\nNOW_AZURE_BLOB_SAS_URL=\"old\"\n")
            .unwrap();

        write_env_value(
            env.path(),
            "NOW_AZURE_BLOB_SAS_URL",
            "https://acct.blob.core.windows.net/$web?sv=1&sig=secret",
        )
        .unwrap();

        let text = fs::read_to_string(env.path()).unwrap();
        assert!(text.contains("KEEP=1\n"));
        assert!(text.contains(
            "NOW_AZURE_BLOB_SAS_URL=\"https://acct.blob.core.windows.net/$web?sv=1&sig=secret\""
        ));
        assert_eq!(
            read_env_file(env.path())
                .unwrap()
                .get("NOW_AZURE_BLOB_SAS_URL"),
            Some("https://acct.blob.core.windows.net/$web?sv=1&sig=secret")
        );
    }
}
