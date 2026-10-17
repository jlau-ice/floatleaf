# FloatLeaf 开发交接

更新日期：2026-10-17。当前阶段：已公开发布 Windows x64 / macOS Apple Silicon v0.2.2 试用版，加入明确的快捷键录制入口与保存反馈，包含此前 EPUB/TXT 章节、自动翻页控制、Windows 标题栏和隐藏修复。六个发布附件已公开下载并与本地逐字节/哈希核验。持续目标尚未完成，用户设备的章节兼容性、物理键鼠与完整系统验收仍待反馈。

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

下一步由用户转发 **v0.2.2** Windows 安装器或 ZIP，macOS 用户安装同版本 DMG；先退出旧版再安装，从“日志位置”确认版本。先复测「录制快捷键」入口、取消、保存反馈及新组合键实际触发，再查看 Windows 系统浅色/深色下的标题栏及按钮悬停，复测缩放、拖动和贴靠。导入手头的 EPUB/TXT，检查目录、搜索和选章；隐藏时跳章、调整字号及重启后检查位置。再复测自动翻页间隔、不同松键顺序和紧急隐藏，异常时回传日志与操作步骤。用户继续 macOS 手工验收。收到反馈后补齐 `docs/MANUAL_VALIDATION.md` 剩余验收。正式签名、公证和付费服务仍需另行授权。

保持实现直接、简单：rusqlite bundled，无 ORM、额外仓储框架或独立小说文件。不得将当前本机程序验证外推为双平台通过。发布说明见 `docs/RELEASE_v0.1.0.md`，附件在 `.local/github-release/`，包含两端安装器、Windows 试用 ZIP、Word 说明、MIT 协议和 SHA-256 清单，不进入 Git。历史提交已检查，未包含依赖目录、编译缓存、真实小说、用户书库或已知凭据模式。发布标签指向 `149e9ff`；后续换行规则与交接记录修正不改变安装包，阶段提交见 `git log`。
```bash
nvm use
pnpm desktop:dev
# 直接用 cargo 前 source "$HOME/.cargo/env"；pnpm 脚本已为子进程补 PATH
```

原生验证脚本运行方法、精确限制及实机复现顺序见 `docs/P0_NATIVE_VERIFICATION.md`。临时测试进程应只按本任务 PID 退出，不结束用户其他应用。
