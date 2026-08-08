---
name: bump-and-release
description: Safely bump patch, minor, or major versions with patch as the default, update CHANGELOG.md, create a detailed Traditional Chinese Conventional Commit, push the commit and vX.Y.Z tag, wait for GitHub Actions, verify npm publication, and update the published GitHub Release notes. Use when the user asks to bump a version, cut a release, publish a release, release a patch/minor/major version, or ship a new npm package.
---

# Bump and Release

## 目標

依照使用者指定的版本層級建立可追溯的正式發版。未指定層級時使用 patch；只接受 `patch`、`minor`、`major`，不自行推斷 prerelease 或自訂版本。

將版本來源、`CHANGELOG.md`、本機檢查、Conventional Commit、Git tag、GitHub Actions、npm 發布與 GitHub Release notes 綁在同一條可驗證流程中。任何必要檢查失敗時停止，不能宣稱已發布。

先讀取 [references/now-release-contract.md](references/now-release-contract.md)；若目前 repository 與該契約不同，先以 repository 的實際設定為準，並重新確認發布入口與版本來源。

## 版本層級

將使用者請求正規化如下：

| 使用者輸入 | 動作 |
| --- | --- |
| 無輸入 | `patch` |
| `patch` | 遞增第三段版本號，例如 `0.1.2` → `0.1.3` |
| `minor` | 遞增第二段並將第三段歸零，例如 `0.1.2` → `0.2.0` |
| `major` | 遞增第一段並將其餘歸零，例如 `0.1.2` → `1.0.0` |

遇到其他版本格式或層級時停止並指出原因。

## 發版前檢查

1. 解析 repository root、目前 branch、upstream、remote 與 GitHub repository；確認 `git`、`cargo`、`node`、`npm`、`gh` 可用。
2. 執行 `git status --short`。工作樹不是乾淨狀態時停止，不得 stash、reset、checkout 或納入未授權的變更。
3. 執行 `gh auth status`，確認具備讀取 repository、Actions、Release 與推送所需權限。未登入時停止，要求使用者先完成登入。
4. 讀取所有版本來源並確認一致。對本專案至少檢查 `Cargo.toml`、`Cargo.lock` 與 `package.json`；若存在 `package-lock.json`，也檢查其 root version。
5. 找出上一個符合 `vX.Y.Z` 的 tag，檢查目前版本與 tag 是否一致；找不到上一個 tag 時保留此事實並以全歷史建立 changelog。
6. 確認目標版本、目標 tag、GitHub Release 與 npm 版本尚未存在。已存在任何一項時停止，不覆寫、不刪除、不強制推送。
7. 先執行 repository 定義的本機檢查，例如 `make check`。若檢查失敗，先處理失敗或回報阻塞，不進行版本修改、提交或推送。

## 更新版本與變更記錄

1. 使用隨附腳本 `scripts/bump_versions.py` 更新所有已確認的版本來源：

   ```sh
   python3 <skill-directory>/scripts/bump_versions.py --root . --bump <patch|minor|major>
   ```

   腳本會先驗證版本一致，再計算目標版本；不要手動只修改其中一個檔案。先以 `--dry-run` 檢查計算結果，再執行實際更新。

2. 從上一個 release tag 到目前 HEAD 閱讀 `git log`、`git diff`、測試變更與使用者此次工作內容。只記錄可由 repository 證明的變更，不捏造功能或發布結果。
3. 在 `CHANGELOG.md` 的 `## [Unreleased]` 後新增：

   ```markdown
   ## [X.Y.Z] - YYYY-MM-DD

   ### 新增

   - ...

   ### 變更

   - ...

   ### 修正

   - ...
   ```

   只保留實際有內容的分類；日期使用執行環境的當地日期。保留 `Unreleased` 區塊，並同步更新底部比較連結，使 `[Unreleased]` 指向 `vX.Y.Z...HEAD`，新版本連結指向上一版至新版本的比較範圍。

4. 用 `git diff` 檢查版本、CHANGELOG 與任何預期變更；確認沒有 lockfile、格式或其他不相關的意外修改。若發現意外修改，停止並回報。

## 本機驗證與提交

1. 版本與 changelog 更新後再次執行完整檢查。對本專案使用 `make check`；至少確認格式、Clippy、Rust 測試、npm 測試與 npm package dry-run 均通過。
2. 確認 `npm pack --dry-run` 的內容包含必要的 `CHANGELOG.md`，且不包含 native binary、`target/` 或其他不應發布的檔案。
3. 建立 UTF-8 純文字提交訊息暫存檔，每次使用不同路徑：

   ```sh
   commit_msg_file="$(mktemp -t codex-commit-message)"
   ```

   提交訊息必須符合 CC 1.0.0：第一行為 `<type>(<scope>): <summary>`，第二行為空白行，後文以正體中文詳細說明版本、變更分類、影響範圍、本機驗證與發布流程。使用現在式，summary 簡潔且不超過 72 字元。提交固定使用：

   ```sh
   git commit -F "$commit_msg_file"
   ```

   不使用 `git commit -m`，不把測試尚未執行的結果寫成已通過，也不在提交訊息中宣稱 npm 或 GitHub Release 已成功。

4. 提交前執行 `git diff --check`、`git status --short` 與 `git diff --cached --stat`。只 stage 版本檔、`CHANGELOG.md`、必要 lockfile 及此次明確授權的檔案。

## 推送提交與 release tag

1. 先提交版本變更，再確認提交 SHA 與工作樹狀態。
2. 使用已確認的 branch 與 remote 推送提交：

   ```sh
   git push <remote> <branch>
   ```

3. 建立註解 tag `vX.Y.Z`，再推送 tag：

   ```sh
   git tag -a "vX.Y.Z" -m "Release vX.Y.Z"
   git push <remote> "vX.Y.Z"
   ```

   不使用 `--force`。推送完成後確認 tag 指向剛提交的 SHA。

## 等待 CI、Release 與 npm

1. 找出此次提交對應的 CI workflow run 與 tag 對應的 release workflow run；以 commit SHA、tag、workflow 名稱與 event 交叉確認，不只取最新一筆 run。

   對本專案可先查詢：

   ```sh
   gh run list --workflow ci.yml --commit "$COMMIT_SHA" --limit 10 \
     --json databaseId,status,conclusion,url,headSha
   gh run list --workflow release.yml --limit 20 \
     --json databaseId,status,conclusion,url,headSha,headBranch,event
   ```

   只 watch `headSha` 等於此次提交、且 `headBranch` 或 tag 等於目標版本的 run。
2. 使用 `gh run watch <run-id> --exit-status` 等待每個相關 run 完成。若尚未出現，短暫輪詢 `gh run list`；等待期間定期回報狀態，不要重複推送或重跑 workflow。
3. CI 或 release workflow 失敗時停止，取得 `gh run view <run-id> --log-failed` 的失敗證據並回報；不得更新 Release notes，也不得宣稱 npm 已發布。
4. Release workflow 成功後，用 `gh release view "vX.Y.Z" --json tagName,isDraft,isPrerelease,publishedAt,url,assets` 確認 GitHub Release 已發布且 assets 存在。
5. 以 npm registry 查證套件版本，而不是只看 Actions 綠燈：

   ```sh
   npm view "@willh/now@X.Y.Z" version --json
   npm view "@willh/now@X.Y.Z" dist.tarball --json
   ```

   兩者都必須成功，且版本必須等於 `X.Y.Z`。Registry 尚未同步時採有限次數輪詢；超過等待上限時停止並明確回報「查無足夠資料」，不得猜測已發布。若本 repository 沒有可驗證的 npm 發布 workflow，停止並要求發布策略，不自行執行不可逆的 `npm publish`。

## 更新 GitHub Release notes

只在 CI、GitHub Release 與 npm registry 都驗證成功後執行：

1. 從 `CHANGELOG.md` 擷取 `## [X.Y.Z]` 區塊，排除底部 link definitions；將該區塊作為 Release notes 的唯一來源，避免使用 GitHub 自動產生的未審核摘要。
2. 使用暫存 UTF-8 檔案執行：

   ```sh
   release_notes_file="$(mktemp -t codex-release-notes)"
   python3 <skill-directory>/scripts/extract_changelog_section.py \
     --changelog CHANGELOG.md --version "X.Y.Z" > "$release_notes_file"
   gh release edit "vX.Y.Z" --notes-file "$release_notes_file"
   ```

3. 再次使用 `gh release view "vX.Y.Z" --json body,url,publishedAt` 讀回並確認 notes 包含此次 changelog 的版本內容、分類與重要變更。確認 Release URL、發布時間、tag 與 npm 版本一致。
4. 發布完成後執行 `git status --short`，確認本機工作樹沒有因 release notes 或暫存檔留下變更。不要為遠端 Release notes 建立額外本機提交，除非使用者另行要求。

## 失敗與中止規則

- 不使用 `git reset --hard`、`git checkout --`、`git clean`、force push 或刪除 tag/release 來繞過檢查。
- 推送前的失敗應留下乾淨且可檢查的本機狀態；若已提交但推送失敗，保留提交並回報 SHA，不重做版本 bump。
- tag 推送後的 CI、Release 或 npm 失敗，保留遠端證據，先修正失敗原因再由使用者決定是否重跑；不建立另一個版本掩蓋問題。
- 任何步驟未能取得可靠證據時，直接說明未完成的階段與可驗證的錯誤，不以推測填補結果。

## 最終回報

以正體中文回報：版本、bump 類型、提交 SHA、tag、GitHub Release URL、CI 與 release run 狀態、npm registry 查證結果、Release notes 更新結果，以及仍待處理的阻塞。只在每一項都有證據時使用「已發布」這個結論。
