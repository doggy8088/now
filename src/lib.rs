pub mod azure_blob;
pub mod cli;
pub mod config;
pub mod deploy;
pub mod env_file;
pub mod fs_rules;
pub mod onboarding;
pub mod provider;

pub fn run() -> anyhow::Result<()> {
    cli::run_from(std::env::args_os())
}
