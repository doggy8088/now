# 變更記錄

本文件記錄 `now` 各版本的重要變更。

格式參考 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.1.0/)，版本編號遵循 [Semantic Versioning](https://semver.org/lang/zh-TW/)。

* * *

## [Unreleased]

* * *

## [0.1.2] - 2026-07-19

### 新增

- 部署命令新增 `--source`、`--prefix` 與 `--remote_dir` 具名旗標，可在單次部署覆寫來源目錄、Azure Storage Blob 前綴與 FTP 遠端目錄，且不會改寫 `.now.json`。
- `--remote-dir` 可作為 `--remote_dir` 的別名。

### 變更

- 將 GitHub Actions 的 artifact、Node.js 與 Pages actions 升級至 Node.js 24 版本，消除 Node.js 20 棄用警告。

### 修正

- Release workflow 在 GitHub Release 資產發布成功後直接呼叫 npm 發布 workflow，不再依賴 `GITHUB_TOKEN` 無法遞迴觸發的 `release.published` 事件。
- npm 發布 workflow 新增 release tag 格式與 `package.json` 版本一致性驗證，並保留帶入 tag 的手動執行入口。

* * *

## [0.1.1] - 2026-07-18

### 新增

- 新增 `--verbose` 詳細偵錯模式，可顯示設定檔、來源選擇、外部命令、工作目錄與環境變數對應等診斷資訊。
- Azure Storage Blob 新增 `azure_blob.prefix`，可部署到 container 內的指定子目錄，並同步套用於公開 URL 推導。
- 首次設定明確指定來源路徑時，會驗證目錄並保存到 `.now.json` 的 `source`；專案內保存相對路徑，專案外保存絕對路徑。
- 首次設定可記住是否將可發布檔案移到 `public/`，後續部署不再重複詢問。
- 新增 `.env` 讀寫支援，供 Azure Storage Blob SAS URL 與 Azure Static Web App deployment token 安全使用。
- 新增支援五種語系、響應式版面、淺色與深色主題的公開網站，並加入 SEO、OpenGraph、結構化資料與 GitHub Pages 自動部署。

### 變更

- 設定初始化入口由 `now config init` 改為頂層 `now init`，並支援重新設定現有專案。
- Azure Storage Blob onboarding 可沿用既有 `.env` SAS URL，顯示時會遮蔽敏感 query string。
- Azure Static Web App 部署可從專案 `.env` 載入 deployment token。
- 部署目前目錄、明確指定目前目錄或設定 `"source": "."` 時，都會排除 `.now.json`、`.env`、`.env.*`、`.git/`、`node_modules/`、`target/` 與暫存檔。
- GitHub Actions 升級 checkout 與 setup-node，並修正 Release 資產發布流程。

### 修正

- Azure Storage Blob SAS URL 不再寫入 `.now.json`，改由 `azure_blob.sas_url_env` 指向環境變數或 `.env`。
- 無效的明確來源路徑會在首次設定前失敗，不會留下不完整的 `.now.json`。
- 後續明確指定來源路徑只影響當次部署，不會暗中覆寫既有 `source`。
- 修正公開網站的多語系標題切換、鍵盤操作、標題階層、主題參數優先序與版面配置問題。

* * *

## [0.1.0] - 2026-07-07

### 新增

- 首次發布 `now` Rust CLI，可用單一命令部署靜態網站。
- 支援 Firebase Hosting、Azure Storage Blob、Azure Static Web App 與 Any Website (FTP)。
- 支援命令列路徑、`dist/`、`build/`、`public/` 與目前目錄的來源選擇規則。
- 支援本機 `.now.json`、全域設定、命令列 provider 覆寫與設定檔診斷。
- 支援 dry-run、JSON 部署摘要與預設公開 URL 推導。
- Azure Storage Blob 使用內建 REST API 上傳，不需要 Azure CLI。
- 提供 macOS Apple Silicon、macOS Intel、Linux x64 glibc 與 Windows x64 原生執行檔及 SHA-256 checksum。
- 提供 npm、Unix-like 安裝腳本、Windows PowerShell 安裝腳本與 GitHub Release 發布流程。

[Unreleased]: https://github.com/doggy8088/now/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/doggy8088/now/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/doggy8088/now/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/doggy8088/now/releases/tag/v0.1.0
