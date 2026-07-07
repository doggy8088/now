# now 產品需求文件

**本文件記錄 now 初版的產品需求、功能範圍、安裝與發布契約、驗收標準。**

* * *

## 1. 背景與目標

`now` 是一個 Rust CLI，目標是讓使用者以一致的命令部署靜態網站，同時保留各 provider 官方或既有 CLI 的部署能力。

核心定位：

- binary 名稱為 `now`。
- npm package 名稱為 `@willh/now`。
- GitHub Release repository 為 `doggy8088/now`。
- `now` 預設等同 `now deploy`。
- 部署時自動選擇靜態資產目錄。
- 實際部署交給既有 provider CLI 執行。
- 部署後輸出預設 URL。

**初版重點是包裝與協調部署流程，不重新實作雲端 provider 的部署協定。**

* * *

## 2. 使用者與使用情境

目標使用者：

- 前端工程師。
- 維護靜態網站或文件網站的開發者。
- 需要在多種 provider 間切換部署流程的使用者。
- 希望用 npm 或 shell 腳本快速安裝 CLI 的使用者。

主要情境：

- 在專案根目錄直接執行 `now` 完成部署。
- 指定 `dist/`、`build/` 或 `public/` 目錄部署。
- 使用 `.now.json` 保存非祕密設定。
- 使用全域設定保存常用 provider 設定。
- 透過 npm 安裝跨平台 CLI。
- 透過 GitHub Release 手動下載 binary。

* * *

## 3. 功能範圍

### 3.1 CLI 行為

`now [path]` 必須等同：

```sh
now deploy [path]
```

需要支援的子命令：

```sh
now deploy [path] [--provider <firebase|azure-blob|azure-swa|ftp>] [--dry-run] [--json]
now config init [--global|--local]
now config set <key> <value> [--global|--local]
now config get [key] [--global|--local]
now config doctor
```

功能要求：

- `path` 預設為目前目錄。
- `--provider` 可覆寫設定檔 provider。
- `--dry-run` 不執行 provider CLI，只輸出將執行的部署資訊。
- `--json` 以 JSON 格式輸出部署摘要。
- `config doctor` 檢查設定檔、provider、provider CLI 可用性與明顯祕密設定。

### 3.2 來源目錄決策

未指定 `path` 時，依序尋找：

1. `dist/`
2. `build/`
3. `public/`

若三者都不存在：

- 互動式終端機詢問是否把目前目錄可發布檔案移到 `public/`。
- 若使用者拒絕，部署目前目錄。
- 部署目前目錄時需排除 `.now.json`、`.git/`、`node_modules/`、`target/` 與暫存檔。

### 3.3 預設 URL 決策

部署後輸出的 URL 依序選擇：

1. `.now.json` 或全域設定中的 `default_url`
2. `index.html`
3. `index.htm`
4. 根目錄唯一的 `.html` 或 `.htm` 頁面
5. provider base URL

若未設定 `base_url`，可輸出相對檔名。

* * *

## 4. 設定需求

設定檔位置：

| 類型 | 路徑 |
| --- | --- |
| 本機設定 | `.now.json` |
| 全域設定 | `~/.config/now/settings.json` |

設定合併優先序：

1. CLI flags
2. `.now.json`
3. `~/.config/now/settings.json`

**設定檔只保存非祕密設定。token、password、account key 不得寫入設定檔。**

設定範例：

```json
{
  "provider": "firebase",
  "source": null,
  "base_url": "https://example.web.app",
  "default_url": null,
  "firebase": {
    "project": "my-project",
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

* * *

## 5. Provider 需求

### 5.1 Firebase Hosting

需求：

- 使用 Firebase CLI。
- 使用者需自行完成 Firebase CLI 安裝與登入。
- 部署命令為 `firebase deploy --only hosting`。
- 若設定 `firebase.site`，部署目標應使用 `hosting:<site>`。

必要設定：

- `provider`: `firebase`

可選設定：

- `firebase.project`
- `firebase.site`
- `base_url`
- `default_url`

### 5.2 Azure Blob static website

需求：

- 使用 Azure CLI。
- 使用者需自行完成 `az login`。
- 部署命令使用 `az storage blob upload-batch`。
- 預設 container 為 `$web`。
- 初版不自動建立 storage account、container 或 static website 設定。

必要設定：

- `provider`: `azure-blob`
- `azure_blob.account`

可選設定：

- `azure_blob.container`
- `azure_blob.destination_path`
- `base_url`
- `default_url`

### 5.3 Azure Static Web Apps

需求：

- 使用 Azure Static Web Apps CLI。
- 部署命令為 `swa deploy <source>`。
- deployment token 應使用環境變數保存。
- 預設 token 環境變數為 `SWA_CLI_DEPLOYMENT_TOKEN`。

必要設定：

- `provider`: `azure-swa`

可選設定：

- `azure_swa.app_name`
- `azure_swa.environment`
- `azure_swa.deployment_token_env`
- `base_url`
- `default_url`

### 5.4 FTP

需求：

- 使用 `lftp`。
- 帳號與密碼必須使用環境變數保存。
- 初版使用上傳模式，不做遠端刪除同步。

必要設定：

- `provider`: `ftp`
- `ftp.host`

可選設定：

- `ftp.remote_dir`
- `ftp.username_env`
- `ftp.password_env`
- `base_url`
- `default_url`

* * *

## 6. npm 包裝需求

npm package：

- 名稱：`@willh/now`
- `bin`：`now`
- 套件只包含 JavaScript wrapper、postinstall、測試、README、授權與必要 metadata。
- 不得包含原生 binary、`target/`、`node_modules/` 或下載後的暫存目錄。

postinstall 行為：

- 偵測目前平台與 CPU 架構。
- 對應 Rust target。
- 從 GitHub Release 下載 archive。
- 下載同名 `.sha256`。
- 驗證 SHA-256 checksum。
- 解壓縮並安裝 binary 到 npm package 內部 binary 目錄。

支援平台與 target：

| Node platform/arch | Rust target | Archive |
| --- | --- | --- |
| `darwin-arm64` | `aarch64-apple-darwin` | `.tar.xz` |
| `darwin-x64` | `x86_64-apple-darwin` | `.tar.xz` |
| `linux-x64` | `x86_64-unknown-linux-gnu` | `.tar.xz` |
| `win32-x64` | `x86_64-pc-windows-msvc` | `.zip` |

**不支援的平台必須輸出明確錯誤。**

* * *

## 7. Release 需求

Release asset 命名固定如下：

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

GitHub Actions 需求：

- CI workflow 執行 Rust format、clippy、Rust 測試與 npm 測試。
- Release workflow 建立跨平台 binary archive 與 checksum。
- npm publish workflow 使用 `npm publish --provenance --access public`。
- npm publish 前需確認對應版本的 GitHub Release assets 存在。

* * *

## 8. 安裝腳本需求

### 8.1 Unix-like install.sh

需求：

- 從 GitHub latest release 下載對應平台 asset。
- 偵測 macOS arm64、macOS x64、Linux x64。
- 下載 archive 與 `.sha256`。
- 驗證 checksum。
- 預設安裝到 `$HOME/.local/bin`。
- 支援 `NOW_INSTALL_DIR=/custom/bin` 覆寫。
- 若目標目錄不在 `PATH`，輸出明確提示。

### 8.2 Windows install.ps1

需求：

- 從 GitHub latest release 下載 Windows x64 asset。
- 下載 `.zip` 與 `.sha256`。
- 驗證 checksum。
- 預設安裝到 `$env:LOCALAPPDATA\now\bin`。
- 支援 `-InstallDir` 覆寫。
- 若目標目錄不在 `PATH`，輸出明確提示。

* * *

## 9. Makefile 需求

需要提供下列目標：

```sh
make help
make build
make release-build
make test
make fmt
make fmt-check
make lint
make npm-pack
make check
make install-local
make clean
```

驗收要求：

- `make help` 可列出目標用途。
- `make check` 執行格式檢查、lint、測試與 npm pack dry-run。
- `make install-local` 可把 release binary 安裝到指定 prefix。

* * *

## 10. README 需求

README 必須使用正體中文台灣用語，且內容詳細但可操作。

至少包含：

- 專案簡介與用途。
- 支援平台。
- 安裝方式。
- 快速開始。
- 設定檔說明。
- Provider 設定。
- 預設目錄規則。
- URL 選擇規則。
- 常用命令範例。
- 安全性說明。
- 疑難排解。
- 開發者指南。
- 授權與貢獻方式。

README 不應承諾尚未支援的平台或 provider 行為。

* * *

## 11. 測試需求

Rust 單元測試：

- 設定合併優先序。
- 預設來源目錄選擇。
- 排除規則。
- 預設 URL 決策。
- provider command builder 不含明文祕密。

Rust 整合測試：

- `now --help`
- `now deploy --dry-run`
- `now config get/set`
- provider CLI 缺失提示。

npm 測試：

- wrapper 可找到 binary。
- postinstall target mapping 正確。
- checksum 驗證失敗會中止。
- `npm pack --dry-run` 不包含 binary、`target/`、`node_modules/`。

安裝腳本測試：

- `install.sh` 的 macOS arm64、macOS x64、Linux x64 target mapping。
- `install.ps1` 的 Windows x64 target mapping。
- checksum 不符時中止。
- 自訂安裝目錄可運作。

Makefile 驗收：

- `make help`
- `make check`
- `make npm-pack`
- `make install-local`

README 驗收：

- 所有命令與實作一致。
- provider 名稱、設定 key、release repo、asset 名稱與程式一致。
- 全文使用正體中文台灣用語。

* * *

## 12. 非目標

初版不處理下列事項：

- 不自動建立雲端資源。
- 不支援 Linux arm64。
- 不支援 musl Linux。
- 不重新實作 Firebase、Azure 或 FTP 的部署協定。
- FTP 不做遠端刪除同步。
- 不在設定檔保存 token、password、secret 或 account key。

* * *

## 13. 假設

- GitHub Release repo 固定為 `doggy8088/now`。
- Unix-like 直接安裝預設位置為 `$HOME/.local/bin`。
- Windows 直接安裝預設位置為 `$env:LOCALAPPDATA\now\bin`。
- 初版支援 macOS arm64、macOS x64、Linux x64、Windows x64。
- 初版不自動建立雲端資源，只部署到已設定完成的 provider target。
- FTP 初版不做遠端刪除同步，避免誤刪。

* * *

## 14. 驗收標準

**初版完成時，使用者應能透過 `now` 或 `now deploy` 部署已建置完成的靜態網站，並能透過 npm、install script 或 GitHub Release 取得 CLI。**

必要驗收：

- CLI 可正常顯示 help。
- `now deploy --dry-run` 可輸出 provider、source、command 與 URL。
- 設定檔可 init、set、get、doctor。
- provider CLI 缺失時只輸出安裝提示，不自行部署。
- npm postinstall 可下載、驗證並安裝 release binary。
- install scripts 可下載、驗證並安裝 release binary。
- CI、release、npm publish workflow 與 asset 命名契約一致。
- README 與實作保持一致。
