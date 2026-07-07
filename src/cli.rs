use crate::config::{
    ProviderKind, default_config, get_key, global_config_path, local_config_path,
    merged_config_value, parse_config, parse_config_value, read_json_file, secret_paths, set_key,
    write_json_file,
};
use crate::deploy::{DeployRequest, execute_deploy};
use crate::provider::{build_provider_command, program_available, provider_install_hint};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde_json::Value;
use std::ffi::OsString;
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
}

#[derive(Debug, Subcommand)]
enum Command {
    Deploy(DeployArgs),
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Args)]
struct DeployArgs {
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    #[arg(long, value_enum)]
    provider: Option<ProviderKind>,

    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Init(ScopeArgs),
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
        }),
        Some(Command::Config { command }) => execute_config(command, cwd),
        None => execute_deploy(DeployRequest {
            cwd,
            path_was_explicit: cli.path.is_some(),
            path: cli.path,
            provider: None,
            dry_run: false,
            json: false,
        }),
    }
}

fn execute_config(command: ConfigCommand, cwd: PathBuf) -> Result<()> {
    match command {
        ConfigCommand::Init(scope) => init_config(scope.write_scope(), &cwd),
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
    let path = config_path(scope, cwd)?;
    if path.exists() {
        bail!("config already exists: {}", path.display());
    }

    write_json_file(&path, &default_config())?;
    println!("Created {}", path.display());
    Ok(())
}

fn set_config(
    scope: ConfigWriteScope,
    cwd: &std::path::Path,
    key: &str,
    raw_value: &str,
) -> Result<()> {
    let path = config_path(scope, cwd)?;
    let mut value = read_json_file(&path)?;
    set_key(&mut value, key, parse_config_value(raw_value))?;
    write_json_file(&path, &value)?;
    println!("Updated {}", path.display());
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
            match build_provider_command(provider, &config, cwd) {
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
