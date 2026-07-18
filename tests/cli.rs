use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use std::env;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn now_cmd(config_home: &TempDir) -> Command {
    let mut command = Command::cargo_bin("now").unwrap();
    command.env("NOW_CONFIG_HOME", config_home.path());
    command.env_remove("NOW_AZURE_BLOB_SAS_URL");
    command.env_remove("SWA_CLI_DEPLOYMENT_TOKEN");
    command
}

#[cfg(unix)]
fn write_fake_cli(bin_dir: &TempDir, name: &str, body: &str) {
    let script = bin_dir.child(name);
    script.write_str(body).unwrap();
    let mut permissions = std::fs::metadata(script.path()).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(script.path(), permissions).unwrap();
}

#[cfg(windows)]
fn write_fake_cli(bin_dir: &TempDir, name: &str, body: &str) {
    bin_dir
        .child(format!("{name}.cmd"))
        .write_str(body)
        .unwrap();
}

fn path_with_fake_bin(bin_dir: &TempDir) -> String {
    let old_path = env::var_os("PATH").unwrap_or_default();
    env::join_paths(
        std::iter::once(bin_dir.path().to_path_buf()).chain(env::split_paths(&old_path)),
    )
    .unwrap()
    .to_string_lossy()
    .into_owned()
}

#[test]
fn help_is_available() {
    let config_home = TempDir::new().unwrap();
    now_cmd(&config_home)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploy static sites"));
}

#[test]
fn verbose_dry_run_reports_diagnostics_and_full_external_command() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(r#"{"provider":"firebase-hosting"}"#)
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--verbose", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "firebase --debug deploy --only hosting",
        ))
        .stderr(predicate::str::contains("[verbose] Project root:"))
        .stderr(predicate::str::contains(
            "[verbose] External command: firebase --debug deploy --only hosting",
        ));
}

#[test]
fn verbose_flag_works_with_default_now_deploy_command() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    let bin_dir = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(r#"{"provider":"firebase-hosting"}"#)
        .unwrap();

    #[cfg(unix)]
    write_fake_cli(
        &bin_dir,
        "firebase",
        "#!/bin/sh\nprintf 'provider-debug-log'\n",
    );
    #[cfg(windows)]
    write_fake_cli(
        &bin_dir,
        "firebase",
        "@echo off\r\n<nul set /p =provider-debug-log\r\n",
    );

    now_cmd(&config_home)
        .current_dir(site.path())
        .env("PATH", path_with_fake_bin(&bin_dir))
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("provider-debug-log").not())
        .stderr(predicate::str::contains(
            "[verbose] External command: firebase --debug deploy --only hosting",
        ))
        .stderr(predicate::str::contains("provider-debug-log"));
}

#[test]
fn deploy_dry_run_uses_configured_provider() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "firebase-hosting",
  "base_url": "https://example.web.app"
}
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("firebase deploy --only hosting"))
        .stdout(predicate::str::contains(
            "Default URL: https://example.web.app/index.html",
        ));
}

#[test]
fn explicit_path_does_not_rewrite_existing_source() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("configured/index.html")
        .write_str("configured")
        .unwrap();
    site.child("one-off/index.html")
        .write_str("one-off")
        .unwrap();
    let original = r#"{
  "provider": "firebase-hosting",
  "source": "configured"
}
"#;
    site.child(".now.json").write_str(original).unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "one-off", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Source mode: ExplicitPath"));

    assert_eq!(
        std::fs::read_to_string(site.path().join(".now.json")).unwrap(),
        original
    );
}

#[test]
fn source_flag_overrides_config_without_rewriting_it() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("configured/index.html")
        .write_str("configured")
        .unwrap();
    site.child("one-off/index.html")
        .write_str("one-off")
        .unwrap();
    let original = r#"{
  "provider": "firebase-hosting",
  "source": "configured"
}
"#;
    site.child(".now.json").write_str(original).unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--source", "one-off", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            site.path().join("one-off").display().to_string(),
        ));

    assert_eq!(
        std::fs::read_to_string(site.path().join(".now.json")).unwrap(),
        original
    );
}

#[test]
fn invalid_explicit_path_fails_without_creating_local_config() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "missing", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("source path is not a directory"));

    assert!(!site.path().join(".now.json").exists());
}

#[test]
fn config_set_and_get_local_value() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["config", "set", "provider", "firebase-hosting"])
        .assert()
        .success();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["config", "get", "provider"])
        .assert()
        .success()
        .stdout(predicate::str::contains("firebase-hosting"));
}

#[test]
fn config_set_azure_blob_sas_url_writes_env_file_and_env_name() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args([
            "config",
            "set",
            "azure_blob.sas_url",
            "https://acct.blob.core.windows.net/$web?sv=1&sig=secret",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(".now.json"))
        .stdout(predicate::str::contains(".env"));

    let config: Value =
        serde_json::from_str(&std::fs::read_to_string(site.path().join(".now.json")).unwrap())
            .unwrap();
    assert_eq!(
        config
            .pointer("/azure_blob/sas_url_env")
            .and_then(Value::as_str),
        Some("NOW_AZURE_BLOB_SAS_URL")
    );
    assert!(config.pointer("/azure_blob/sas_url").is_none());

    let env_text = std::fs::read_to_string(site.path().join(".env")).unwrap();
    assert!(env_text.contains("NOW_AZURE_BLOB_SAS_URL="));
    assert!(env_text.contains("sig=secret"));
}

#[test]
fn init_runs_interactive_setup_without_deploying() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .arg("init")
        .write_stdin("1\nhttps://example.web.app\n\nmy-project\n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Choose a provider"))
        .stdout(predicate::str::contains("Configuration complete"))
        .stdout(predicate::str::contains("Continuing with deployment").not());

    let config: Value =
        serde_json::from_str(&std::fs::read_to_string(site.path().join(".now.json")).unwrap())
            .unwrap();
    assert_eq!(
        config.pointer("/provider").and_then(Value::as_str),
        Some("firebase-hosting")
    );
    assert_eq!(
        config.pointer("/firebase/project").and_then(Value::as_str),
        Some("my-project")
    );
}

#[test]
fn init_existing_config_prompts_and_keeps_it_when_declined() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    let original = r#"{
  "provider": "firebase-hosting"
}
"#;
    site.child(".now.json").write_str(original).unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Reconfigure and overwrite existing config? [y/N]",
        ))
        .stdout(predicate::str::contains("Kept"));

    assert_eq!(
        std::fs::read_to_string(site.path().join(".now.json")).unwrap(),
        original
    );
}

#[test]
fn init_existing_config_reconfigures_it_when_confirmed() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "firebase-hosting"
}
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .arg("init")
        .write_stdin("y\n\n\n\n\n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Reconfigure and overwrite existing config? [y/N]",
        ))
        .stdout(predicate::str::contains("Choose a provider"))
        .stdout(predicate::str::contains("Configuration complete"))
        .stdout(predicate::str::contains("Continuing with deployment").not());

    let config: Value =
        serde_json::from_str(&std::fs::read_to_string(site.path().join(".now.json")).unwrap())
            .unwrap();
    assert_eq!(
        config.pointer("/provider").and_then(Value::as_str),
        Some("firebase-hosting")
    );
}

#[test]
fn config_init_is_no_longer_supported() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["config", "init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'init'"));
}

#[test]
fn missing_provider_cli_returns_install_hint() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "firebase-hosting",
  "base_url": "https://infinitybin.blob.core.windows.net/now/now"
}
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .env("PATH", "/definitely/missing")
        .arg("deploy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provider CLI not found"))
        .stderr(predicate::str::contains("npm install -g firebase-tools"));
}

#[test]
fn deploy_reports_default_url_from_auto_selection_rules() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    let bin_dir = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "firebase-hosting",
  "base_url": "https://infinitybin.blob.core.windows.net/now/now"
}
"#,
        )
        .unwrap();

    #[cfg(unix)]
    write_fake_cli(
        &bin_dir,
        "firebase",
        "#!/bin/sh\nprintf 'Project Console: https://console.firebase.google.com/project/demo\\nHosting URL: https://demo.web.app\\n'\n",
    );
    #[cfg(windows)]
    write_fake_cli(
        &bin_dir,
        "firebase",
        "@echo off\r\necho Project Console: https://console.firebase.google.com/project/demo\r\necho Hosting URL: https://demo.web.app\r\n",
    );

    let output = now_cmd(&config_home)
        .current_dir(site.path())
        .env("PATH", path_with_fake_bin(&bin_dir))
        .arg("deploy")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hosting URL: https://demo.web.app"));
    assert!(
        stdout.contains(
            "\nDefault URL: https://infinitybin.blob.core.windows.net/now/now/index.html\n"
        )
    );
    assert!(!stdout.contains("\nDefault URL: https://demo.web.app\n"));
}

#[test]
fn firebase_hosting_provider_accepts_display_name_and_legacy_alias() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(r#"{ "provider": "firebase" }"#)
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--provider", "Firebase Hosting", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider: Firebase Hosting"))
        .stdout(predicate::str::contains("firebase deploy --only hosting"));
}

#[test]
fn missing_provider_in_non_interactive_mode_does_not_prompt() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--dry-run"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("provider is not configured"))
        .stderr(predicate::str::contains("Choose a provider").not());
}

#[test]
fn azure_blob_dry_run_does_not_require_azure_cli_or_print_sas_secret() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "azure-storage-blob",
  "azure_blob": {
    "sas_url_env": "NOW_AZURE_BLOB_SAS_URL"
  }
}
"#,
        )
        .unwrap();
    site.child(".env")
        .write_str(
            r#"NOW_AZURE_BLOB_SAS_URL="https://infinitybin.blob.core.windows.net/now/now?sv=1&sig=secret"
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .env("PATH", "/definitely/missing")
        .args(["deploy", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Azure Storage Blob SAS upload"))
        .stdout(predicate::str::contains(
            "Default URL: https://infinitybin.blob.core.windows.net/now/now/index.html",
        ))
        .stdout(predicate::str::contains("secret").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn azure_blob_dry_run_supports_prefix_and_masks_sas() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "azure-storage-blob",
  "azure_blob": {
    "sas_url_env": "NOW_AZURE_BLOB_SAS_URL",
    "prefix": "my-project/sub"
  }
}
"#,
        )
        .unwrap();
    site.child(".env")
        .write_str(
            r#"NOW_AZURE_BLOB_SAS_URL="https://infinitybin.blob.core.windows.net/now/now?sv=1&sig=secret"
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .env("PATH", "/definitely/missing")
        .args(["deploy", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Azure Storage Blob SAS upload"))
        .stdout(predicate::str::contains(
            "https://infinitybin.blob.core.windows.net/now/now/my-project/sub?<redacted>",
        ))
        .stdout(predicate::str::contains(
            "Default URL: https://infinitybin.blob.core.windows.net/now/now/my-project/sub/index.html",
        ))
        .stdout(predicate::str::contains("secret").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn prefix_flag_overrides_azure_blob_config() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "azure-storage-blob",
  "azure_blob": {
    "sas_url_env": "NOW_AZURE_BLOB_SAS_URL",
    "prefix": "configured"
  }
}
"#,
        )
        .unwrap();
    site.child(".env")
        .write_str(
            r#"NOW_AZURE_BLOB_SAS_URL="https://acct.blob.core.windows.net/$web?sv=1&sig=secret"
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--prefix", "one-off/sub", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/$web/one-off/sub?<redacted>"))
        .stdout(predicate::str::contains("/$web/configured?").not());
}

#[test]
fn azure_storage_blob_provider_accepts_display_name_as_cli_value() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "azure_blob": {
    "sas_url_env": "NOW_AZURE_BLOB_SAS_URL"
  }
}
"#,
        )
        .unwrap();
    site.child(".env")
        .write_str(
            r#"NOW_AZURE_BLOB_SAS_URL="https://acct.blob.core.windows.net/$web?sv=1&sig=secret"
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--provider", "Azure Storage Blob", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider: Azure Storage Blob"))
        .stdout(predicate::str::contains("secret").not());
}

#[test]
fn azure_static_web_app_provider_accepts_display_name() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "azure_swa": {
    "deployment_token_env": "SWA_TOKEN"
  }
}
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--provider", "Azure Static Web App", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider: Azure Static Web App"))
        .stdout(predicate::str::contains("swa deploy"));
}

#[test]
fn azure_static_web_app_deploy_reads_token_from_env_file() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    let bin_dir = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "azure-static-web-app",
  "azure_swa": {
    "deployment_token_env": "MY_SWA_TOKEN"
  }
}
"#,
        )
        .unwrap();
    site.child(".env")
        .write_str("MY_SWA_TOKEN=token-from-dotenv\n")
        .unwrap();

    #[cfg(unix)]
    write_fake_cli(
        &bin_dir,
        "swa",
        "#!/bin/sh\nprintf '%s' \"$SWA_CLI_DEPLOYMENT_TOKEN\"\n",
    );
    #[cfg(windows)]
    write_fake_cli(
        &bin_dir,
        "swa",
        "@echo off\r\n<nul set /p =%SWA_CLI_DEPLOYMENT_TOKEN%\r\n",
    );

    now_cmd(&config_home)
        .current_dir(site.path())
        .env_remove("MY_SWA_TOKEN")
        .env("PATH", path_with_fake_bin(&bin_dir))
        .arg("deploy")
        .assert()
        .success()
        .stdout(predicate::str::contains("token-from-dotenv"));
}

#[test]
fn any_website_ftp_provider_accepts_display_name() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "ftp": {
    "host": "ftp.example.com"
  }
}
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--provider", "Any Website (FTP)", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Provider: Any Website (FTP)"))
        .stdout(predicate::str::contains("lftp"));
}

#[test]
fn remote_dir_flag_overrides_ftp_config() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "any-website-ftp",
  "ftp": {
    "host": "ftp.example.com",
    "remote_dir": "/configured"
  }
}
"#,
        )
        .unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["deploy", "--remote_dir", "/one-off", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/one-off"))
        .stdout(predicate::str::contains("/configured").not());
}
