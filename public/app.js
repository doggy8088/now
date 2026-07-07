/* now — landing page logic: i18n, theme, reveal, tabs, copy, menus */
(function () {
  "use strict";

  var SHORT = { "zh-TW": "繁中", "zh-CN": "简中", "en": "EN", "ja": "日", "ko": "한" };
  var META = {
    "zh-TW": { title: "now｜一行命令部署靜態網站", desc: "now 讓靜態網站部署收斂成一行命令，整合 Firebase Hosting、Azure 與 FTP 的既有部署流程。選好目錄、組好命令、輸出預設 URL，部署從多步驟變成單一步驟。" },
    "zh-CN": { title: "now｜一行命令部署静态网站", desc: "now 把静态网站部署收敛成一行命令，整合 Firebase Hosting、Azure 和 FTP 的既有部署流程。选好目录、组装命令、输出预设网址，部署从多步骤变成单一步骤。" },
    "en":    { title: "now — ship static sites in one command", desc: "now folds static-site deploy into one command, wrapping the Firebase Hosting, Azure, and FTP flows you already use. Pick the folder, run the command, get the URL." },
    "ja":    { title: "now｜一行で静的サイトをデプロイ", desc: "now は静的サイトのデプロイを一行のコマンドにまとめ、Firebase Hosting・Azure・FTP の既存フローを統合します。フォルダを選び、コマンドを組み立て、URL を返します。" },
    "ko":    { title: "now｜한 줄로 정적 사이트 배포", desc: "now는 정적 사이트 배포를 한 줄 명령으로 묶어, Firebase Hosting·Azure·FTP의 기존 흐름을 통합합니다. 폴더를 고르고, 명령을 조립하고, URL을 돌려줍니다." }
  };

  var T = {
    "zh-TW": {
      "nav.tagline":"一行命令部署靜態網站","nav.why":"為什麼","nav.providers":"支援平台","nav.flow":"流程","nav.config":"設定","nav.install":"安裝",
      "hero.eyebrow":"靜態網站部署 · v0.1","hero.title":"一行命令，<br>把靜態網站<em>交給雲端</em>。","hero.lede":"<code>now</code> 整合 Firebase Hosting、Azure 與 FTP 的部署流程。選好資料夾、組好命令、部署完直接給你網址——部署從一堆步驟，變成一步。","hero.ctaPrimary":"開始使用","copy.label":"複製",
      "hero.meta1":"macOS Apple / Intel","hero.meta2":"Linux x64","hero.meta3":"Windows x64","hero.meta4":"SHA-256 驗證",
      "why.eyebrow":"為什麼是 now","why.title":"不重寫雲端，<br>只把部署收攏成一步。",
      "why.1.h":"不重新造輪子","why.1.p":"now 不重寫 Firebase、Azure 或 FTP，而是接手你已經在用的 CLI。你既有的登入和權限都直接沿用，不用重新設定。",
      "why.2.h":"一支命令，四個平台","why.2.p":"不管背後是 Firebase Hosting、Azure Storage Blob、Azure Static Web Apps 還是 FTP，對你都是同一支 <code>now</code>。換平台只換設定，肌肉記憶不用重學。",
      "why.3.h":"設定檔不碰機密","why.3.p":"<code>.now.json</code> 只存 provider、資料夾、網址這類安全設定。token 和密碼一律走環境變數或既有登入，不寫進設定檔、不進 repo。",
      "prov.eyebrow":"支援平台","prov.title":"四個雲端提供者，<br>各自交給各自的 CLI。","prov.lede":"<code>now</code> 負責選資料夾、組命令，實際部署交給下面的 provider CLI。裝好對應 CLI、登入一次，<code>now</code> 就能接手。",
      "prov.1.p":"沿用 <code>firebase login</code> 的既有登入。設定 <code>firebase.site</code> 時會改用 <code>hosting:&lt;site&gt;</code>。",
      "prov.2.p":"不必裝 Azure CLI。給它一個 container SAS URL，<code>now</code> 就直接用 Azure Storage Blob REST API 把檔案上傳到 <code>$web</code>。",
      "prov.3.p":"deployment token 走環境變數 <code>SWA_CLI_DEPLOYMENT_TOKEN</code>，不寫進設定檔。",
      "prov.4.p":"帳號密碼用 <code>NOW_FTP_USERNAME</code> / <code>NOW_FTP_PASSWORD</code> 環境變數。初版只上傳同步，不做遠端刪除。",
      "flow.eyebrow":"三步完成","flow.title":"設定、選擇、部署，<br>然後完成。",
      "flow.1.h":"建立本機設定","flow.1.p":"在專案根目錄初始化 <code>.now.json</code>，只存非機密設定。",
      "flow.2.h":"選擇 provider 並填入設定","flow.2.p":"指定 provider 與對應的 project、account 或 host 等非機密欄位。",
      "flow.3.h":"部署並拿到網址","flow.3.p":"<code>now [path]</code> 等同 <code>now deploy [path]</code>。部署完成後依序挑出預設網址並輸出。",
      "config.eyebrow":"設定檔","config.title":"兩份設定檔，<br>一份本機、一份全域。",
      "config.merge.h":"合併優先序","config.merge.p":"CLI flags &gt; <code>.now.json</code> &gt; <code>~/.config/now/settings.json</code>。本機設定覆寫全域，命令列 flag 覆寫一切。",
      "config.security.h":"安全性","config.security.p":"token、密碼、account key 一律不寫進設定檔。Firebase Hosting 用 <code>firebase login</code>、Azure Storage Blob 的 SAS URL 放在 <code>.env</code>，SWA 與 FTP 用環境變數。<code>now config set</code> 也會擋下明顯像機密的 key。",
      "install.eyebrow":"安裝","install.title":"三種方式，<br>挑一個順手的。","install.tab.manual":"手動下載",
      "install.npm.p":"npm 套件只含 JavaScript wrapper 與安裝邏輯。安裝時自動從 GitHub Release 下載對應平台原生 binary，並驗證 SHA-256。",
      "install.unix.p":"預設安裝到 <code>$HOME/.local/bin</code>。可用 <code>NOW_INSTALL_DIR=/usr/local/bin</code> 覆寫。",
      "install.win.p":"預設安裝到 <code>$env:LOCALAPPDATA\\now\\bin</code>。可用 <code>-InstallDir</code> 覆寫。",
      "install.manual.p":"下載對應平台 archive 與同名 <code>.sha256</code>，驗證 checksum 後解壓縮，把 <code>now</code> 或 <code>now.exe</code> 放進 PATH 內目錄。",
      "cta.title":"準備好了嗎？","cta.lede":"建置好靜態資產，一行命令把它交給雲端。","cta.github":"查看 GitHub","cta.install":"安裝 now",
      "footer.desc":"把靜態網站交給既有 provider 部署流程。不重寫雲端協定，只把流程收攏成一步。","footer.resources":"資源","footer.latest":"最新 Release","footer.nav":"導覽","footer.tagline":"在午後陽光裡部署靜態網站。","footer.copy":"MIT License · © 2026 Will 保哥",
      "aria.theme":"切換主題","aria.lang":"選擇語言","aria.menu":"選單"
    },
    "zh-CN": {
      "nav.tagline":"一行命令部署静态网站","nav.why":"为什么","nav.providers":"支持平台","nav.flow":"流程","nav.config":"设置","nav.install":"安装",
      "hero.eyebrow":"静态网站部署 · v0.1","hero.title":"一行命令，<br>把静态网站<em>交给云端</em>。","hero.lede":"<code>now</code> 整合 Firebase Hosting、Azure 和 FTP 的部署流程。选好文件夹、组装好命令、部署完直接给你网址——部署从一堆步骤，变成一步。","hero.ctaPrimary":"开始使用","copy.label":"复制",
      "hero.meta1":"macOS Apple / Intel","hero.meta2":"Linux x64","hero.meta3":"Windows x64","hero.meta4":"SHA-256 校验",
      "why.eyebrow":"为什么选 now","why.title":"不重写云端，<br>只把部署收拢成一步。",
      "why.1.h":"不重新造轮子","why.1.p":"now 不重写 Firebase、Azure 或 FTP，而是接手你已经在用的 CLI。你已有的登录和权限都能直接沿用，不用重新设置。",
      "why.2.h":"一条命令，四个平台","why.2.p":"无论背后是 Firebase Hosting、Azure Storage Blob、Azure Static Web Apps 还是 FTP，对你都是同一支 <code>now</code>。换平台只换设置，肌肉记忆不用重学。",
      "why.3.h":"配置文件不碰机密","why.3.p":"<code>.now.json</code> 只存 provider、文件夹、网址这类安全配置。token 和密码一律走环境变量或已有登录，不写进配置文件、不进仓库。",
      "prov.eyebrow":"支持平台","prov.title":"四个 provider，<br>各自交给各自的 CLI。","prov.lede":"<code>now</code> 负责选文件夹、组装命令，实际部署交给下面的 provider CLI。装好对应 CLI、登录一次，<code>now</code> 就能接手。",
      "prov.1.p":"沿用 <code>firebase login</code> 的已有登录。设置 <code>firebase.site</code> 时会改用 <code>hosting:&lt;site&gt;</code>。",
      "prov.2.p":"不必装 Azure CLI。给它一个 container SAS URL，<code>now</code> 就直接用 Azure Storage Blob REST API 把文件上传到 <code>$web</code>。",
      "prov.3.p":"deployment token 走环境变量 <code>SWA_CLI_DEPLOYMENT_TOKEN</code>，不写进配置文件。",
      "prov.4.p":"账号密码用 <code>NOW_FTP_USERNAME</code> / <code>NOW_FTP_PASSWORD</code> 环境变量。初版只上传同步，不做远端删除。",
      "flow.eyebrow":"三步完成","flow.title":"设置、选择、部署，<br>然后完成。",
      "flow.1.h":"创建本地配置","flow.1.p":"在项目根目录初始化 <code>.now.json</code>，只存非机密配置。",
      "flow.2.h":"选择 provider 并填入配置","flow.2.p":"指定 provider 和对应的 project、account 或 host 等非机密字段。",
      "flow.3.h":"部署并拿到网址","flow.3.p":"<code>now [path]</code> 等同 <code>now deploy [path]</code>。部署完成后依次挑出默认网址并输出。",
      "config.eyebrow":"配置文件","config.title":"两份配置文件，<br>一份本地、一份全局。",
      "config.merge.h":"合并优先级","config.merge.p":"CLI flags &gt; <code>.now.json</code> &gt; <code>~/.config/now/settings.json</code>。本地配置覆盖全局，命令行 flag 覆盖一切。",
      "config.security.h":"安全性","config.security.p":"token、密码、account key 一律不写进配置文件。Firebase Hosting 用 <code>firebase login</code>、Azure Storage Blob 的 SAS URL 放在 <code>.env</code>，SWA 和 FTP 用环境变量。<code>now config set</code> 也会挡下明显像机密的 key。",
      "install.eyebrow":"安装","install.title":"三种方式，<br>挑一个顺手的。","install.tab.manual":"手动下载",
      "install.npm.p":"npm 包只含 JavaScript wrapper 和安装逻辑。安装时自动从 GitHub Release 下载对应平台原生 binary，并校验 SHA-256。",
      "install.unix.p":"默认安装到 <code>$HOME/.local/bin</code>。可用 <code>NOW_INSTALL_DIR=/usr/local/bin</code> 覆盖。",
      "install.win.p":"默认安装到 <code>$env:LOCALAPPDATA\\now\\bin</code>。可用 <code>-InstallDir</code> 覆盖。",
      "install.manual.p":"下载对应平台 archive 和同名 <code>.sha256</code>，校验 checksum 后解压，把 <code>now</code> 或 <code>now.exe</code> 放进 PATH 内目录。",
      "cta.title":"准备好了吗？","cta.lede":"构建好静态资源，一行命令把它交给云端。","cta.github":"查看 GitHub","cta.install":"安装 now",
      "footer.desc":"把静态网站交给已有 provider 部署流程。不重写云端协议，只把流程收拢成一步。","footer.resources":"资源","footer.latest":"最新 Release","footer.nav":"导航","footer.tagline":"在午后阳光里部署静态网站。","footer.copy":"MIT License · © 2026 Will 保哥",
      "aria.theme":"切换主题","aria.lang":"选择语言","aria.menu":"菜单"
    },
    "en": {
      "nav.tagline":"Deploy static sites in one line","nav.why":"Why","nav.providers":"Platforms","nav.flow":"Flow","nav.config":"Config","nav.install":"Install",
      "hero.eyebrow":"Static site deploy · v0.1","hero.title":"One line,<br>and your site is <em>live</em>.","hero.lede":"<code>now</code> wraps the deploy tools you already use — Firebase Hosting, Azure, and FTP. Point it at your build folder; it picks the right command, runs it, and hands you the URL. Deploy goes from a dozen steps to one.","hero.ctaPrimary":"Get started","copy.label":"Copy",
      "hero.meta1":"macOS Apple / Intel","hero.meta2":"Linux x64","hero.meta3":"Windows x64","hero.meta4":"SHA-256 verified",
      "why.eyebrow":"Why now","why.title":"No cloud gymnastics.<br>Just one command, then done.",
      "why.1.h":"It doesn't reinvent the cloud","why.1.p":"now doesn't rewrite Firebase, Azure, or FTP. It leans on the CLIs you already trust — your logins and permissions just work, no re-setup.",
      "why.2.h":"One command, four platforms","why.2.p":"Firebase Hosting, Azure Storage Blob, Azure Static Web Apps, or plain FTP — it's the same <code>now</code>. Swap the platform, keep your muscle memory.",
      "why.3.h":"Config stays clean","why.3.p":"<code>.now.json</code> only stores the safe stuff — provider, folder, base URL. Tokens and passwords live in environment variables or your existing logins, never in the file.",
      "prov.eyebrow":"Platforms","prov.title":"Four providers,<br>each handled by its own CLI.","prov.lede":"<code>now</code> picks the folder and builds the command; the actual deploy runs on the provider's CLI below. Install it once, log in, and <code>now</code> takes it from there.",
      "prov.1.p":"Reuses your existing <code>firebase login</code>. Set <code>firebase.site</code> to target <code>hosting:&lt;site&gt;</code>.",
      "prov.2.p":"No Azure CLI needed. Hand it a container SAS URL and <code>now</code> uploads straight to <code>$web</code> via the Azure Storage Blob REST API.",
      "prov.3.p":"The deployment token lives in the <code>SWA_CLI_DEPLOYMENT_TOKEN</code> env var — never written to the config file.",
      "prov.4.p":"Credentials go in <code>NOW_FTP_USERNAME</code> / <code>NOW_FTP_PASSWORD</code> env vars. The first release only syncs uploads, no remote deletes.",
      "flow.eyebrow":"Three steps","flow.title":"Set up, pick, deploy.<br>Then you're done.",
      "flow.1.h":"Create your local config","flow.1.p":"Initialize <code>.now.json</code> in your project root — it only stores the safe stuff.",
      "flow.2.h":"Pick a provider and fill it in","flow.2.p":"Set the provider and its non-secret fields — project, account, or host.",
      "flow.3.h":"Deploy and grab the URL","flow.3.p":"<code>now [path]</code> is shorthand for <code>now deploy [path]</code>. When it's done, <code>now</code> picks the default URL and prints it.",
      "config.eyebrow":"Config","config.title":"Two config files,<br>one local, one global.",
      "config.merge.h":"Merge order","config.merge.p":"CLI flags &gt; <code>.now.json</code> &gt; <code>~/.config/now/settings.json</code>. Local overrides global; a CLI flag overrides everything.",
      "config.security.h":"Security","config.security.p":"Tokens, passwords, and account keys never go in the config file. Firebase Hosting uses <code>firebase login</code>, the Azure Storage Blob SAS URL lives in <code>.env</code>, and SWA and FTP use env vars. <code>now config set</code> also rejects keys that look like secrets.",
      "install.eyebrow":"Install","install.title":"Three ways,<br>pick what's handy.","install.tab.manual":"Manual download",
      "install.npm.p":"The npm package only contains the JavaScript wrapper and install logic. It downloads the native binary for your platform from GitHub Release and verifies the SHA-256.",
      "install.unix.p":"Installs to <code>$HOME/.local/bin</code> by default. Override with <code>NOW_INSTALL_DIR=/usr/local/bin</code>.",
      "install.win.p":"Installs to <code>$env:LOCALAPPDATA\\now\\bin</code> by default. Override with <code>-InstallDir</code>.",
      "install.manual.p":"Download the archive for your platform and its matching <code>.sha256</code>, verify the checksum, then extract and drop <code>now</code> or <code>now.exe</code> into a folder on your PATH.",
      "cta.title":"Ready to ship?","cta.lede":"Build your static assets, then hand them to the cloud in one line.","cta.github":"View on GitHub","cta.install":"Install now",
      "footer.desc":"Hands static sites to your existing provider deploy flows. It doesn't rewrite the cloud — it just folds the steps into one.","footer.resources":"Resources","footer.latest":"Latest release","footer.nav":"Navigation","footer.tagline":"Shipping static sites in the afternoon light.","footer.copy":"MIT License · © 2026 Will",
      "aria.theme":"Toggle theme","aria.lang":"Language","aria.menu":"Menu"
    },
    "ja": {
      "nav.tagline":"静的サイトを一行でデプロイ","nav.why":"目的","nav.providers":"対応プラットフォーム","nav.flow":"流れ","nav.config":"設定","nav.install":"インストール",
      "hero.eyebrow":"静的サイトデプロイ · v0.1","hero.title":"一行のコマンドで、<br>静的サイトを<em>クラウドへ</em>。","hero.lede":"<code>now</code> は Firebase Hosting・Azure・FTP の既存デプロイ手順をまとめます。公開フォルダを選び、コマンドを組み立て、デプロイ後に URL を返します。いくつもの手順が、ひとつに。","hero.ctaPrimary":"はじめる","copy.label":"コピー",
      "hero.meta1":"macOS Apple / Intel","hero.meta2":"Linux x64","hero.meta3":"Windows x64","hero.meta4":"SHA-256 検証",
      "why.eyebrow":"why now","why.title":"クラウドを作り直さない。<br>デプロイをひとつにまとめるだけ。",
      "why.1.h":"車輪を再発明しない","why.1.p":"now は Firebase・Azure・FTP を書き直しません。すでに使っている CLI をそのまま活用します。ログインや権限もそのまま引き継げ、再設定は不要です。",
      "why.2.h":"ひとつのコマンド、4つのプラットフォーム","why.2.p":"Firebase Hosting・Azure Storage Blob・Azure Static Web Apps・FTP、どれも同じ <code>now</code> です。プラットフォームを変えても設定だけ。筋肉記憶はそのまま。",
      "why.3.h":"設定ファイルに秘密は書かない","why.3.p":"<code>.now.json</code> には provider・フォルダ・URL など安全な設定だけを保存します。トークンやパスワードは環境変数や既存ログインに任せ、設定ファイルにもリポジトリにも入れません。",
      "prov.eyebrow":"対応プラットフォーム","prov.title":"4つのプロバイダ、<br>それぞれの CLI にお任せ。","prov.lede":"<code>now</code> はフォルダの選択とコマンドの組み立てを担い、実際のデプロイは下のプロバイダ CLI が実行します。対象 CLI を入れてログインすれば、<code>now</code> があとは引き受けます。",
      "prov.1.p":"<code>firebase login</code> の既存ログインをそのまま利用します。<code>firebase.site</code> を設定すると <code>hosting:&lt;site&gt;</code> に切り替わります。",
      "prov.2.p":"Azure CLI は不要です。コンテナの SAS URL を渡せば、<code>now</code> が Azure Storage Blob REST API で <code>$web</code> に直接アップロードします。",
      "prov.3.p":"デプロイトークンは環境変数 <code>SWA_CLI_DEPLOYMENT_TOKEN</code> に置き、設定ファイルには書きません。",
      "prov.4.p":"アカウントは <code>NOW_FTP_USERNAME</code> / <code>NOW_FTP_PASSWORD</code> 環境変数で渡します。初版はアップロード同期のみで、リモート削除は行いません。",
      "flow.eyebrow":"3ステップで完了","flow.title":"設定・選択・デプロイ、<br>それでおしまい。",
      "flow.1.h":"ローカル設定を作る","flow.1.p":"プロジェクトルートで <code>.now.json</code> を初期化します。秘密以外の設定だけを保存します。",
      "flow.2.h":"プロバイダを選んで設定を入れる","flow.2.p":"provider と、project・account・host などの非機密項目を指定します。",
      "flow.3.h":"デプロイして URL を受け取る","flow.3.p":"<code>now [path]</code> は <code>now deploy [path]</code> と同じです。完了すると、既定の URL を選んで出力します。",
      "config.eyebrow":"設定ファイル","config.title":"設定ファイルは2つ、<br>ローカルとグローバル。",
      "config.merge.h":"マージの優先順位","config.merge.p":"CLI flags &gt; <code>.now.json</code> &gt; <code>~/.config/now/settings.json</code>。ローカルがグローバルを上書きし、CLI flag がすべてを上書きします。",
      "config.security.h":"セキュリティ","config.security.p":"トークン・パスワード・アカウントキーは設定ファイルに書きません。Firebase Hosting は <code>firebase login</code>、Azure Storage Blob の SAS URL は <code>.env</code>、SWA と FTP は環境変数を使います。<code>now config set</code> は秘密っぽいキーを弾きます。",
      "install.eyebrow":"インストール","install.title":"3つの方法、<br>使いやすいものを。","install.tab.manual":"手動ダウンロード",
      "install.npm.p":"npm パッケージには JavaScript ラッパーとインストール処理だけが入ります。インストール時に GitHub Release から該当プラットフォームのネイティブ binary をダウンロードし、SHA-256 を検証します。",
      "install.unix.p":"既定で <code>$HOME/.local/bin</code> にインストールします。<code>NOW_INSTALL_DIR=/usr/local/bin</code> で上書きできます。",
      "install.win.p":"既定で <code>$env:LOCALAPPDATA\\now\\bin</code> にインストールします。<code>-InstallDir</code> で上書きできます。",
      "install.manual.p":"該当プラットフォームの archive と同名の <code>.sha256</code> をダウンロードし、checksum を検証して解凍、<code>now</code> または <code>now.exe</code> を PATH の通ったディレクトリに置きます。",
      "cta.title":"準備はいい？","cta.lede":"静的アセットができたら、一行でクラウドへ。","cta.github":"GitHub を見る","cta.install":"now をインストール",
      "footer.desc":"静的サイトを既存のプロバイダでデプロイします。クラウドを作り直さず、手順をひとつにまとめるだけ。","footer.resources":"リソース","footer.latest":"最新 Release","footer.nav":"ナビゲーション","footer.tagline":"午後の光の中で静的サイトをデプロイ。","footer.copy":"MIT License · © 2026 Will",
      "aria.theme":"テーマ切替","aria.lang":"言語","aria.menu":"メニュー"
    },
    "ko": {
      "nav.tagline":"정적 사이트 한 줄 배포","nav.why":"왜","nav.providers":"지원 플랫폼","nav.flow":"흐름","nav.config":"설정","nav.install":"설치",
      "hero.eyebrow":"정적 사이트 배포 · v0.1","hero.title":"한 줄 명령으로,<br>정적 사이트를 <em>클라우드로</em>.","hero.lede":"<code>now</code>는 Firebase Hosting·Azure·FTP의 기존 배포 흐름을 하나로 묶습니다. 폴더를 고르고, 명령을 조립하고, 배포 뒤 URL을 돌려줍니다. 여러 단계가 한 단계로.","hero.ctaPrimary":"시작하기","copy.label":"복사",
      "hero.meta1":"macOS Apple / Intel","hero.meta2":"Linux x64","hero.meta3":"Windows x64","hero.meta4":"SHA-256 검증",
      "why.eyebrow":"왜 now인가","why.title":"클라우드를 다시 만들지 않고,<br>배포만 한 단계로.",
      "why.1.h":"바퀴를 재발명하지 않습니다","why.1.p":"now는 Firebase·Azure·FTP를 다시 구현하지 않고, 이미 쓰는 CLI에 올라탑니다. 기존 로그인과 권한도 그대로 쓰고, 다시 설정할 필요가 없습니다.",
      "why.2.h":"명령 하나, 플랫폼 넷","why.2.p":"Firebase Hosting·Azure Storage Blob·Azure Static Web Apps·FTP, 모두 같은 <code>now</code>입니다. 플랫폼만 바꾸고 설정만 바꾸면, 익힌 손감각은 그대로.",
      "why.3.h":"설정엔 비밀을 두지 않습니다","why.3.p":"<code>.now.json</code>은 provider·폴더·URL 같이 안전한 설정만 저장합니다. 토큰과 비밀번호는 환경 변수나 기존 로그인에 맡기고, 설정 파일이나 저장소에 두지 않습니다.",
      "prov.eyebrow":"지원 플랫폼","prov.title":"네 개의 provider,<br>각자의 CLI에 맡깁니다.","prov.lede":"<code>now</code>는 폴더 선택과 명령 조립을 맡고, 실제 배포는 아래의 provider CLI가 실행합니다. 해당 CLI를 설치하고 로그인하면, <code>now</code>가 나머지를 이어갑니다.",
      "prov.1.p":"<code>firebase login</code>의 기존 로그인을 그대로 씁니다. <code>firebase.site</code>를 설정하면 <code>hosting:&lt;site&gt;</code>로 바뀝니다.",
      "prov.2.p":"Azure CLI는 필요 없습니다. 컨테이너 SAS URL을 주면, <code>now</code>가 Azure Storage Blob REST API로 <code>$web</code>에 바로 업로드합니다.",
      "prov.3.p":"배포 토큰은 <code>SWA_CLI_DEPLOYMENT_TOKEN</code> 환경 변수에 두고, 설정 파일엔 쓰지 않습니다.",
      "prov.4.p":"계정은 <code>NOW_FTP_USERNAME</code> / <code>NOW_FTP_PASSWORD</code> 환경 변수로 전달합니다. 첫 버전은 업로드 동기화만 하고, 원격 삭제는 하지 않습니다.",
      "flow.eyebrow":"세 단계로 끝","flow.title":"설정·선택·배포,<br>그리고 끝.",
      "flow.1.h":"로컬 설정 만들기","flow.1.p":"프로젝트 루트에서 <code>.now.json</code>을 초기화합니다. 비밀이 아닌 설정만 저장합니다.",
      "flow.2.h":"provider 선택 후 채우기","flow.2.p":"provider와 project·account·host 같은 비밀이 아닌 항목을 지정합니다.",
      "flow.3.h":"배포하고 URL 받기","flow.3.p":"<code>now [path]</code>는 <code>now deploy [path]</code>와 같습니다. 배포가 끝나면 기본 URL을 골라 출력합니다.",
      "config.eyebrow":"설정 파일","config.title":"설정 파일 둘,<br>로컬과 전역.",
      "config.merge.h":"병합 우선순위","config.merge.p":"CLI flags &gt; <code>.now.json</code> &gt; <code>~/.config/now/settings.json</code>. 로컬이 전역을 덮고, CLI flag가 전부를 덮습니다.",
      "config.security.h":"보안","config.security.p":"토큰·비밀번호·계정 키는 설정 파일에 쓰지 않습니다. Firebase Hosting은 <code>firebase login</code>, Azure Storage Blob의 SAS URL은 <code>.env</code>, SWA와 FTP는 환경 변수를 씁니다. <code>now config set</code>도 비밀처럼 보이는 키는 거릅니다.",
      "install.eyebrow":"설치","install.title":"세 가지 방법,<br>편한 것을 고르세요.","install.tab.manual":"수동 다운로드",
      "install.npm.p":"npm 패키지에는 JavaScript 래퍼와 설치 로직만 들어 있습니다. 설치 시 GitHub Release에서 해당 플랫폼의 네이티브 binary를 내려받고 SHA-256을 검증합니다.",
      "install.unix.p":"기본적으로 <code>$HOME/.local/bin</code>에 설치합니다. <code>NOW_INSTALL_DIR=/usr/local/bin</code>으로 덮어쓸 수 있습니다.",
      "install.win.p":"기본적으로 <code>$env:LOCALAPPDATA\\now\\bin</code>에 설치합니다. <code>-InstallDir</code>로 덮어쓸 수 있습니다.",
      "install.manual.p":"해당 플랫폼의 archive와 같은 이름의 <code>.sha256</code>을 내려받아, checksum을 검증한 뒤 압축을 풀고 <code>now</code> 또는 <code>now.exe</code>를 PATH에 있는 디렉터리에 둡니다.",
      "cta.title":"배포할 준비 끝?","cta.lede":"정적 자산이 준비되면, 한 줄로 클라우드로.","cta.github":"GitHub 보기","cta.install":"now 설치",
      "footer.desc":"정적 사이트를 기존 provider 배포 흐름에 맡깁니다. 클라우드를 다시 만들지 않고, 단계만 하나로.","footer.resources":"리소스","footer.latest":"최신 Release","footer.nav":"탐색","footer.tagline":"오후의 햇살 속에서 정적 사이트를 배포합니다.","footer.copy":"MIT License · © 2026 Will",
      "aria.theme":"테마 전환","aria.lang":"언어","aria.menu":"메뉴"
    }
  };

  var FONT_BASE = "https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500;12..96,700;12..96,800&family=JetBrains+Mono:wght@400;500;600&family=Schibsted+Grotesk:opsz,wght@10..72,400;10..72,500;10..72,600";
  var CJK_FONT = {
    "zh-TW": "&family=Noto+Sans+TC:wght@300;400;500;700&family=Noto+Serif+TC:wght@600;700",
    "zh-CN": "&family=Noto+Sans+SC:wght@300;400;500;700&family=Noto+Serif+SC:wght@600;700",
    "ja":    "&family=Noto+Sans+JP:wght@300;400;500;700&family=Noto+Serif+JP:wght@600;700",
    "ko":    "&family=Noto+Sans+KR:wght@300;400;500;700&family=Noto+Serif+KR:wght@600;700",
    "en":    ""
  };

  var html = document.documentElement;
  var LS_LANG = "now-lang", LS_THEME = "now-theme";

  function $(s, ctx) { return (ctx || document).querySelector(s); }
  function $all(s, ctx) { return Array.prototype.slice.call((ctx || document).querySelectorAll(s)); }

  // ---- theme ----
  function applyTheme(t) {
    html.setAttribute("data-theme", t);
    var tc = t === "dark" ? "#1a1410" : "#ffffff";
    $all('meta[name="theme-color"]').forEach(function (m) { m.setAttribute("content", tc); });
  }
  function initTheme() {
    var og = new URLSearchParams(location.search).get("og");
    var qTheme = new URLSearchParams(location.search).get("theme");
    var stored = localStorage.getItem(LS_THEME);
    var pref = og === "1" ? "light" : (qTheme === "dark" || qTheme === "light" ? qTheme : (stored || (window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")));
    applyTheme(pref);
    var btn = $("#themeBtn");
    if (btn) btn.addEventListener("click", function () {
      var next = html.getAttribute("data-theme") === "dark" ? "light" : "dark";
      localStorage.setItem(LS_THEME, next);
      applyTheme(next);
    });
  }

  // ---- language ----
  function loadFont(lang) {
    var link = $("#fontCJK");
    if (link) link.href = FONT_BASE + (CJK_FONT[lang] || "");
  }
  function applyLang(lang) {
    if (!T[lang]) lang = "zh-TW";
    var dict = T[lang];
    html.setAttribute("lang", lang);
    $all("[data-i18n]").forEach(function (el) {
      var k = el.getAttribute("data-i18n");
      if (dict[k] != null) el.innerHTML = dict[k];
    });
    var cur = $("#langCur");
    if (cur) cur.textContent = SHORT[lang] || lang;
    $all("#langMenu li").forEach(function (li) {
      li.classList.toggle("active", li.getAttribute("data-lang") === lang);
    });
    var m = META[lang] || META["zh-TW"];
    document.title = m.title;
    var desc = $('meta[name="description"]'); if (desc) desc.setAttribute("content", m.desc);
    setMeta('og:title', m.title); setMeta('og:description', m.desc);
    setMeta('twitter:title', m.title); setMeta('twitter:description', m.desc);
    var tb = $("#themeBtn"); if (tb) tb.setAttribute("aria-label", dict["aria.theme"]);
    var mb = $("#menuBtn");  if (mb) mb.setAttribute("aria-label", dict["aria.menu"]);
    var lb = $("#langBtn");  if (lb) lb.setAttribute("aria-label", dict["aria.lang"]);
    loadFont(lang);
  }
  function setMeta(prop, val) { var m = $('meta[property="' + prop + '"]'); if (m) m.setAttribute("content", val); }

  function initLang() {
    var q = new URLSearchParams(location.search).get("lang");
    var lang = q || localStorage.getItem(LS_LANG) || (navigator.language && navigator.language.toUpperCase()) || "zh-TW";
    // normalize: zh-Hant/TW -> zh-TW, zh-Hans/CN -> zh-CN
    var L = lang.toUpperCase();
    if (L.indexOf("ZH-TW") === 0 || L.indexOf("ZH-HANT") === 0 || L === "ZH") lang = "zh-TW";
    else if (L.indexOf("ZH-CN") === 0 || L.indexOf("ZH-HANS") === 0) lang = "zh-CN";
    else if (L.indexOf("JA") === 0) lang = "ja";
    else if (L.indexOf("KO") === 0) lang = "ko";
    else if (L.indexOf("EN") === 0) lang = "en";
    if (!T[lang]) lang = "zh-TW";
    applyLang(lang);

    var btn = $("#langBtn"), menu = $("#langMenu");
    if (btn && menu) {
      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        var open = menu.classList.toggle("open");
        btn.setAttribute("aria-expanded", open ? "true" : "false");
      });
      $all("li", menu).forEach(function (li) {
        li.addEventListener("click", function () {
          var l = li.getAttribute("data-lang");
          localStorage.setItem(LS_LANG, l);
          applyLang(l);
          menu.classList.remove("open");
          btn.setAttribute("aria-expanded", "false");
        });
      });
      document.addEventListener("click", function () {
        menu.classList.remove("open"); btn.setAttribute("aria-expanded", "false");
      });
      menu.addEventListener("click", function (e) { e.stopPropagation(); });
    }
  }

  // ---- reveal ----
  function initReveal() {
    var og = new URLSearchParams(location.search).get("og");
    var reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    var els = $all(".reveal");
    if (og === "1") { els.forEach(function (e) { e.classList.add("in"); }); return; }
    if (reduce || !("IntersectionObserver" in window)) {
      els.forEach(function (e) { e.classList.add("in"); });
      return;
    }
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (en) {
        if (en.isIntersecting) { en.target.classList.add("in"); io.unobserve(en.target); }
      });
    }, { threshold: 0.12, rootMargin: "0px 0px -8% 0px" });
    els.forEach(function (e) { io.observe(e); });
  }

  // ---- install tabs ----
  function initTabs() {
    var tabs = $all(".tab");
    tabs.forEach(function (t) {
      t.addEventListener("click", function () {
        var key = t.getAttribute("data-tab");
        tabs.forEach(function (x) { x.setAttribute("aria-selected", x === t ? "true" : "false"); });
        $all(".install-panel").forEach(function (p) {
          p.classList.toggle("active", p.getAttribute("data-panel") === key);
        });
      });
    });
  }

  // ---- copy ----
  function initCopy() {
    $all(".copy").forEach(function (b) {
      b.addEventListener("click", function () {
        var text = b.getAttribute("data-copy") || "";
        var done = function () {
          var o = b.textContent;
          b.textContent = b.getAttribute("data-i18n") === "copy.label" ? "✓" : "✓";
          b.classList.add("copied");
          setTimeout(function () { b.textContent = o; b.classList.remove("copied"); }, 1400);
        };
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(done, done);
        } else { done(); }
      });
    });
  }

  // ---- mobile menu ----
  function initMobileMenu() {
    var btn = $("#menuBtn");
    var panel = document.createElement("div");
    panel.className = "mobile-menu";
    panel.id = "mobileMenu";
    panel.setAttribute("hidden", "");
    var links = $("#navLinks");
    if (links) panel.innerHTML = links.innerHTML;
    document.body.appendChild(panel);
    function close() {
      panel.classList.remove("open");
      btn.setAttribute("aria-expanded", "false");
      document.body.style.overflow = "";
      setTimeout(function () { panel.setAttribute("hidden", ""); }, 450);
    }
    function open() {
      panel.removeAttribute("hidden");
      requestAnimationFrame(function () { panel.classList.add("open"); });
      btn.setAttribute("aria-expanded", "true");
      document.body.style.overflow = "hidden";
    }
    if (btn) btn.addEventListener("click", function () {
      if (panel.classList.contains("open")) close(); else open();
    });
    $all("a", panel).forEach(function (a) { a.addEventListener("click", close); });
  }

  initTheme();
  initLang();
  initReveal();
  initTabs();
  initCopy();
  initMobileMenu();
})();