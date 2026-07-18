use crate::azure_blob::{display_upload_command, upload_directory};
use crate::config::{
    NowConfig, ProviderKind, global_config_path, local_config_path, merged_config_value,
    parse_config, read_json_file, set_key, write_json_file,
};
use crate::env_file::{EnvFile, read_local_env};
use crate::fs_rules::is_excluded_path;
use crate::onboarding::run_first_run_setup;
use crate::provider::{build_provider_command, program_available, provider_install_hint};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DeployRequest {
    pub cwd: PathBuf,
    pub path: Option<PathBuf>,
    pub source: Option<PathBuf>,
    pub path_was_explicit: bool,
    pub provider: Option<ProviderKind>,
    pub prefix: Option<String>,
    pub remote_dir: Option<String>,
    pub dry_run: bool,
    pub json: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceMode {
    ExplicitPath,
    ConfiguredSource,
    AutoDetected,
    CurrentDirectoryWithExcludes,
    PublicDirectoryCreated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSelection {
    pub project_root: PathBuf,
    pub source: PathBuf,
    pub mode: SourceMode,
    pub excludes_enabled: bool,
}

pub struct PreparedSource {
    pub path: PathBuf,
    pub staging: Option<TempDir>,
}

pub fn execute_deploy(request: DeployRequest) -> Result<()> {
    verbose_log(
        request.verbose,
        format_args!("Project root: {}", request.cwd.display()),
    );
    verbose_log(
        request.verbose,
        format_args!(
            "Local config: {}",
            local_config_path(&request.cwd).display()
        ),
    );
    if request.verbose {
        verbose_log(
            true,
            format_args!("Global config: {}", global_config_path()?.display()),
        );
    }

    let mut merged_value = merged_config_value(&request.cwd, request.provider)?;
    let mut config = parse_config(merged_value)?;
    let source_override = request.source.as_deref();
    let mut selection = select_source(
        &request.cwd,
        request.path.as_deref().or(source_override),
        request.path_was_explicit || source_override.is_some(),
        config.source.as_deref(),
    )?;
    let provider = match config.provider {
        Some(provider) => provider,
        None if should_prompt_first_run(&request) => {
            let initial_source = if request.path_was_explicit {
                Some(config_source_for_path(&request.cwd, &selection.source)?)
            } else {
                None
            };
            let stdin = io::stdin();
            let mut input = io::BufReader::new(stdin.lock());
            let stdout = io::stdout();
            let mut output = stdout.lock();
            run_first_run_setup(
                &request.cwd,
                initial_source.as_deref(),
                &mut input,
                &mut output,
            )?;

            merged_value = merged_config_value(&request.cwd, request.provider)?;
            config = parse_config(merged_value)?;
            config
                .provider
                .context("provider is not configured after first-time setup")?
        }
        None => {
            bail!("provider is not configured; use --provider or set provider in .now.json");
        }
    };
    if let Some(prefix) = request.prefix {
        config.azure_blob.prefix = Some(prefix);
    }
    if let Some(remote_dir) = request.remote_dir {
        config.ftp.remote_dir = Some(remote_dir);
    }
    verbose_log(request.verbose, format_args!("Provider: {provider}"));

    let local_env = if matches!(provider, ProviderKind::AzureBlob | ProviderKind::AzureSwa) {
        Some(read_local_env(&request.cwd)?)
    } else {
        None
    };
    verbose_log(
        request.verbose,
        format_args!(
            "Source selection: mode={:?}, path={}, excludes={}",
            selection.mode,
            selection.source.display(),
            selection.excludes_enabled
        ),
    );
    let move_publishable_files =
        if selection.mode == SourceMode::CurrentDirectoryWithExcludes && !request.dry_run {
            match config.move_publishable_files_to_public {
                Some(choice) => Some(choice),
                None if !request.json && io::stdin().is_terminal() => {
                    Some(prompt_create_public_dir(&request.cwd)?)
                }
                None => None,
            }
        } else {
            None
        };

    if move_publishable_files == Some(true) {
        move_publishable_files_to_public(&request.cwd)?;
        selection = SourceSelection {
            project_root: request.cwd.clone(),
            source: request.cwd.join("public"),
            mode: SourceMode::PublicDirectoryCreated,
            excludes_enabled: false,
        };
    }

    let prepared = prepare_source(&selection)?;
    verbose_log(
        request.verbose,
        format_args!("Prepared source: {}", prepared.path.display()),
    );
    let default_url = choose_default_url(&config, provider, local_env.as_ref(), &prepared.path);

    if provider == ProviderKind::AzureBlob {
        let command = display_upload_command(&config, local_env.as_ref(), &prepared.path)?;
        verbose_log(
            request.verbose,
            format_args!("Built-in deployment: {command}"),
        );
        if request.dry_run {
            print_deploy_summary(
                &selection,
                &prepared.path,
                provider,
                &command,
                default_url.as_deref(),
                true,
                request.json,
            )?;
            return Ok(());
        }

        let upload_summary =
            upload_directory(&config, local_env.as_ref(), &prepared.path, request.verbose)?;
        print_deploy_summary(
            &selection,
            &prepared.path,
            provider,
            &command,
            default_url.as_deref(),
            false,
            request.json,
        )?;
        if !request.json {
            println!(
                "Uploaded: {} files, {} bytes",
                upload_summary.files, upload_summary.bytes
            );
        }
        return Ok(());
    }

    let command = build_provider_command(provider, &config, &prepared.path, request.verbose)?;
    verbose_log(
        request.verbose,
        format_args!("External command: {}", command.execution_line()),
    );
    verbose_log(
        request.verbose,
        format_args!(
            "Required environment variables: {}",
            command.required_env.join(", ")
        ),
    );
    if request.verbose && !command.env_mappings.is_empty() {
        let mappings = command
            .env_mappings
            .iter()
            .map(|(target, source)| format!("{target} <- {source}"))
            .collect::<Vec<_>>()
            .join(", ");
        verbose_log(true, format_args!("Environment mappings: {mappings}"));
    }

    if request.dry_run {
        print_deploy_summary(
            &selection,
            &prepared.path,
            provider,
            &command.display_line(),
            default_url.as_deref(),
            true,
            request.json,
        )?;
        return Ok(());
    }

    if !program_available(&command.required_cli) {
        bail!(
            "Provider CLI not found for {}: {}\n{}",
            provider,
            command.required_cli,
            provider_install_hint(provider)
        );
    }
    command.validate_environment(local_env.as_ref())?;

    run_provider_command(&command, &request.cwd, local_env.as_ref(), request.verbose)?;

    print_deploy_summary(
        &selection,
        &prepared.path,
        provider,
        &command.display_line(),
        default_url.as_deref(),
        false,
        request.json,
    )?;
    Ok(())
}

fn run_provider_command(
    command: &crate::provider::ProviderCommand,
    request_cwd: &Path,
    env_file: Option<&crate::env_file::EnvFile>,
    verbose: bool,
) -> Result<()> {
    let mut process = Command::new(&command.program);
    process.args(&command.args);
    if let Some(cwd) = &command.cwd {
        process.current_dir(cwd);
    } else {
        process.current_dir(request_cwd);
    }
    command.apply_environment(&mut process, env_file)?;
    let working_directory = command.cwd.as_deref().unwrap_or(request_cwd);
    verbose_log(
        verbose,
        format_args!(
            "External working directory: {}",
            working_directory.display()
        ),
    );

    let output = process
        .output()
        .with_context(|| format!("failed to run provider CLI: {}", command.program))?;

    if verbose {
        let mut stderr = io::stderr().lock();
        if !output.stdout.is_empty() {
            writeln!(stderr, "[verbose] External stdout:")?;
            stderr.write_all(&output.stdout)?;
            if !output.stdout.ends_with(b"\n") {
                writeln!(stderr)?;
            }
        }
        if !output.stderr.is_empty() {
            writeln!(stderr, "[verbose] External stderr:")?;
            stderr.write_all(&output.stderr)?;
            if !output.stderr.ends_with(b"\n") {
                writeln!(stderr)?;
            }
        }
    } else {
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
    }

    verbose_log(
        verbose,
        format_args!("External exit status: {}", output.status),
    );

    if !output.status.success() {
        bail!("provider command failed with status {}", output.status);
    }

    Ok(())
}

fn verbose_log(enabled: bool, message: std::fmt::Arguments<'_>) {
    if enabled {
        eprintln!("[verbose] {message}");
    }
}

fn should_prompt_first_run(request: &DeployRequest) -> bool {
    request.provider.is_none() && !request.json && io::stdin().is_terminal()
}

pub fn select_source(
    project_root: &Path,
    path: Option<&Path>,
    path_was_explicit: bool,
    config_source: Option<&str>,
) -> Result<SourceSelection> {
    if path_was_explicit {
        let path = path.context("explicit path flag was set without a path")?;
        let source = resolve_path(project_root, path);
        ensure_directory(&source)?;
        let excludes_enabled = source == project_root || path == Path::new(".");
        return Ok(SourceSelection {
            project_root: project_root.to_path_buf(),
            source,
            mode: SourceMode::ExplicitPath,
            excludes_enabled,
        });
    }

    if let Some(config_source) = config_source.filter(|value| !value.trim().is_empty()) {
        let configured_path = Path::new(config_source);
        let source = resolve_path(project_root, configured_path);
        ensure_directory(&source)?;
        let excludes_enabled = source == project_root || configured_path == Path::new(".");
        return Ok(SourceSelection {
            project_root: project_root.to_path_buf(),
            source,
            mode: SourceMode::ConfiguredSource,
            excludes_enabled,
        });
    }

    for candidate in ["dist", "build", "public"] {
        let source = project_root.join(candidate);
        if source.is_dir() {
            return Ok(SourceSelection {
                project_root: project_root.to_path_buf(),
                source,
                mode: SourceMode::AutoDetected,
                excludes_enabled: false,
            });
        }
    }

    ensure_directory(project_root)?;
    Ok(SourceSelection {
        project_root: project_root.to_path_buf(),
        source: project_root.to_path_buf(),
        mode: SourceMode::CurrentDirectoryWithExcludes,
        excludes_enabled: true,
    })
}

pub fn prepare_source(selection: &SourceSelection) -> Result<PreparedSource> {
    if !selection.excludes_enabled {
        return Ok(PreparedSource {
            path: selection.source.clone(),
            staging: None,
        });
    }

    let temp_dir = TempDir::new().context("failed to create staging directory")?;
    copy_filtered(&selection.source, temp_dir.path())?;

    Ok(PreparedSource {
        path: temp_dir.path().to_path_buf(),
        staging: Some(temp_dir),
    })
}

pub fn choose_default_url(
    config: &NowConfig,
    provider: ProviderKind,
    env_file: Option<&EnvFile>,
    source: &Path,
) -> Option<String> {
    if let Some(default_url) = config
        .default_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(default_url.to_owned());
    }

    let base_url = config.provider_base_url(provider);
    for file_name in ["index.html", "index.htm"] {
        if source.join(file_name).is_file() {
            return Some(default_url_for_file(
                config,
                provider,
                env_file,
                base_url,
                Path::new(file_name),
            ));
        }
    }

    if let Ok(entries) = fs::read_dir(source) {
        let html_files = entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if !path.is_file() {
                    return None;
                }
                let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
                if matches!(extension.as_str(), "html" | "htm") {
                    path.file_name()
                        .map(|file_name| file_name.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if html_files.len() == 1 {
            return Some(default_url_for_file(
                config,
                provider,
                env_file,
                base_url,
                Path::new(&html_files[0]),
            ));
        }
    }

    base_url
        .map(str::to_owned)
        .or_else(|| inferred_provider_base_url(config, provider, env_file))
}

fn default_url_for_file(
    config: &NowConfig,
    provider: ProviderKind,
    env_file: Option<&EnvFile>,
    base_url: Option<&str>,
    relative_path: &Path,
) -> String {
    if let Some(base_url) = base_url {
        return join_url(Some(base_url), &relative_path.to_string_lossy());
    }

    match provider {
        ProviderKind::AzureBlob => {
            crate::azure_blob::public_blob_url_for_relative_path(config, env_file, relative_path)
                .unwrap_or_else(|| relative_path.to_string_lossy().into_owned())
        }
        _ => relative_path.to_string_lossy().into_owned(),
    }
}

fn inferred_provider_base_url(
    config: &NowConfig,
    provider: ProviderKind,
    env_file: Option<&EnvFile>,
) -> Option<String> {
    match provider {
        ProviderKind::AzureBlob => crate::azure_blob::public_base_url(config, env_file),
        _ => None,
    }
}

fn join_url(base_url: Option<&str>, file_name: &str) -> String {
    match base_url {
        Some(base_url) => format!("{}/{}", base_url.trim_end_matches('/'), file_name),
        None => file_name.to_owned(),
    }
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn config_source_for_path(project_root: &Path, source: &Path) -> Result<String> {
    let canonical_root = project_root.canonicalize().with_context(|| {
        format!(
            "failed to resolve project root for saved source: {}",
            project_root.display()
        )
    })?;
    let canonical_source = source.canonicalize().with_context(|| {
        format!(
            "failed to resolve source path for saved source: {}",
            source.display()
        )
    })?;

    let stored_path = match canonical_source.strip_prefix(&canonical_root) {
        Ok(relative) if relative.as_os_str().is_empty() => return Ok(".".to_owned()),
        Ok(relative) => relative,
        Err(_) => canonical_source.as_path(),
    };

    stored_path.to_str().map(str::to_owned).with_context(|| {
        format!(
            "source path cannot be saved as UTF-8 in .now.json: {}",
            stored_path.display()
        )
    })
}

fn ensure_directory(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("source path is not a directory: {}", path.display());
    }
    Ok(())
}

fn copy_filtered(source: &Path, destination: &Path) -> Result<()> {
    for entry in WalkDir::new(source)
        .into_iter()
        .filter_entry(|entry| entry.path() == source || !is_excluded_path(source, entry.path()))
    {
        let entry = entry?;
        if entry.path() == source {
            continue;
        }
        if is_excluded_path(source, entry.path()) {
            continue;
        }

        let relative = entry.path().strip_prefix(source)?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn prompt_create_public_dir(root: &Path) -> Result<bool> {
    let stdin = io::stdin();
    let mut input = io::BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut output = stdout.lock();
    prompt_and_remember_move_publishable_files(root, &mut input, &mut output)
}

fn prompt_and_remember_move_publishable_files<R: BufRead, W: Write>(
    root: &Path,
    input: &mut R,
    output: &mut W,
) -> Result<bool> {
    write!(
        output,
        "No dist/, build/, or public/ directory was found. Move publishable files into public/? [y/N] "
    )?;
    output.flush()?;

    let mut answer = String::new();
    input.read_line(&mut answer)?;
    let choice = matches!(answer.trim(), "y" | "Y" | "yes" | "YES");
    remember_move_publishable_files_choice(root, choice)?;
    Ok(choice)
}

fn remember_move_publishable_files_choice(root: &Path, choice: bool) -> Result<()> {
    let path = local_config_path(root);
    let mut config = read_json_file(&path)?;
    set_key(
        &mut config,
        "move_publishable_files_to_public",
        json!(choice),
    )?;
    write_json_file(&path, &config)
}

fn move_publishable_files_to_public(root: &Path) -> Result<()> {
    let public = root.join("public");
    fs::create_dir_all(&public)?;

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name == "public" || is_excluded_path(root, &path) {
            continue;
        }

        fs::rename(&path, public.join(file_name))?;
    }

    Ok(())
}

fn print_deploy_summary(
    selection: &SourceSelection,
    prepared_path: &Path,
    provider: ProviderKind,
    command: &str,
    default_url: Option<&str>,
    dry_run: bool,
    as_json: bool,
) -> Result<()> {
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "provider": provider.as_str(),
                "source": prepared_path,
                "source_mode": selection.mode,
                "excluded": selection.excludes_enabled,
                "command": command,
                "default_url": default_url,
                "dry_run": dry_run
            }))?
        );
        return Ok(());
    }

    println!("Provider: {provider}");
    println!("Source: {}", prepared_path.display());
    println!("Source mode: {:?}", selection.mode);
    if selection.excludes_enabled {
        println!("Excluded: .now.json, .env, .env.*, .git/, node_modules/, target/, temp files");
    }
    println!("Command: {command}");
    if dry_run {
        println!("Dry run: provider command was not executed");
    }
    match default_url {
        Some(url) => println!("Default URL: {url}"),
        None => println!("Default URL: not resolved; set default_url or base_url"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NowConfig;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    #[test]
    fn selects_default_source_by_priority() {
        let temp = TempDir::new().unwrap();
        temp.child("build").create_dir_all().unwrap();
        temp.child("dist").create_dir_all().unwrap();

        let selection = select_source(temp.path(), None, false, None).unwrap();

        assert_eq!(selection.source, temp.path().join("dist"));
        assert_eq!(selection.mode, SourceMode::AutoDetected);
    }

    #[test]
    fn explicit_path_bypasses_auto_detection() {
        let temp = TempDir::new().unwrap();
        temp.child("dist").create_dir_all().unwrap();
        temp.child("site").create_dir_all().unwrap();

        let selection = select_source(temp.path(), Some(Path::new("site")), true, None).unwrap();

        assert_eq!(selection.source, temp.path().join("site"));
        assert_eq!(selection.mode, SourceMode::ExplicitPath);
    }

    #[test]
    fn explicit_source_inside_project_is_saved_as_relative_path() {
        let temp = TempDir::new().unwrap();
        temp.child("reports/litellm-spend/2026-07-18")
            .create_dir_all()
            .unwrap();
        let source = temp.path().join("reports/litellm-spend/2026-07-18");

        let config_source = config_source_for_path(temp.path(), &source).unwrap();

        assert_eq!(config_source, "reports/litellm-spend/2026-07-18");
    }

    #[test]
    fn explicit_project_root_is_saved_as_dot() {
        let temp = TempDir::new().unwrap();

        let config_source = config_source_for_path(temp.path(), temp.path()).unwrap();

        assert_eq!(config_source, ".");
    }

    #[test]
    fn explicit_project_root_keeps_runtime_excludes_enabled() {
        let temp = TempDir::new().unwrap();

        let selection = select_source(temp.path(), Some(Path::new(".")), true, None).unwrap();

        assert_eq!(selection.mode, SourceMode::ExplicitPath);
        assert!(selection.excludes_enabled);
    }

    #[test]
    fn configured_project_root_keeps_runtime_excludes_enabled() {
        let temp = TempDir::new().unwrap();

        let selection = select_source(temp.path(), None, false, Some(".")).unwrap();

        assert_eq!(selection.mode, SourceMode::ConfiguredSource);
        assert!(selection.excludes_enabled);
    }

    #[test]
    fn explicit_source_outside_project_is_saved_as_absolute_path() {
        let project = TempDir::new().unwrap();
        let source = TempDir::new().unwrap();

        let config_source = config_source_for_path(project.path(), source.path()).unwrap();

        assert_eq!(
            config_source,
            source.path().canonicalize().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn falls_back_to_current_directory_with_excludes() {
        let temp = TempDir::new().unwrap();

        let selection = select_source(temp.path(), None, false, None).unwrap();

        assert_eq!(selection.source, temp.path());
        assert!(selection.excludes_enabled);
    }

    #[test]
    fn remembers_publishable_files_choice_in_local_config() {
        let temp = TempDir::new().unwrap();
        temp.child(".now.json")
            .write_str(r#"{"provider":"firebase-hosting"}"#)
            .unwrap();
        let mut input = std::io::Cursor::new(b"n\n");
        let mut output = Vec::new();

        let choice =
            prompt_and_remember_move_publishable_files(temp.path(), &mut input, &mut output)
                .unwrap();

        assert!(!choice);
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("Move publishable files")
        );
        let config = crate::config::read_json_file(&temp.path().join(".now.json")).unwrap();
        assert_eq!(
            crate::config::get_key(&config, "provider"),
            Some(&json!("firebase-hosting"))
        );
        assert_eq!(
            crate::config::get_key(&config, "move_publishable_files_to_public"),
            Some(&json!(false))
        );
    }

    #[test]
    fn staged_source_excludes_runtime_files() {
        let temp = TempDir::new().unwrap();
        temp.child("index.html").write_str("ok").unwrap();
        temp.child(".now.json").write_str("{}").unwrap();
        temp.child(".env").write_str("SECRET=value").unwrap();
        temp.child("node_modules/pkg/index.js")
            .write_str("skip")
            .unwrap();

        let selection = select_source(temp.path(), None, false, None).unwrap();
        let prepared = prepare_source(&selection).unwrap();

        assert!(prepared.path.join("index.html").is_file());
        assert!(!prepared.path.join(".now.json").exists());
        assert!(!prepared.path.join(".env").exists());
        assert!(!prepared.path.join("node_modules").exists());
    }

    #[test]
    fn moving_publishable_files_keeps_env_file_outside_public() {
        let temp = TempDir::new().unwrap();
        temp.child("index.html").write_str("ok").unwrap();
        temp.child(".env").write_str("SECRET=value").unwrap();

        move_publishable_files_to_public(temp.path()).unwrap();

        assert!(temp.path().join("public/index.html").is_file());
        assert!(temp.path().join(".env").is_file());
        assert!(!temp.path().join("public/.env").exists());
    }

    #[test]
    fn default_url_prefers_config_then_index_then_single_html() {
        let temp = TempDir::new().unwrap();
        temp.child("index.html").write_str("ok").unwrap();

        let config = NowConfig {
            base_url: Some("https://example.com/site/".to_owned()),
            ..NowConfig::default()
        };
        assert_eq!(
            choose_default_url(&config, ProviderKind::Firebase, None, temp.path()).as_deref(),
            Some("https://example.com/site/index.html")
        );

        let config = NowConfig {
            default_url: Some("https://example.com/custom".to_owned()),
            ..config
        };
        assert_eq!(
            choose_default_url(&config, ProviderKind::Firebase, None, temp.path()).as_deref(),
            Some("https://example.com/custom")
        );
    }

    #[test]
    fn default_url_infers_azure_blob_url_from_sas_when_base_url_is_missing() {
        let temp = TempDir::new().unwrap();
        temp.child("index.html").write_str("ok").unwrap();

        let env_file = EnvFile::from_pairs(&[(
            "NOW_AZURE_BLOB_SAS_URL",
            "https://infinitybin.blob.core.windows.net/now/now?sv=1&sig=secret",
        )]);
        let config = NowConfig {
            azure_blob: crate::config::AzureBlobConfig {
                sas_url_env: Some("NOW_AZURE_BLOB_SAS_URL".to_owned()),
                ..Default::default()
            },
            ..NowConfig::default()
        };

        assert_eq!(
            choose_default_url(
                &config,
                ProviderKind::AzureBlob,
                Some(&env_file),
                temp.path()
            )
            .as_deref(),
            Some("https://infinitybin.blob.core.windows.net/now/now/index.html")
        );
    }
}
