# Copilot Instructions — `now`

`now` 是一個 Rust CLI，把靜態網站交給既有 provider CLI 部署。初版重點是包裝與協調，不重新實作雲端 provider 的部署協定。

## Build, test, lint

- 全檢查（CI 等價）：`make check` = `fmt-check` + `clippy`（`-D warnings`）+ `cargo test` + `npm test` + `npm pack --dry-run`。
- 單一 Rust 測試：`cargo test <test_name>`（整合測試在 `tests/cli.rs`，用 `assert_cmd`）。
- npm 測試：`npm test`（跑 `tests/postinstall.test.cjs`）。
- Release build 必須帶 lockfile：`make release-build`（`cargo build --release --locked`）。
- 格式化：`make fmt` / 檢查 `make fmt-check`。Clippy 警告即失敗，提交前務必先跑 `make check`。

## Architecture

- **Rust CLI**（`src/`）：`main.rs` / `cli.rs` 為入口；`config.rs` 定義 `.now.json` schema 與 `ProviderKind`；`provider.rs` 依 provider 組裝 CLI 命令；`deploy.rs` 執行部署；`fs_rules.rs` 自動選靜態資產目錄；`onboarding.rs` 處理首次設定；`env_file.rs` 載入 `.env`；`azure_blob.rs` 是唯一不靠外部 CLI、改用內建 SAS URL 上傳的 provider。
- **npm wrapper**（`npm/`）：`cli.cjs` 為薄殼，`postinstall.cjs` 從 GitHub Release 下載對應平台 native binary 並驗證 SHA-256。套件名 `@willh/now`，binary 名 `now`。改 wrapper 邏輯後要同步更新 `tests/postinstall.test.cjs` 與 `npm/prepublish-check.cjs`。
- **公開網站**（`public/`）：單頁靜態站，經 `.github/workflows/pages.yml` 在 `public/` 有變動時部署到 GitHub Pages。
- **設定**：`.now.json` 只保存非機密設定（provider、source、base_url 等）；token / 密碼 / account key 一律走環境變數或既有 CLI 登入，不寫進設定檔、不進 repo。

## Provider 命名（務必一致）

`src/config.rs` 的 `ProviderKind` 是 provider 名稱的唯一來源，**不要自創簡寫或別名**：

| Provider | id slug | 顯示名稱（`display_name()`） |
| --- | --- | --- |
| Firebase | `firebase-hosting` | Firebase Hosting |
| Azure Blob | `azure-storage-blob` | **Azure Storage Blob** |
| Azure SWA | `azure-static-web-app` | Azure Static Web App |
| FTP | `any-website-ftp` | Any Website (FTP) |

在 UI、文件、commit 訊息中一律用上面的顯示名稱；設定檔與 CLI flag 用 id slug。常見錯誤是把 Azure Storage Blob 寫成「Azure Blob」——請避免。

## 使用者面向文案慣例

- 預設語言為**繁體中文（zh-tw）**；技術詞優先用中文友善說法（如「雲端提供者」而非 raw「provider」），盡量不把內部識別字直接搬進行銷文案。
- 不要在公開網頁顯示版號（如「· v0.1」）除非使用者明確要求；版號屬於 release metadata，不是行銷內容。
- `public/index.html` 是多語系（zh-tw / zh-cn / en / jp / ko），改文案時記得同步 `data-i18n` 對應的所有語系鍵。

## Commit 慣例

- 預設使用**繁體中文（zh-tw）詳細 commit 訊息**，不需每次再特別交代。
- 一個目的一個 commit；跨目的的工作分多次 commit。
- 推送前先跑 `make check` 確保通過。