# `now` 專案發版契約

使用 `bump-and-release` 處理此 repository 時，先確認以下目前契約；若 workflow 或檔案已變更，以實際檔案為準。

## 版本來源

- `Cargo.toml`：`[package]` 的 `version`。
- `Cargo.lock`：root package `now` 的 `version`。
- `package.json`：npm 套件 `@willh/now` 的 `version`。
- 目前沒有 `package-lock.json`；若日後新增，必須同步 root version。
- Git tag 格式為 `vX.Y.Z`。

## CHANGELOG

- `CHANGELOG.md` 使用 Keep a Changelog 結構與正體中文分類。
- 保留 `## [Unreleased]`，正式版本放在其後。
- 底部維護 `[Unreleased]` 與每個版本的 compare/release 連結。

## 本機檢查

使用：

```sh
make check
```

此 target 會執行 Rust format check、Clippy、Rust 測試、npm 測試與 `npm pack --dry-run`。Release build 應使用：

```sh
make release-build
```

## GitHub Actions

- `.github/workflows/ci.yml`：`main` push 與 pull request 觸發，執行 format、Clippy、Rust 測試與 npm 測試。
- `.github/workflows/release.yml`：`v*.*.*` tag 觸發，先跨平台建置 binary，再建立或更新 GitHub Release assets，最後呼叫 npm publish workflow。
- `.github/workflows/npm-publish.yml`：以 tag checkout，驗證 tag 與 `package.json` 版本一致，執行 `npm publish --provenance --access public`。
- npm publish workflow 使用 npm trusted publishing，不能以本機登入狀態代替 GitHub Actions 的成功證據。

## 發布驗證

- GitHub Release 必須不是 draft，且應有四個平台的 archive 與 checksum assets。
- npm package 名稱為 `@willh/now`；以 `npm view "@willh/now@X.Y.Z" version` 查證 registry。
- GitHub Release notes 必須在 npm registry 查證成功後，以該版本 `CHANGELOG.md` 區塊更新，不直接接受 `--generate-notes` 的自動內容。
