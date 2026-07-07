# Product

## Register

brand

## Users

前端工程師與維護靜態網站、文件網站的開發者。他們在專案根目錄、終端機前，已經建置好靜態資產，需要在 Firebase、Azure 或 FTP 之間用一致的命令快速發布。使用情境是專注、單次、低摩擦：一行命令部署，拿到 URL，回去繼續寫東西。

## Product Purpose

`now` 是一支 Rust CLI，把靜態網站交給既有 provider CLI 部署。它不重寫雲端協定，只負責：選擇可發布目錄、讀取非祕密設定、組合 provider CLI 命令、部署後輸出預設 URL。成功就是「一行命令，然後完成」——部署流程從多步驟變成單一步驟。

## Brand Personality

溫暖、機械、誠實。像一份用心印製的技術手冊，或一個午後陽光裡從容運轉的工具。三個詞：considered、reliable、warm。語氣沉穩不張揚，把部署這件例行工作做成值得信任的日常。

## Anti-references

- SaaS 奶油色 dashboard 模板：暖米背景 + 灰字 + 一點強調色，2026 年 AI 預設。
- Vercel 純黑複製：dev tool 就一定要暗黑、霓虹、glow。
- Firebase 紫 / 通用科技紫漸層 hero。
- 編輯型雜誌套路：展示襯線 + 斜體 drop cap + 三欄分隔線 + 單色克制——這是另一層的 AI 反射。
- 漸層文字、glassmorphism 卡片、每個 section 上方的小型大寫追蹤眉題、01/02/03 編號眉題。

## Design Principles

- **一行命令，然後完成。** 把部署流程本身當成 hero——真實的 `now` 命令與終機輸出是頁面的主角，不是裝飾。
- **溫暖不等於暖底色。** 溫度來自 ochre 主色、字體與 imagery；背景維持純白，讓主色發聲，不落入 cream/sand 預設。
- **誠實的終端機。** mono 只出現在它真正該出現的地方——命令、設定、輸出。不拿 mono 當「看起來很 technical」的廉價暗示。
- **用心做細節。** 字距、節奏、留白都經過選擇；elegant 來自節制與精確，不是裝飾。

## Accessibility & Inclusion

- 正文 ≥ 4.5:1 對比；ochre 主色上的文字使用純白以確保可讀。
- 鍵盤可達、focus 可見。
- `prefers-reduced-motion` 提供靜態替代。
- zh-tw 為預設語言；CJK 字體與拉丁字體並重，行高與斷行依中文調整。