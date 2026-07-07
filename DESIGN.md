# Design

## Visual theme

「午後部署」——午後陽光裡從容運轉的工具。溫暖的 ochre/honey 主色在純白上發聲，深墨藍作為編輯對比，機械般的終端機片段誠實呈現部署流程。整體像一份用心印製的技術手冊：節制、精確、有溫度。

## Color palette (OKLCH)

| Role | OKLCH | Use |
| --- | --- | --- |
| `--bg` | `oklch(1.000 0.000 0)` | 純白底色——溫度由主色攜帶，不藏在底色 |
| `--surface` | `oklch(0.975 0.010 77)` | 區塊／卡片底，極淡的暖偏移 |
| `--ink` | `oklch(0.220 0.012 60)` | 正文深墨，對比 ≥ 12:1 |
| `--primary` | `oklch(0.720 0.135 70)` | honey/ochre 主色；其上用純白文字 |
| `--accent` | `oklch(0.420 0.110 235)` | 深墨藍，連結／徽章／編輯對比 |
| `--muted` | `oklch(0.500 0.012 60)` | 次要文字，對比 ≥ 4.5:1 |
| `--rule` | `oklch(0.900 0.010 77)` | 細分隔線 |

## Typography

對比軸：CJK 襯線展示（Noto Serif TC）+ 拉丁 grotesque 展示（Bricolage Grotesque）；正文用 Noto Sans TC + Schibsted Grotesk；命令與設定用 JetBrains Mono。 elegance 來自字重對比與節制，不是斜體 drop cap。

| Role | Font | Weights |
| --- | --- | --- |
| CJK display | Noto Serif TC | 600 / 700 |
| CJK body | Noto Sans TC | 400 / 500 |
| Latin display | Bricolage Grotesque | 700 / 800 |
| Latin body | Schibsted Grotesk | 400 / 500 |
| Mono | JetBrains Mono | 500 |

- 中文標題 `text-wrap: balance`；長文 `text-wrap: pretty`。
- 中文行高較拉丁略寬（1.7 body / 1.15 heading）。
- 展示標題 clamp 上限 ≤ 5rem；letter-spacing ≥ -0.03em。
- mono 僅用於命令、設定鍵值、終機輸出。

## Layout

- 單欄長卷，刻意節奏：hero 寬鬆 → 命令區緊湊 → providers 寬鬆。
- 不用巢狀卡片；區塊用留白與細 rule 分隔，不用框。
- 回應式：`repeat(auto-fit, minmax(280px, 1fr))` 用於 provider 網格，無 breakpoint。
- hero 用 asymmetric：左側文案，右側 atmosphere image 與終機片段。
- 結尾 CTA 單一聚焦。

## Motion

- 頁面載入一次編排：hero 文字以 ease-out-quart 上浮，終機片段逐字打字感出現。
- 所有動畫有 `prefers-reduced-motion` 靜態替代。
- 不 bounce、不 elastic、不 animate layout 屬性。

## Components

- 命令晶片：mono + 1px rule + 微暖底，承載真實 `now` 命令。
- 終機片段：深墨底 + ochre prompt + 白色輸出，模擬真實 deploy 輸出。
- provider 列：名稱 + 命令 + 一句說明，不用圖示卡片。
- 章節節奏用大號展示標題 + 短引言，不用編號眉題。

## Imagery

- Hero：暖 ochre 光透過建築垂直縫隙的抽象氛圍圖，午後、克制、有揚升感。負空間留文案。
- 區段質感：手印紙／刷紋 ochre pigment 的微距質感，作為低調的區段背景。
- 不用通用 stock；兩張圖都服務「午後部署」的場景設定。