# now

**now 是一個 Rust CLI，用來把靜態網站交給既有 provider CLI 部署。**

它不嘗試重寫 Firebase、Azure 或 FTP 的部署流程；它負責選擇可發布目錄、讀取非祕密設定、組合 provider CLI 命令，並在部署後輸出預設 URL。

* * *

## 支援平台

| 平台 | Release asset |
| --- | --- |
| macOS Apple Silicon | `now-aarch64-apple-darwin.tar.xz` |
| macOS Intel | `now-x86_64-apple-darwin.tar.xz` |
| Linux x64 glibc | `now-x86_64-unknown-linux-gnu.tar.xz` |
| Windows x64 | `now-x86_64-pc-windows-msvc.zip` |

**初版不支援 Linux arm64 與 musl。**

每個 archive 旁邊都會有同名 `.sha256` 檔案。

* * *

## 安裝

### npm

```sh
npm install -g @willh/now
```

npm 套件只包含 JavaScript wrapper 與安裝邏輯。安裝時會從 GitHub Release 下載目前平台的原生 binary，並驗證 SHA-256 checksum。

### Unix-like

```sh
curl -fsSL https://raw.githubusercontent.com/doggy8088/now/main/install.sh | sh
```

預設安裝到 `$HOME/.local/bin`。可用 `NOW_INSTALL_DIR` 覆寫：

```sh
NOW_INSTALL_DIR=/usr/local/bin sh install.sh
```

### Windows PowerShell

```powershell
iwr https://raw.githubusercontent.com/doggy8088/now/main/install.ps1 -OutFile install.ps1
.\install.ps1
```

預設安裝到 `$env:LOCALAPPDATA\now\bin`。可用 `-InstallDir` 覆寫：

```powershell
.\install.ps1 -InstallDir "$env:USERPROFILE\bin"
```

### 手動下載

1. 到 `https://github.com/doggy8088/now/releases/latest` 下載符合平台的 archive。
2. 下載同名 `.sha256`。
3. 驗證 checksum。
4. 解壓縮後把 `now` 或 `now.exe` 放進 `PATH` 內的目錄。

* * *

## 快速開始

```sh
now config init
now config set provider firebase
now
now deploy
now deploy dist
```

`now [path]` 等同 `now deploy [path]`。

* * *

## 設定檔

now 讀取兩種設定檔：

| 類型 | 路徑 |
| --- | --- |
| 本機設定 | `.now.json` |
| 全域設定 | `~/.config/now/settings.json` |

合併優先序如下：

1. CLI flags
2. `.now.json`
3. `~/.config/now/settings.json`

**設定檔只保存非祕密設定。token、password、secret、account key 不應寫入設定檔。**

完整範例：

```json
{
  "provider": "firebase",
  "source": null,
  "base_url": "https://example.web.app",
  "default_url": null,
  "firebase": {
    "project": "my-firebase-project",
    "site": null
  },
  "azure_blob": {
    "account": "mystorageaccount",
    "container": "$web",
    "destination_path": null
  },
  "azure_swa": {
    "app_name": null,
    "environment": "production",
    "deployment_token_env": "SWA_CLI_DEPLOYMENT_TOKEN"
  },
  "ftp": {
    "host": "ftp.example.com",
    "remote_dir": "/public_html",
    "username_env": "NOW_FTP_USERNAME",
    "password_env": "NOW_FTP_PASSWORD"
  }
}
```

常用設定命令：

```sh
now config init
now config init --global
now config set provider firebase
now config set firebase.project my-project
now config get
now config get provider
now config doctor
```

* * *

## Provider

### Firebase Hosting

需求：

```sh
npm install -g firebase-tools
firebase login
```

建議設定：

```sh
now config set provider firebase
now config set firebase.project my-project
now config set base_url https://my-project.web.app
```

部署時會呼叫：

```sh
firebase deploy --only hosting
```

若設定 `firebase.site`，會改用 `hosting:<site>`。

### Azure Blob static website

需求：

```sh
az login
```

Azure Storage static website 通常使用 `$web` container。建議設定：

```sh
now config set provider azure-blob
now config set azure_blob.account mystorageaccount
now config set azure_blob.container '$web'
now config set base_url https://mystorageaccount.z13.web.core.windows.net
```

部署時會呼叫：

```sh
az storage blob upload-batch --source <source> --destination '$web' --overwrite true --auth-mode login --account-name mystorageaccount
```

### Azure Static Web Apps

需求：

```sh
npm install -g @azure/static-web-apps-cli
```

建議把 deployment token 放在環境變數：

```sh
export SWA_CLI_DEPLOYMENT_TOKEN=...
now config set provider azure-swa
now config set azure_swa.environment production
now config set azure_swa.deployment_token_env SWA_CLI_DEPLOYMENT_TOKEN
```

部署時會呼叫：

```sh
swa deploy <source> --env production
```

### FTP

需求：

```sh
lftp --version
```

帳號密碼請使用環境變數：

```sh
export NOW_FTP_USERNAME=deploy-user
export NOW_FTP_PASSWORD=...
now config set provider ftp
now config set ftp.host ftp.example.com
now config set ftp.remote_dir /public_html
```

部署時會用 `lftp mirror -R --only-newer` 上傳。初版不做遠端刪除同步。

* * *

## 來源目錄規則

未指定 path 時，now 會依序尋找：

1. `dist/`
2. `build/`
3. `public/`

若三者都不存在：

1. 互動式終端機會詢問是否把目前目錄的可發布檔案移到 `public/`。
2. 若拒絕或處於非互動式環境，會部署目前目錄。
3. 部署目前目錄時會排除 `.now.json`、`.git/`、`node_modules/`、`target/` 與暫存檔。

指定 path 時會直接使用該目錄：

```sh
now deploy dist
now ./public
```

* * *

## URL 選擇規則

部署後輸出的 URL 依序選擇：

1. `.now.json` 或全域設定中的 `default_url`
2. `<base_url>/index.html`
3. `<base_url>/index.htm`
4. 根目錄唯一的 `.html` 或 `.htm` 頁面
5. provider base URL

若沒有 `base_url`，檔案規則會輸出相對頁面名稱，例如 `index.html`。

* * *

## 常用命令

```sh
now --help
now
now public
now deploy
now deploy dist --provider firebase
now deploy --dry-run
now deploy --dry-run --json
now config get --global
now config doctor
```

* * *

## 安全性

**不要把 token、password、secret 或 account key 寫入 `.now.json`。**

建議做法：

| 類型 | 建議 |
| --- | --- |
| Firebase | 使用 `firebase login` 的既有登入狀態 |
| Azure Blob | 使用 `az login` 與 `--auth-mode login` |
| Azure Static Web Apps | 使用 `SWA_CLI_DEPLOYMENT_TOKEN` 或自訂 token 環境變數 |
| FTP | 使用 `NOW_FTP_USERNAME` 與 `NOW_FTP_PASSWORD` 環境變數 |

`now config set` 會拒絕明顯像祕密的 key，但仍應避免把敏感值放進 repository。

* * *

## 疑難排解

| 問題 | 處理方式 |
| --- | --- |
| provider CLI 找不到 | 安裝對應 CLI，並確認它在 `PATH` 內 |
| 沒有部署權限 | 先用 provider CLI 完成登入與權限確認 |
| 找不到預設 URL | 設定 `default_url` 或 `base_url` |
| npm 安裝時下載 release asset 失敗 | 確認版本對應的 GitHub Release asset 已發布 |
| checksum 驗證失敗 | 刪除安裝快取後重裝，並確認 release asset 與 `.sha256` 來自同一個版本 |

macOS DNS 快取異常時，可先執行：

```sh
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

* * *

## 開發者指南

需求：

```sh
rustc --version
cargo --version
node --version
npm --version
```

常用流程：

```sh
make check
make test
make release-build
make npm-pack
make install-local
```

release asset 命名必須固定：

```text
now-aarch64-apple-darwin.tar.xz
now-aarch64-apple-darwin.tar.xz.sha256
now-x86_64-apple-darwin.tar.xz
now-x86_64-apple-darwin.tar.xz.sha256
now-x86_64-unknown-linux-gnu.tar.xz
now-x86_64-unknown-linux-gnu.tar.xz.sha256
now-x86_64-pc-windows-msvc.zip
now-x86_64-pc-windows-msvc.zip.sha256
```

CI 會執行 Rust format、clippy、Rust 測試與 npm 測試。release workflow 會建立跨平台 binary archive 與 checksum。npm publish workflow 使用：

```sh
npm publish --provenance --access public
```

* * *

## 授權與貢獻

本專案採用 MIT License。

貢獻前請先執行：

```sh
make check
```
