# FloatLeaf 开发交接

## 正在交付：Windows 截图 / 录屏保护（2026-10-22）

本机 main 已快进到另一端 `227f275`，工作区原先干净，v0.2.6 菜单栏与自动发布更新保留。新增默认关闭的阅读窗口保护选项，存于现有 SQLite 设置；Windows 原生 WDA_EXCLUDEFROMCAPTURE 设置/读回失败走隐藏与错误提示，日志含期望值/实际 affinity/系统错误。仅保护 overlay，设置页仍可截图，macOS 不显示开关。DXGI DISPLAY_ONLY 需要自有交换链，当前 WebView2 路线未实现该标志，详情及限制见 `docs/CAPTURE_PROTECTION.md`。

本机 `pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm test:rust`（29 项）、`pnpm test:web`（13 项）、`pnpm format:check`、`pnpm tauri build --debug --no-bundle` 与 debug app 构建已通过。实际启动隔离书库的 macOS app，WKWebView 确认不显示 Windows 开关，原生自动阅读五项回归通过；结果在 `.local/capture-protection/mac-probe/`，测试后正常退出。没有用户书库或系统权限变更。版本准备为 0.2.7，当前公开下载仍为 0.2.6。Windows 普通包 GDI/CAPTUREBLT 捕获正反对照已接入现有 CI，待运行；完整实机矩阵不随之勾选。下一步：提交并运行 Windows 原生捕获验证，补齐证据与交付记录。

更新日期：2026-10-22。菜单栏单色素材已接入，本机工具链、检查和调试 app 构建/启动已完成。v0.2.6 两端试用版已公开发布，标签 `2d24437`；双平台 CI、Windows 53 项原生检查和六附件公开下载核验通过。README 开头已加入 LINUX DO 社区致意，下载入口更新为 v0.2.6。菜单栏主题与完整系统体验仍待实机反馈。

此前发布状态（2026-10-18）：已公开发布 Windows x64 / macOS Apple Silicon v0.2.5 紧急隐藏修复试用版，标签 `5c3c93a`；六附件经未登录公开下载逐字节/哈希核验。最终源码 CI 和公开 Windows EXE 的 53 项原生回归通过。当时已明确路径恢复用户刚授权的调试 app（PID 27605）与原书库/Alt+D，权限 true；已授权 app 的后台显示/松键隐藏获用户确认。原始开发子进程、新 macOS 安装包授权、Windows 11 完整实机矩阵仍未完成，持续目标保持进行中。

## macOS 菜单栏素材（2026-10-22）

- 已实现：`assets/floatleaf-menubar-white.svg` 来自用户提供的同名素材，原路径/描边不变，画布从 `0 0 256 256` 裁为 `40 17 161 224`。`src-tauri/icons/menubar-template.png` 为 44 × 44 透明 RGBA，SVG 等比渲染为 26 × 36 后居中；图形像素边界为 x=9..34、y=4..39，保留显示留白。`runtime.rs` 仅在 macOS 使用此 PNG 并启用模板着色；其他平台托盘及 Dock/安装包图标不变。
- 已验证：通过 Sharp 0.35.5 检查 SVG 全部 path 与原文件一致、PNG 无彩色像素且描边未触边，目视核对深浅背景素材预览。证据位于忽略提交的 `.local/menubar-icon/`；预览是素材示意，不是原生菜单栏截图。
- 工具链：首次缺少 Rust 的检查已在用户授权安装后补齐。官方 rustup 1.29.2 安装 Rust 1.99.1、rustfmt、Clippy 到 `~/.cargo` / `~/.rustup`，使用 `--no-modify-path`，未修改全局 shell 配置；nvm 安装并使用 `.nvmrc` 的 Node 22.22.3，pnpm 10.33.4。依赖按锁文件安装。
- 已编译：`pnpm env:check`、`pnpm build`、`pnpm check`（含 Clippy all-targets）、`pnpm format:check`、`pnpm tauri build --debug --no-bundle`、`pnpm tauri build --debug --bundles app` 和 `codesign --verify --deep --strict` 全部通过。
- 已实测：`pnpm desktop:dev --no-watch` 编译并启动后正常停止；再打开明确路径 `src-tauri/target/debug/bundle/macos/FloatLeaf.app`，设置窗口可见，运行 PID 21826。用户开始操作后停止界面接管，保留该调试 app 供使用；其版本仍为 0.2.5，不随后续源码版本变更自动更新。
- 待实测：菜单栏浅/深背景、选中状态和点击菜单，以及新包后台按键/焦点/穿透仍待用户反馈。本轮没有 Windows 11 实机证据，不改变完整系统验收状态。

## v0.2.6 发布结果

版本四处统一为 0.2.6，发布说明 `docs/RELEASE_v0.2.6.md`。Check 工作流在版本标签上构建 macOS 普通 app/DMG 与 Windows 普通 NSIS，保留双平台测试和 Windows 原生安装回归；仅在两端成功后，由单独的 release job 使用 `contents: write` 创建草稿、上传六附件、下载核对哈希/字节再公开为 Pre-release。常规检查 job 仍只有 `contents: read`。`scripts/package-release.py` 核对版本与两端安装包，生成试用 ZIP 和 SHA256SUMS。发布期间不重建用户正在使用的调试 app。

- [发布页](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.6) 于北京时间 2026-10-22 18:41 公开，`draft=false`、`prerelease=true`；标签指向 `2d24437bcb49e997a46dc00ccbfe1584cfdb969d`。
- [标签 CI 39763224518](https://github.com/jlau-ice/floatleaf/actions/runs/39763224518) 两端及 release job 全部成功。`pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm format:check`、Rust（macOS 29 / Windows 30 项）与浏览器（各 12 项）测试通过；普通 DMG/NSIS 构建通过，Windows Server 2025 安装该 EXE 后 53 项原生断言通过。
- 六附件先在 CI 从草稿下载逐字节/哈希核对，公开后本机再以未登录 URL 下载，大小、GitHub SHA-256 与清单一致；Windows ZIP 的 CRC、四项内容及所含 EXE 字节一致。公开 DMG 通过 `hdiutil verify`、只读挂载、0.2.6/arm64 核对、`codesign --verify --deep --strict` 和调试探针排除；未启动发布 app，保留用户正在使用的调试 app。
- 证据在忽略目录 `.local/release-026/`，文件与哈希见 `docs/GITHUB_RELEASE.md`。本次 README 和交付记录作为文档提交推送，不再新增版本；菜单栏实际深浅背景/选中/点击及 Windows 11 等验收保持待办。

## 当前排查：Windows 商家工作台

后续发现并修复紧急隐藏的另一边界：配置的 Pressed 已收到但组合键采样为 false 时，旧共用条件会拒绝紧急隐藏。v0.2.5 源码让该事件直接隐藏并暂停自动阅读，其余显示/翻页规则不变。本机 29 项 Rust 测试、类型/格式/Clippy、Windows 目标 Clippy 与 macOS debug app/no-bundle 构建通过，旧逻辑失败证据保留。修复 `0006b75` 及托盘复用 `hide` 的 `5c3c93a` 已推送。源码 CI `39091188787` / `39091533390` 两端通过：Windows 30 项 / macOS 29 项 Rust、各 12 项浏览器及类型/格式/Clippy，Windows 53 项原生检查通过。公开 v0.2.5 EXE 的 `39092267958` 同样 53 项通过，原生日志与截图核对通过。两端普通包、签名/DMG/PE/探针排除与草稿/公开下载一致性已验证，详见 `docs/GITHUB_RELEASE.md`。旧 v0.2.4 不包含本轮修复。

macOS 用户添加权限后仍提示未授权，实际运行路径为旧发布包 `.local/release-024/mac-trial/FloatLeaf.app`（PID 25942）。正常退出旧包，以明确路径通过 `open` 启动其刚授权的 `src-tauri/target/debug/bundle/macos/FloatLeaf.app`（PID 26295）后权限 true、界面提示消失；用户确认后台显示和松键隐藏正常，五组物理按键日志一致。该调试包保留不重建，独立合成书库在 `.local/emergency-input/fixed/data`。新 v0.2.5 DMG 复制后启动 PID 27467、版本/单实例/退出通过，但权限仍 false；随后明确路径恢复已授权调试包 PID 27605 与原书库/原快捷键，权限 true。临时紧急键 E 未得到有效复测，已恢复默认快捷键；此次不外推到原始开发进程或最终 0.2.5 包。完整记录见 `docs/EMERGENCY_INPUT_FIX.md` 和 `docs/MACOS_BACKGROUND_INPUT.md`。

用户补充：拼多多商家工作台前台无法唤起浮页，关闭其快捷键无效，微信覆盖到前台后恢复。旧普通 v0.2.2 日志中显示/紧急隐藏注册成功、两组翻页冲突，并有多组 1～2 ms 内 Pressed/Released；不能仅归因为快捷键占用或置顶。新增仅配置快捷键与显隐时的前台进程名/PID、完整性 RID、实际按键采样、控制器前后状态和原生窗口属性日志，沿用两份 1 MiB 轮转，不记录业务内容或普通输入。版本 0.2.3 已交付，完整步骤与证据见 `docs/PDD_SHORTCUT_DIAGNOSTICS.md`。

本机类型/rustfmt/Clippy、27 项 Rust 测试、前端构建/格式和 Windows 目标 release Clippy 已通过。Windows 普通 NSIS/复测 ZIP 与 macOS app/DMG 位于 `.local/github-release-0.2.3/`，已公开发布。NSIS 解包、排除调试探针标记和 DMG/ad-hoc 签名校验通过；macOS 从 DMG 复制后启动（PID 18649）、单实例与正常退出通过，之后恢复已获授权的旧独立调试 app，保留用户数据。新包仍显示未授权，不能借用旧包授权结论。源码 `0abb69f`、工作流 `93d7a46` 已推送；CI `39087706786` 两端各 27 项 Rust/11 项浏览器测试及类型/格式/Clippy 通过，Windows 普通安装包构建和 47 项原生断言通过。`39088371240` 另对公开的同一 EXE 实际安装，47 项断言通过；日志确认新字段有效、两进程均为高完整性，尚不覆盖普通浮页对高权限前台应用。公开下载、原生报告与包启动证据均在 `.local/pdd-023/`。

用户回传 v0.2.3 新日志：显示/翻页等五个快捷键均注册成功；普通桌面/浏览器/微信前台共 48 次 Pressed 的主键和组合键采样均 true。PddWorkbench 前台 2 次显示、pddwebworkbench 前台 13 次翻页均收到 Pressed，但主键/组合键/任一成员全 false，显示操作被控制器拒绝。前者完整性 RID 12288，浮页 8192；后者 token 查询错误 5，权限未知。同学已临时以管理员身份启动浮页完成对照并确认恢复，未改永久运行权限。原日志不入 Git/Release；根因证据与限制见诊断文档。

Windows 后续进展：源码补设置页权限提示，仅采样失败且明确查到前台权限更高时出现，正常读取按键后清除；不触发提权或抢焦点。提示已包含在 v0.2.4，旧 v0.2.3 不包含。本机 27 项 Rust / 12 项浏览器测试、类型/格式/Clippy/构建和 Windows 目标 Clippy 通过。两次专项 CI 在建立跨权限场景前失败，未形成有效复现，试验代码已撤回；保留原 47 项测试，详见诊断文档。

提示实现 `75cf762` 已推送，macOS debug/no-bundle 桌面构建通过；双平台 CI `39089252667` 全通过：Windows 28 项 / macOS 27 项 Rust 测试、两端各 12 项浏览器与类型/格式/Clippy；Windows 本机构建的普通包通过 47 项原生回归。

v0.2.4 发布完成：版本四处统一，普通 Windows NSIS 与 macOS app/DMG 构建、Windows 解包和调试探针排除、macOS ad-hoc 签名及 DMG 校验通过。macOS 从只读 DMG 复制后实际启动 PID 22085、版本显示、单实例和正常退出通过；新包仍提示未授权，未测试其物理后台快捷键。`39090055661` 实际下载安装公开 EXE，Windows Server 2025 的 47 项原生断言通过，日志确认 v0.2.4、debug=false；真实截图核对通过。两进程均为高完整性，不能代替跨权限提示与 Windows 11 物理验收。六个附件均通过未登录公开下载逐字节核验，文件 `.local/github-release-0.2.4/`，证据 `.local/release-024/`。下一步由用户设备复测新权限提示及完整键鼠场景；macOS 已恢复原授权调试包和原书库。

## 当前排查：macOS 后台按键

用户确认浮页退出前台后，微信及其他普通应用前台均无法触发显示。同机独立测试书库的真实按键对照证实：Carbon 全局 Pressed/Released 收到，输入监控权限为 false；前台主键采样 true 并显示，后台主键采样 false 并被控制器拒绝。该采样方法自 P0 `e6072f7` 已存在，近期录制按钮未改此路径；用户此前正常的变化点尚无证据，不能归因为某次 UI 改动或系统更新。

已补 macOS 输入监控提示、仅用户点击才发起授权的设置入口、状态刷新及快捷键采样日志。类型/rustfmt/Clippy、27 项 Rust / 11 项浏览器测试、macOS debug/no-bundle 与 app 构建、签名校验通过；最终开发模式提示的 4 项交互回归及前端构建通过。

用户已为 `.local/background-input/FloatLeaf.app` 开启权限，重启 PID 14393 返回 true；用户确认后台显示、先松空格/Option 隐藏、紧急隐藏保持隐藏且全松后恢复均正常，原生日志一致。独立书库证据在 `.local/background-input/after-permission.json`。原始 `pnpm desktop:dev --no-watch` 的二进制单独授权后，重启 PID 15370 仍返回 false（`dev-after-permission.json`）。后续固定同一已授权 app、不重新编译或改授权的对照：直接执行内部程序 PID 21368 返回 false，`open` 正常启动 PID 21613 返回 true，两次均使用独立书库且正常退出。证据在 `launch-direct/` 与 `launch-services/`；本机已明确启动方式影响授权结果，但不推断 TCC 内部归属，不扩大终端/Codex 权限。调试后台快捷键使用 `.app` + `open`；原始开发后台输入仍待解决。已恢复原已授权 app 和原书库；源码、调试包与原始开发进程验证范围见 `docs/MACOS_BACKGROUND_INPUT.md`。

## v0.2.2 发布完成

[v0.2.2](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.2) 已公开发布为 Pre-release，标签指向 `344824f5e0d07232ebd1daa2147ab3b42cfe4057`，录制入口实现为 `4fbc4a0`。版本统一升级为 0.2.2，两端普通安装包不含 diagnostics 探针；源码双平台 CI `39042534515` 各 27 项 Rust / 10 项浏览器测试及类型/格式/Clippy 通过。本轮 `pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm format:check` 与 Windows 目标 release Clippy all-targets 均通过。

Windows 普通 NSIS 和 macOS app/DMG 构建、包版本与诊断入口排除、归档/签名校验通过。Windows 初次 NSIS 步骤因沙箱缓存写权限失败，获准在沙箱外重试后成功。macOS DMG 只读挂载并复制后实际启动（PID 5178）、设置窗口可见、第二实例退出与正常退出通过，证据 `.local/release-022/mac-package-smoke.json`；原开发应用正常退出后已用 `pnpm desktop:dev` 恢复运行。包为 ad-hoc 签名且未公证，Windows 未正式签名。

Windows Server 2025 的 [39044669420](https://github.com/jlau-ice/floatleaf/actions/runs/39044669420) 实际下载并安装公开 v0.2.2 EXE，深色应用偏好下 43 项系统合成按键、原生显隐/焦点/样式、窗口控制及标题栏像素断言通过，日志版本确认为 0.2.2，真实截图核对通过。原始报告、截图和日志在 `.local/release-022/windows-trial/`，运行元数据 `trial-ci.json`；工作流保持 `contents: read`。此项不代表 Windows 11 新录制入口或物理键鼠已验收。

DMG、Windows EXE、试用 ZIP、独立章节示例、复测说明和 SHA256SUMS 共六个附件已从未登录公开 URL 下载，与本地逐字节及 SHA-256 一致，报告 `.local/release-022/download/verification.json`。构建与检查证据在 `.local/release-022/`，发布文件在 `.local/github-release-0.2.2/`；附件无真实小说或用户数据。哈希与完整发布记录见 `docs/GITHUB_RELEASE.md`。

## 当前源码：快捷键录制入口

用户反馈旧只读输入框看不出可以录制。已改为每项明确显示「录制快捷键」的按钮；点击后进入高亮录制态并主动聚焦（兼容 WKWebView 按钮点击焦点行为），单按 Esc、Tab 或失焦取消，录好后显示「尚未保存」，保存成功或注册失败在按钮附近给出反馈。Rust 注册/冲突/存储路径未修改。

环境检查、前端构建、类型/rustfmt/Clippy、Prettier 通过；10 项浏览器测试通过，其中新增 3 项录制交互测试。为保留用户已启动的 1420 桌面开发服务，测试使用临时 1422 端口配置，位于 `.local/shortcut-ux/`。macOS 独立标识的 debug 桌面构建及真实 WKWebView 录制/保存通过，执行 `node scripts/native-final-smoke.mjs .local/shortcut-ux/native --shortcuts-only`；验证点击聚焦、录制状态、Tab 导航、Rust 实际注册与 SQLite 保存，之后恢复测试配置。入口和取消状态截图已目视核对，证据为 `native-shortcuts-result.log`、`shortcuts-idle.png` 和 `shortcuts-cancelled.png`。测试应用已正常退出，用户原开发程序保留。此次改动已包含于 v0.2.2 两端安装包；旧 v0.2.1 / v0.2.0 附件保留不覆盖。

验证边界：WebView 测试通过 DOM 按钮点击和 KeyboardEvent 驱动，实际调用 Rust 与 SQLite；不代表物理快捷键已验收。第一次综合冒烟在阅读窗口处于紧急隐藏状态时等待显示超时，尚未走到录制；针对录制的首次执行又因脚本在 Vue 更新保存按钮前同步点击而中断。已增加等待按钮更新并复跑录制测试通过，先前记录分别保存在 `reading-smoke-interrupted.json`、`synchronous-click-interrupted.json`；不宣称整套阅读综合冒烟本轮通过。

## v0.2.1 Windows 标题栏

用户反馈 Windows 黑色标题栏与浅色页面割裂。已仅对 Windows 设置窗口设置浅色主题、暖白标题栏和深绿文字，保留原生窗口控件及拖动/缩放行为；macOS 与悬浮阅读控制不变。类型、Rust 格式、Clippy、Windows 目标 Clippy、Windows 普通 NSIS 与 macOS debug 构建通过。源码 CI `39037365528` 两端各 27 项 Rust / 7 项浏览器测试通过，但 Windows 原生测试因脚本窗口等待失败，整体状态仍为失败。

修正测试脚本后，`39040506097` 在 Windows Server 2025 实际安装公开候选 EXE，43 项原生显隐/焦点/窗口控制及标题栏像素检查全部通过；深色应用偏好下的真实截图目视核对通过。应用实现为 `249c0c5`，后续脚本调整不改变安装器。原始报告、截图及日志在 `.local/titlebar-021/final-test/`，完整过程见 `docs/WINDOWS_TITLEBAR.md`。[v0.2.1](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.1) 已公开发布为 Pre-release，标签指向 `140c974`；安装器、试用 ZIP、复测说明、原生截图与 SHA256SUMS 共五个附件通过未登录公开 URL 重新下载，逐字节/哈希一致。证据在 `.local/titlebar-021/download/verification.json`；macOS 沿用 v0.2.0。工作流保留只读权限，先用公开候选包验收，最终版安装器与候选包完全一致。

## v0.2.0 章节与 EPUB

用户确认先支持 EPUB / TXT，章节选择是必要功能。已接入 EPUB 2/3 导航与阅读顺序解析、TXT 标题识别、SQLite 章节表和旧书库升级；新增可搜索目录、当前章节和前后章跳转，隐藏时跳章不改变窗口显隐。正文以纯文本导入，原书图片和版式不保留；加密正文拒绝导入。27 项 Rust、7 项浏览器测试、类型/格式/Clippy 和真实 macOS 桌面导入/搜索/跳章/重启恢复已通过；两端普通安装包、Windows 目标 Clippy、macOS 签名/DMG、Windows 解包与版本校验已通过，发布附件在 `.local/github-release-0.2.0/`。实现提交 `a566296` 已推送，CI `39035419034` 两端全部通过。官方 EPUB 样例额外导入与选章通过（141 / 314 个目录项），测试应用均已退出。证据与边界见 `docs/CHAPTERS_EPUB.md`。

发布：[v0.2.0](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.0)，标签指向 `5685863`。DMG、Windows EXE、试用 ZIP、独立章节示例、复测说明和 SHA256SUMS 共六个附件通过未登录公开 URL 下载，并与本地逐字节及 SHA-256 核对一致；副本与核验报告在 `.local/chapters-020/download/`。试用包只附原创合成文本，无真实小说或用户数据。

Windows Server 2025 CI 实际安装本轮构建的普通 v0.2.0 NSIS 包，注册表版本为 0.2.0，35 项系统合成按键与原生 HWND 显隐/焦点/样式断言全部通过；旧 v0.1.0 包仍复现隐藏失败。原始附件 `.local/chapters-020/windows-native-evidence/`，完整日志 `ci-windows.log`，运行结果 `ci-result.json`。这不等于 Windows 11 物理键鼠、真实点击穿透或 EPUB 目录界面已验收；后者仍由用户设备补测。

## v0.1.2 自动翻页控制

修复按住模式下无法开关自动翻页，将 2～60 秒间隔滑块移到阅读按钮旁，隐藏时开启只改变阅读意图，不推进计时或进度。19 项 Rust / 5 项浏览器测试和真实 macOS WebView/IPC/SQLite 回归通过；正常退出重启后间隔 3 秒和锚点 55 恢复。两端安装包、Windows 目标 Clippy、macOS ad-hoc 签名/DMG 校验通过，见 `docs/AUTO_READING_FIX.md`。提交 `edb350a` 已推送；CI `39019344546` 的 macOS 检查及 Windows 功能验证均通过，Windows 最后附件上传失败的原因与修正见下。

Windows Server 2025 CI 已安装 v0.1.2 普通 NSIS 包，35 项原生断言全部通过：六种松键顺序、紧急隐藏、旧组合键保护、重新触发和前台焦点保持。使用系统合成输入及 HWND 查询，不代表 Windows 11 物理键鼠/点击穿透验收。此前新包 EXE 路径问题经 `Join-Path` 修正已消失。CI 唯一失败为日志与 JSON 分处 C:/D: 盘，附件上传无法计算共同目录；已拆成两个附件上传，后续 CI `39020157470` 两端及附件上传全部通过。报告从完整运行日志提取到 `.local/auto-012/windows-fixed-from-ci.json`。

发布：[v0.1.2](https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.2)，标签指向 `2d17dec`，应用实现为 `edb350a`。DMG、Windows EXE、试用 ZIP、复测说明、SHA256SUMS 五个附件公开下载核验通过，文件副本在 `.local/auto-012/download/`。

## v0.1.1 Windows 修复与日志（历史记录）

`platform::hide` 在 Windows 调用原生 `SW_HIDE`，覆盖普通、紧急及错误路径；macOS 原隐藏实现保留。日志按窗口实际状态记录显隐结果，只采集应用快捷键和窗口诊断，不采集小说正文/书名/普通输入；设置页显示版本和路径，可打开日志文件夹。两个日志文件共约 2 MiB 上限，轮转测试已加入。

本机类型、rustfmt、Clippy、18 项 Rust / 5 项浏览器测试、前端构建和 Windows 普通 release/NSIS 构建通过。新增 Windows 安装包回归脚本及 CI 步骤，对旧包复现和新包实际 HWND 显隐进行检查；最终云端结果见上方 v0.1.2 记录。复测说明见 `docs/RELEASE_v0.1.1.md`，根因/验证见 `docs/WINDOWS_HIDE_FIX.md`。下方 v0.1.0 阶段记录作为历史保留。

补充：Windows 目标 Clippy 通过；macOS 独立测试标识的 debug 桌面构建、100 次原生显隐、日志入口 WKWebView/IPC 实测和截图检查通过，日志中期望/实际可见性一致。首次显隐测试因前台进程变化中断，随后完整复跑通过。测试应用正常退出，原有应用和书库保留。源码修复提交 `683fcfd` 已推送；CI 运行 `39013867420` 已结束：macOS 通过；Windows 旧包复现通过，新包测试因安装目录未找到 EXE 而中断。新安装器及复测 ZIP 在 `.local/github-release-0.1.1/`，不覆盖 v0.1.0。

发布结果：`https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.1`，标签指向 `0b71557`。四个附件（Windows 安装器、试用 ZIP、复测说明、SHA256SUMS）均经公开链接重新下载，逐字节/哈希比对通过。macOS 安装包仍为 v0.1.0。该轮 Windows 旧包复现通过，新包安装回归中断；后续 v0.1.2 已通过原生窗口回归。

## 仓库与环境

目录 `/Users/ice/jlau-ice/floatleaf`，当前分支 `main`，跟踪 `origin/main`。按用户授权已创建公开仓库 https://github.com/jlau-ice/floatleaf ，推送完整阶段历史、README 和 MIT 协议，发布 https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.0 试用版。六个附件下载哈希一致，公开下载链接 HTTP 200。发布证据见 `docs/GITHUB_RELEASE.md`。

Node 22.22.3（nvm）、pnpm 10.33.4；Rust 1.99.1（rustup），rustfmt 与 Clippy。macOS 27.0.1 (26A434)，Apple Silicon arm64。已核对项目工具链；不改全局 shell 配置。

## 首版实现与历史验证

- 隐藏预创建 overlay，透明无框无阴影、置顶、不可聚焦、默认鼠标穿透。
- Rust `controller.rs` 是唯一显隐权威。按住/切换/固定模式、独立位置锁定、紧急隐藏、旧事件保护、编辑布局拖动。
- 默认 Ctrl+Alt+Space，紧急隐藏 Ctrl+Alt+Escape。插件注册加原生组合键成员检查，覆盖主键/修饰键释放顺序，按住检查线程可即时唤醒。
- macOS 不激活显示调用、Workspace/锁屏通知；Windows 不激活显示调用、WTS 和电源广播。Windows 代码已交叉编译，并在 Windows Server 2025 CI 通过完整检查、测试和 debug 桌面构建；Windows 11 实机安装和交互仍待反馈。
- 托盘/菜单栏入口：设置、找回位置、紧急隐藏、退出；关闭设置只隐藏。
- 自定义五组快捷键；编辑态右下角调整宽度和完整行数；SQLite 保存逻辑位置；工作区约束、显示器变化和缩放重排、原屏消失时主屏恢复。
- TXT 文件选择、UTF-8/GBK 预览与确认导入、书架与移除。直接 SQLite 保存正文、每书 UTF-16 锚点和设置，无 ORM。
- WebView 实际排版分页、完整行数、字体与透明度设置；手动翻页、自动计时、隐藏暂停、末尾停止、重排保持锚点。
- 设置和阅读界面、真实状态事件同步、注册冲突错误显示。
- 单实例运行、Dock 重新打开设置；自有图标和两端安装配置。macOS 普通 release 试用 DMG 已产出。
- debug 或显式 `diagnostics` feature 可开启文件探针，普通 release 不包含此入口。原生输入测试窗口和冒烟脚本已纳入仓库，本地产物在 `.local/`，不会提交。

## v0.1.0 验证记录

| 检查 | 结果 |
| --- | --- |
| pnpm env:check | 通过，rustup 固定工具链 |
| pnpm build | 类型检查与前端生产构建通过 |
| pnpm check | 类型、rustfmt、Clippy all-targets 通过，警告视为错误 |
| pnpm test:rust | 17 项状态机、阅读计时、存储、布局、快捷键和解码测试通过 |
| pnpm test:web / pnpm format:check | 5 项通过：3 项 DOM 分页、2 项导入交互；格式通过 |
| pnpm tauri build --debug --no-bundle | 本机 Mach-O 构建通过 |
| macOS 原生运行 | 沙箱外实际启动成功，快捷键注册无错误 |
| 原生窗口探针 | 100 次显示/隐藏通过；最新运行前台始终为独立输入测试进程 42837，overlay 非 key window |
| 原生窗口属性 | opaque=false、level=5；阅读态鼠标穿透 true，编辑态 false，完成编辑恢复 true |
| 紧急隐藏 | 探针验证固定模式不能绕过隐藏锁；旧键事件保护另由状态机测试覆盖 |
| 界面检查 | 本应用 WKWebView 截图：设置页及编辑态阅读文本均已目视检查 |
| 最终原生回归 | P1 阅读冒烟再次通过；移除其他书不中断阅读，外观保存不重置计时，快捷键页面保存与恢复通过 |
| 普通 macOS 安装包 | app/DMG 构建、ad-hoc 签名校验、从 DMG 复制后启动、重复启动与正常退出通过 |
| 性能采样 | 隐藏 300 秒 CPU 0.607%；固定静态 60 秒 0.750%；自动阅读 600 秒 3.168%，详细范围见 PERFORMANCE.md |
| 文档 | 使用、安装、人工验收步骤已补齐；Word 计划书更新并检查 6 页渲染 |
| Windows x64 交叉构建 | release、NSIS 生成、Windows 目标 Clippy all-targets 通过；PE/归档内容核验通过，未实机运行 |
| 试用说明与转发包 | 三页 Word 全页渲染检查通过；Windows ZIP 含安装器、说明、合成示例、反馈表和哈希清单 |
| GitHub 公开发布 | main 已推送；v0.1.0 Pre-release 已发布，六个附件哈希核验和公开下载链接通过 |
| GitHub 双平台 CI | 提交 `c7ee2c8`：macOS 26.6.2 arm64、Windows Server 2025 各 17 项 Rust / 5 项浏览器测试、类型、格式、Clippy 与 debug 桌面构建全部通过 |

首轮 Windows CI 的 Prettier 检查报告全体 16 个文本目标格式不符，新增 `.gitattributes` 统一 LF。本机格式检查、`core.autocrlf=true` 模拟检出和两端 CI 复跑均通过，运行编号 `39011651513`。完整命令、环境与证据见 `docs/GITHUB_RELEASE.md`。云端测试不包含实际安装或系统键鼠交互。

P1 原生 IPC 冒烟通过：导入两本合成文本、分页/重排、隐藏暂停、手动重置、末尾停止、去重、独立进度和移走源文件后的重启恢复。修复重复分页回报造成的计时干扰，增加稳定 revision 回归检查。SQLite 使用 DELETE journal，退出后单文件备份。证据见 `docs/P1_READ_VERIFICATION.md`，本地数据 `.local/p1-final/`。

P0 证据：`docs/P0_NATIVE_VERIFICATION.md`。本地运行数据 `.local/p0/native-smoke.json`，截图 `.local/p0/main.png`、`overlay.png`（忽略提交）。原生测试不等于物理键鼠测试。

P3 证据与包哈希见 `docs/P3_DELIVERY_VERIFICATION.md`。试用包：`.local/release/FloatLeaf_0.1.0_aarch64.dmg`（2,986,687 字节），SHA-256 `89da12640a036d33e42c744d33f5d0af54887906053432c2a104d280fb7f0ddf`。最终原生回归数据在 `.local/p3-smoke/`，包启动证据在 `.local/release/package-smoke.json`。上述自动回归测试进程已退出。

Windows 新产物见 `.local/release/FloatLeaf_0.1.0_windows-x64_trial.zip`（2,395,100 字节），安装器单独保存在同目录。构建命令、工具版本、文件哈希、已知构建警告与核验范围见 `docs/WINDOWS_BUILD.md`。业务代码与锁文件未改变。macOS 手工验收已准备 `.local/manual-macos/` 示例及输入窗口，结果待用户反馈。

## 待验证与外部条件

- P2 程序路径已验证，见 `docs/P2_LAYOUT_VERIFICATION.md`。macOS Carbon 允许共享快捷键；独占冲突检查通过，不能保证检测所有其他应用拦截。
- 本机测试进程的辅助功能自动化和屏幕录制权限均未授予。真实键鼠释放顺序/输入连续性/点击穿透待手工或获授权的自动化验证。未修改系统权限。
- 锁屏、真实休眠、全屏/多桌面、外接屏未实测。Windows 已有交叉构建及 Windows CI 本机构建、单元/浏览器测试证据，仍无 Windows 11 实机安装和交互验收。TASKS 的双平台项目不能勾选。
- CPU/RSS 已测隐藏空闲、固定静态与自动阅读；固定静态不是物理长按。端到端 P95 未测，不能宣称响应目标通过。采样使用优化诊断版，最终普通 DMG 未直接采样。
- macOS 包为 ad-hoc 本地试用包，Windows NSIS 未签名；正式签名、公证、干净设备安装和朋友试用结果待完成。

## 下一步

下一步完成权限提示的 Windows 条件测试、CI 和交付。同权限操作方案由用户实测通过，不默认要求永久管理员运行；继续复测真实按键、焦点、穿透及章节兼容性。macOS 新包需用户核对实际 `.app` 授权，原始开发启动权限仍待闭环。未完成项继续保留在 TASKS.md 和验收矩阵；正式签名、公证和付费服务仍需另行授权。

保持实现直接、简单：rusqlite bundled，无 ORM、额外仓储框架或独立小说文件。不得将当前本机程序验证外推为双平台通过。发布说明见 `docs/RELEASE_v0.1.0.md`，附件在 `.local/github-release/`，包含两端安装器、Windows 试用 ZIP、Word 说明、MIT 协议和 SHA-256 清单，不进入 Git。历史提交已检查，未包含依赖目录、编译缓存、真实小说、用户书库或已知凭据模式。发布标签指向 `149e9ff`；后续换行规则与交接记录修正不改变安装包，阶段提交见 `git log`。
```bash
nvm use
pnpm desktop:dev
# 直接用 cargo 前 source "$HOME/.cargo/env"；pnpm 脚本已为子进程补 PATH
```

原生验证脚本运行方法、精确限制及实机复现顺序见 `docs/P0_NATIVE_VERIFICATION.md`。临时测试进程应只按本任务 PID 退出，不结束用户其他应用。
