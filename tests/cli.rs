use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn now_cmd(config_home: &TempDir) -> Command {
    let mut command = Command::cargo_bin("now").unwrap();
    command.env("NOW_CONFIG_HOME", config_home.path());
    command
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
fn deploy_dry_run_uses_configured_provider() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(
            r#"{
  "provider": "firebase",
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
            "https://example.web.app/index.html",
        ));
}

#[test]
fn config_set_and_get_local_value() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["config", "set", "provider", "firebase"])
        .assert()
        .success();

    now_cmd(&config_home)
        .current_dir(site.path())
        .args(["config", "get", "provider"])
        .assert()
        .success()
        .stdout(predicate::str::contains("firebase"));
}

#[test]
fn missing_provider_cli_returns_install_hint() {
    let site = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    site.child("public/index.html").write_str("ok").unwrap();
    site.child(".now.json")
        .write_str(r#"{ "provider": "firebase" }"#)
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
