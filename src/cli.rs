use crate::azure_blob::display_upload_command;
use crate::config::{
    DEFAULT_AZURE_BLOB_SAS_URL_ENV, ProviderKind, get_key, global_config_path, local_config_path,
    merged_config_value, parse_config, parse_config_value, read_json_file, remove_key,
    secret_paths, set_key, write_json_file,
};
use crate::deploy::{DeployRequest, execute_deploy};
use crate::env_file::{local_env_path, read_local_env, write_env_value};
use crate::onboarding::run_init_setup;
use crate::provider::{build_provider_command, program_available, provider_install_hint};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde_json::Value;
use std::ffi::OsString;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "now",
    version,
    about = "Deploy static sites with provider CLIs",
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    #[arg(
        long,
        help = "Show detailed diagnostics and enable provider debug logs"
    )]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Deploy(DeployArgs),
    Init(ScopeArgs),
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Args)]
struct DeployArgs {
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    #[arg(long, value_parser = parse_provider)]
    provider: Option<ProviderKind>,

    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    json: bool,

    #[arg(
        long,
        help = "Show detailed diagnostics and enable provider debug logs"
    )]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Set(ConfigSetArgs),
    Get(ConfigGetArgs),
    Doctor,
}

#[derive(Debug, Args, Clone, Copy)]
struct ScopeArgs {
    #[arg(long, conflicts_with = "local")]
    global: bool,

    #[arg(long, conflicts_with = "global")]
    local: bool,
}

#[derive(Debug, Args)]
struct ConfigSetArgs {
    key: String,
    value: String,

    #[command(flatten)]
    scope: ScopeArgs,
}

#[derive(Debug, Args)]
struct ConfigGetArgs {
    key: Option<String>,

    #[command(flatten)]
    scope: ScopeArgs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigReadScope {
    Merged,
    Local,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigWriteScope {
    Local,
    Global,
}

impl ScopeArgs {
    fn read_scope(self) -> ConfigReadScope {
        if self.global {
            ConfigReadScope::Global
        } else if self.local {
            ConfigReadScope::Local
        } else {
            ConfigReadScope::Merged
        }
    }

    fn write_scope(self) -> ConfigWriteScope {
        if self.global {
            ConfigWriteScope::Global
        } else {
            ConfigWriteScope::Local
        }
    }
}

pub fn run_from<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    match cli.command {
        Some(Command::Deploy(args)) => execute_deploy(DeployRequest {
            cwd,
            path_was_explicit: args.path.is_some(),
            path: args.path,
            provider: args.provider,
            dry_run: args.dry_run,
            json: args.json,
            verbose: args.verbose,
        }),
        Some(Command::Init(scope)) => init_config(scope.write_scope(), &cwd),
        Some(Command::Config { command }) => execute_config(command, cwd),
        None => execute_deploy(DeployRequest {
            cwd,
            path_was_explicit: cli.path.is_some(),
            path: cli.path,
            provider: None,
            dry_run: false,
            json: false,
            verbose: cli.verbose,
        }),
    }
}

fn execute_config(command: ConfigCommand, cwd: PathBuf) -> Result<()> {
    match command {
        ConfigCommand::Set(args) => {
            set_config(args.scope.write_scope(), &cwd, &args.key, &args.value)
        }
        ConfigCommand::Get(args) => get_config(args.scope.read_scope(), &cwd, args.key.as_deref()),
        ConfigCommand::Doctor => doctor_config(&cwd),
    }
}

fn config_path(scope: ConfigWriteScope, cwd: &std::path::Path) -> Result<PathBuf> {
    Ok(match scope {
        ConfigWriteScope::Local => local_config_path(cwd),
        ConfigWriteScope::Global => global_config_path()?,
    })
}

fn init_config(scope: ConfigWriteScope, cwd: &std::path::Path) -> Result<()> {
    let stdin = io::stdin();
    let mut input = io::BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut output = stdout.lock();
    init_config_with_io(scope, cwd, &mut input, &mut output)
}

fn init_config_with_io<R: BufRead, W: Write>(
    scope: ConfigWriteScope,
    cwd: &std::path::Path,
    input: &mut R,
    output: &mut W,
) -> Result<()> {
    let path = config_path(scope, cwd)?;
    if path.exists() {
        writeln!(output, "Config already exists: {}", path.display())?;
        write!(output, "Reconfigure and overwrite existing config? [y/N] ")?;
        output.flush()?;

        let mut answer = String::new();
        input.read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
            writeln!(output, "Kept {}", path.display())?;
            return Ok(());
        }
    }

    run_init_setup(cwd, &path, input, output)?;
    Ok(())
}

fn set_config(
    scope: ConfigWriteScope,
    cwd: &std::path::Path,
    key: &str,
    raw_value: &str,
) -> Result<()> {
    if key == "azure_blob.sas_url" {
        return set_azure_blob_sas_url(scope, cwd, raw_value);
    }

    let path = config_path(scope, cwd)?;
    let mut value = read_json_file(&path)?;
    set_key(&mut value, key, parse_config_value(raw_value))?;
    if key == "azure_blob.sas_url_env" {
        remove_key(&mut value, "azure_blob.sas_url");
    }
    write_json_file(&path, &value)?;
    println!("Updated {}", path.display());
    Ok(())
}

fn set_azure_blob_sas_url(
    scope: ConfigWriteScope,
    cwd: &std::path::Path,
    sas_url: &str,
) -> Result<()> {
    if scope == ConfigWriteScope::Global {
        bail!(
            "azure_blob.sas_url is secret-like; set azure_blob.sas_url_env globally and provide the SAS URL through an environment variable"
        );
    }

    let config_path = config_path(scope, cwd)?;
    let mut value = read_json_file(&config_path)?;
    let env_name = get_key(&value, "azure_blob.sas_url_env")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_AZURE_BLOB_SAS_URL_ENV)
        .to_owned();

    set_key(
        &mut value,
        "azure_blob.sas_url_env",
        Value::String(env_name.clone()),
    )?;
    remove_key(&mut value, "azure_blob.sas_url");
    write_json_file(&config_path, &value)?;

    let env_path = local_env_path(cwd);
    write_env_value(&env_path, &env_name, sas_url)?;
    println!("Updated {}", config_path.display());
    println!("Updated {}", env_path.display());
    Ok(())
}

fn get_config(scope: ConfigReadScope, cwd: &std::path::Path, key: Option<&str>) -> Result<()> {
    let value = match scope {
        ConfigReadScope::Merged => merged_config_value(cwd, None)?,
        ConfigReadScope::Local => read_json_file(&local_config_path(cwd))?,
        ConfigReadScope::Global => read_json_file(&global_config_path()?)?,
    };

    let output = match key {
        Some(key) => get_key(&value, key)
            .with_context(|| format!("config key not found: {key}"))?
            .clone(),
        None => value,
    };

    print_json_value(&output)?;
    Ok(())
}

fn doctor_config(cwd: &std::path::Path) -> Result<()> {
    let global_path = global_config_path()?;
    let local_path = local_config_path(cwd);
    println!("Global config: {}", global_path.display());
    println!("Local config: {}", local_path.display());

    let merged_value = merged_config_value(cwd, None)?;
    let secret_paths = secret_paths(&merged_value);
    if secret_paths.is_empty() {
        println!("Secrets: none found in config files");
    } else {
        println!("Secrets: remove these keys: {}", secret_paths.join(", "));
    }

    let config = parse_config(merged_value)?;
    match config.provider {
        Some(provider) => {
            println!("Provider: {provider}");
            if provider == ProviderKind::AzureBlob {
                let local_env = read_local_env(cwd)?;
                match display_upload_command(&config, Some(&local_env), cwd) {
                    Ok(command) => {
                        println!("Command: {command}");
                        println!("Provider CLI: not required");
                    }
                    Err(error) => println!("Provider config: {error:#}"),
                }
                return Ok(());
            }

            match build_provider_command(provider, &config, cwd, false) {
                Ok(command) => {
                    println!("Command: {}", command.display_line());
                    if program_available(&command.required_cli) {
                        println!("Provider CLI: found {}", command.required_cli);
                    } else {
                        println!("Provider CLI: missing {}", command.required_cli);
                        println!("{}", provider_install_hint(provider));
                    }
                }
                Err(error) => println!("Provider config: {error:#}"),
            }
        }
        None => println!("Provider: not configured"),
    }

    Ok(())
}

fn print_json_value(value: &Value) -> Result<()> {
    match value {
        Value::String(value) => println!("{value}"),
        _ => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}

fn parse_provider(value: &str) -> std::result::Result<ProviderKind, String> {
    ProviderKind::parse(value).ok_or_else(|| {
        format!(
            "unsupported provider {value}; expected firebase-hosting, azure-storage-blob, azure-static-web-app, or any-website-ftp"
        )
    })
}
