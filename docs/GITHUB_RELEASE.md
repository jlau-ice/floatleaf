# GitHub 公开发布记录

## v0.2.4

[v0.2.4 权限提示试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.4) 已公开发布为 Pre-release，标签指向 `19e617ebaa3d8b30c15825bdc7725649dc48ac19`，权限提示实现为 `75cf762`。Windows 明确检测到前台进程权限更高且按键采样失败时，设置页显示原因与处理方法；恢复正常读取后清除。不自动提权、不改变显隐状态机。用户已确认 v0.2.3 同权限运行可恢复拼多多前台显示、松键隐藏和翻页，普通浮页跨权限读取限制仍保留。

- [双平台源码 CI 39089252667](https://github.com/jlau-ice/floatleaf/actions/runs/39089252667)：Windows 28 项 / macOS 27 项 Rust 测试，两端各 12 项浏览器测试，环境/类型/rustfmt/Clippy/Prettier 与平台构建通过；Windows 本机构建的普通包通过 47 项原生断言。
- [公开 EXE 回归 39090055661](https://github.com/jlau-ice/floatleaf/actions/runs/39090055661)：Windows Server 2025 实际下载安装本次公开 EXE，47 项系统合成按键、显隐/焦点/窗口样式、原生控件、标题栏像素及诊断字段断言通过。日志确认 v0.2.4、debug=false；浮页 PID 9232 与前台 python.exe PID 2916 均为高完整性 12288，不代表普通浮页对高权限程序的新提示已获原生验收。截图目视核对通过。
- macOS DMG 只读挂载复制后实际启动 PID 22085，设置页版本、单实例、正常退出通过；ad-hoc 签名、DMG 完整性与调试探针排除通过。新包仍提示输入监控未授权，未借用旧包的后台按键结论。测试后恢复用户原已授权调试 app 与书库。
- 版本文件变化外无业务变更；新版本的两端普通包均构建成功。macOS 同一已授权调试二进制的直接子进程 / LaunchServices 启动对照见 MACOS_BACKGROUND_INPUT.md，原始开发后台输入限制仍在。

六个附件先从草稿下载，再从未登录公开 URL 下载，均与本地逐字节和 SHA-256 一致。初次草稿创建因 GitHub 不接受缩写提交号返回 422，改用完整提交号后成功，未覆盖旧版。文件在 `.local/github-release-0.2.4/`，构建、包内容、Mac 启动、CI 与公开下载证据在 `.local/release-024/`。Windows ZIP 包含安装器、复测说明、MIT 协议与安装器校验值；没有真实小说、用户日志、数据库或凭据。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| FloatLeaf_0.2.4_aarch64.dmg | 3,755,622 | `964e059873f61d907048187c70a6ec36855c7b68c1be5650981dd57987f3a1ee` |
| FloatLeaf_0.2.4_windows-x64_trial.zip | 2,978,604 | `438ce02e4b81b96371dfc03f46ad76c0192d60197f3e6a71a35c9282963e072c` |
| FloatLeaf_0.2.4_x64-setup.exe | 2,996,318 | `7f4f142184f001c4c1f8f715108ef2436c15843255c9c7a5c46410116376303c` |
| LICENSE.txt | 1,065 | `f527b3b2027179390d943c50ca81edc52a78b8258ebc007288598825a207c123` |
| Retest-Guide.txt | 2,589 | `97c404628571f3817e8ac09d215f19cc0335e1187b8e66a638d0d40867d2cbc6` |
| SHA256SUMS.txt | 455 | `99e4fa2e507bf64240b4e171f451ca77cf59c2e8d699de69a27e4ea2a513421c` |

两端安装器未正式签名，macOS 未公证；Windows 11 新提示、物理释放顺序/紧急隐藏/焦点/点击穿透及多屏休眠仍待复测，持续目标未完成。

## v0.2.3

[v0.2.3 快捷键诊断试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.3) 已公开发布，标签指向应用源码 `0abb69f25ad958e1a2a0b113dcaa0f56f63eaf61`。新增 Windows 配置按键/前台进程完整性/原生窗口诊断与 macOS 输入监控引导。本版不宣称商家工作台兼容性已修复，原始开发启动的 macOS 权限限制亦未解决。

- [双平台源码 CI 39087706786](https://github.com/jlau-ice/floatleaf/actions/runs/39087706786)：两端各 27 项 Rust、11 项浏览器测试，环境/类型/rustfmt/Clippy/Prettier 通过；macOS debug 构建与 Windows 普通 NSIS 构建/47 项原生回归通过。
- [公开 EXE 回归 39088371240](https://github.com/jlau-ice/floatleaf/actions/runs/39088371240)：实际下载并安装本次公开 EXE，Windows Server 2025 的 47 项系统合成输入、显隐/焦点/窗口样式、标题栏像素及四项诊断字段断言通过。日志确认版本 0.2.3、debug=false，前台 python.exe 与浮页均为高完整性 RID 12288；这不覆盖普通权限浮页面对高权限商家工作台。原生截图目视核对通过。
- macOS DMG 只读挂载复制后实际启动（PID 18649）、设置页 v0.2.3、第二实例退出及正常退出通过。新包仍显示需要输入监控，之后恢复此前用户已授权的独立调试 app，保留原书库与设置；不声称新包后台快捷键已获用户实测。

六个公开附件通过未登录下载地址重新下载，与本地逐字节及 SHA-256 一致，证据 `.local/pdd-023/public-download/verification.json`。试用 ZIP 含 EXE、复测步骤和安装器校验值；MIT 协议另附。普通安装器不含 diagnostics feature 的文件命令入口，无用户日志、小说或数据库。构建/包校验、Mac 启动、源码和公开 EXE 报告在 `.local/pdd-023/`。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| FloatLeaf_0.2.3_aarch64.dmg | 3,754,763 | `cd806bf3019ff4cd73a6f258577c4b0d3591b6a3d81c24b791ccfa825ef8d0ed` |
| FloatLeaf_0.2.3_x64-setup.exe | 2,994,136 | `a9cfe4a321063ccad3744227290a0d3de6ac22002848612a59d461ba413572ee` |
| FloatLeaf_0.2.3_windows-x64_trial.zip | 2,974,729 | `c098e7815defd7f4c77ed53a096c5683b67c55fcb1562800136bdc221c185b11` |
| Retest-Guide.txt | 1,143 | `2c95d936b43d3a31b14800057f41ba9eaa6b08e5bf7d24948682a08181d0b1f5` |
| LICENSE.txt | 1,065 | `f527b3b2027179390d943c50ca81edc52a78b8258ebc007288598825a207c123` |
| SHA256SUMS.txt | 455 | `5ea96f1fc9953c0356f28bf006f55263372df61125350f89540eb87a7ae14117` |

安装包未正式签名，macOS ad-hoc 且未公证。新收到的 Windows 11 同学日志确认故障发生于商家工作台前台的按键采样，详情见 PDD_SHORTCUT_DIAGNOSTICS.md；后续用户同权限对照确认恢复，结论见上方 v0.2.4；不以普通 CI 通过代替用户场景。

## v0.2.2

[v0.2.2 两端试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.2) 已公开发布为 Pre-release，标签指向 `344824f5e0d07232ebd1daa2147ab3b42cfe4057`，录制入口实现提交为 `4fbc4a069226899aa19c1b0f7d098582d6c39e3d`。Windows x64 / macOS Apple Silicon 版本统一为 0.2.2，新增明确的「录制快捷键」、正在录制、尚未保存与保存结果提示，包含此前标题栏、章节、隐藏及自动翻页修复。

普通 app/DMG、NSIS 构建成功，版本与诊断探针排除、Windows 安装归档/ZIP CRC、macOS ad-hoc 签名及 DMG 校验通过。本机执行 `pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm format:check` 和 Windows 目标 release Clippy all-targets 均通过。Windows 首次 NSIS 步骤因沙箱禁止写缓存失败，在沙箱外重试成功，记录 `windows-build.log` / `windows-build-retry.log`；业务代码未因此变更。

- [源码 CI 39042534515](https://github.com/jlau-ice/floatleaf/actions/runs/39042534515)：两端各 27 项 Rust / 10 项浏览器测试与环境/类型/格式/Clippy 通过；macOS debug 构建和 Windows 普通 NSIS 构建及 43 项原生回归均通过。该轮为实现提交，包版本尚为 0.2.1。
- [v0.2.2 安装包回归 39044669420](https://github.com/jlau-ice/floatleaf/actions/runs/39044669420)：Windows Server 2025，深色应用偏好下实际下载并安装本次公开 EXE，43 项系统合成按键、原生显隐/焦点/样式、设置窗口控制与标题栏像素检查全部通过；启动日志版本确认为 0.2.2，真实截图目视核对通过。报告和截图在 `.local/release-022/windows-trial/windows-trial-evidence/`，日志在 `windows-trial-logs/`，工作流保留 `contents: read`。
- macOS 普通 DMG 只读挂载并复制应用后，实际启动（PID 5178）、设置窗口可见、第二实例退出、正常退出通过，证据 `.local/release-022/mac-package-smoke.json`。随后恢复用户的桌面开发程序。录制功能的真实 WKWebView / Rust / SQLite 验证见 `HANDOFF.md`，由 DOM 事件驱动，不代表物理键盘验证。

六个附件从未登录公开 URL 重新下载，逐字节及 SHA-256 与本地产物一致。发布文件在 `.local/github-release-0.2.2/`，核验报告 `.local/release-022/download/verification.json`。Windows ZIP 包含安装器、MIT 协议、说明与原创合成章节示例，无真实小说、用户数据库或凭据。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| Chapter-Samples.zip | 2,797 | `9610a3c801556b679a140abeabbbaa3367b10da3cda309e3e2434f0e683b45f3` |
| FloatLeaf_0.2.2_aarch64.dmg | 3,746,639 | `d1844005152cdae4c8671c1a65f1c3a07db5d47180394fc1d33ce9961cf94730` |
| FloatLeaf_0.2.2_windows-x64_trial.zip | 2,975,900 | `6b57ec38ac44e06eec1a1da7e370076bfd8453b998cf31a9f2cdcfb9e13e71ac` |
| FloatLeaf_0.2.2_x64-setup.exe | 2,987,007 | `7c91059e1d0d924d8ffb9c3a74bfb2c9f5119495859d8bb25370f4c73350c461` |
| Retest-Guide.txt | 1,281 | `cf5f3c6c508bf4041d105f788bd4ba9d29c5dfc6eaaa1ae9d87c6fbe122e20b1` |
| SHA256SUMS.txt | 463 | `b2a20be45e219f73e6392044f7dca36e5ea02b72724a94f35eade8d7568d2d35` |

Windows 11 用户设备的新录制入口、物理键鼠、不同缩放、多屏与休眠仍待验收。安装包仍未使用正式发布证书；macOS 为 ad-hoc 签名且未公证。本次发布不代表持续目标和双平台完整验收完成。

## v0.2.1

[v0.2.1 Windows 试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.1) 已公开发布，标签指向 `140c974f5ee8193af1246447f1527f5b93983c77`，应用实现为 `249c0c5`。设置窗口使用与页面一致的暖白原生标题栏和深绿文字，系统窗口按钮保持原生；macOS 下载沿用 v0.2.0。

五个附件（Windows 安装器、试用 ZIP、复测说明、原生截图、SHA256SUMS）已从未登录公开 URL 重新下载，并逐字节及 SHA-256 核对一致。包哈希见 [WINDOWS_TITLEBAR.md](WINDOWS_TITLEBAR.md)，证据在 `.local/titlebar-021/download/verification.json`。附件无真实小说或用户数据，安装器仍未正式签名。

源码检查 `39037365528` 的两端各 27 项 Rust / 7 项浏览器测试及类型/格式/Clippy 通过；macOS job 成功，Windows 普通 NSIS 构建通过，但原生测试脚本等待窗口失败，整轮状态如实保留为失败。修正脚本后，[39040506097](https://github.com/jlau-ice/floatleaf/actions/runs/39040506097) 成功：Windows Server 2025、100% 缩放、深色应用偏好下实际安装公开候选 EXE，43 项显隐/焦点/窗口控制/标题栏像素检查全部通过，真实截图核对通过。最终 v0.2.1 与候选版 `v0.2.1-rc.1` 使用同一 EXE，CI 始终保持 `contents: read`。

云端合成输入与截图验证不等于 Windows 11 物理键鼠、按钮悬停、不同缩放和贴靠已验收；这些项目仍等待用户设备反馈。公开候选页已指向完整的 v0.2.1 试用发布。

## v0.2.0

[v0.2.0 试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.0) 已公开发布，标签指向 `56858631992bcb56676aff2effd660c122292e14`，应用实现提交为 `a566296`。新增 EPUB 2/3、TXT 章节识别、目录搜索和选章，原书库自动升级并保留进度。

macOS Apple Silicon DMG、Windows x64 NSIS 安装器、Windows 试用 ZIP、独立章节示例 ZIP、复测说明及 SHA256SUMS 共六个附件，均从未登录的公开下载 URL 下载并与本地逐字节、SHA-256 核对一致。包哈希、构建与原生测试范围见 [CHAPTERS_EPUB.md](CHAPTERS_EPUB.md)，下载证据在 `.local/chapters-020/download/verification.json`。示例均为原创合成文本，安装包未正式签名/公证，Windows 11 与用户书籍兼容性仍待复测。

实现提交 `a566296` 的 [CI 39035419034](https://github.com/jlau-ice/floatleaf/actions/runs/39035419034) 两端成功，各 27 项 Rust / 7 项浏览器测试、环境/类型/格式/Clippy 通过。macOS debug 桌面构建通过；Windows Server 2025 本机构建并实际安装普通 v0.2.0 NSIS 包，35 项系统合成按键与原生窗口显隐/焦点/样式断言通过，旧包隐藏失败复现通过。报告与日志附件上传成功，原始报告下载在 `.local/chapters-020/windows-native-evidence/`。该结果不代表 Windows 11 物理键鼠或目录界面已完成验收。

## v0.1.2

[v0.1.2 试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.2) 已公开发布，标签指向 `2d17dec299126802606ee41173cdcb6a3bd53b7c`，应用实现提交为 `edb350a`。包含按住模式自动翻页开关、可调间隔、Windows 隐藏修复和日志入口。macOS Apple Silicon DMG、Windows x64 NSIS 安装器、Windows 试用 ZIP、复测说明和 SHA256SUMS 共五个附件已从公开下载 URL 重新下载，逐字节与本地构建一致。

包大小和 SHA-256 见 [AUTO_READING_FIX.md](AUTO_READING_FIX.md)。发布文件在 `.local/github-release-0.1.2/`，下载副本在 `.local/auto-012/download/`；不纳入 Git。包仍未正式签名或公证，用户设备复测待反馈。

`edb350a` 的 [CI](https://github.com/jlau-ice/floatleaf/actions/runs/39019344546) 两端各 19 项 Rust / 5 项浏览器测试及类型/格式/Clippy 通过，macOS job 成功。Windows 普通 NSIS 安装与 35 项系统输入/原生窗口断言通过；仅最后附件上传因 C:/D: 跨盘共同根目录失败，已拆分上传。总状态如实保留为失败，不影响已记录的功能测试结果，修正提交 `bd9999b` 的运行 `39020157470` 两端与附件上传均成功。

## 历史版本

后续修复：已发布 [v0.1.1 Windows 隐藏修复与日志复测版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.1)，标签 `0b71557`，业务修复 `683fcfd`；安装器、ZIP、复测说明、SHA256SUMS 四个附件公开下载与本地逐字节一致。macOS 安装包保持 v0.1.0。新版本根因和验证见 [WINDOWS_HIDE_FIX.md](WINDOWS_HIDE_FIX.md)，以下保留首次发布记录。

日期：2026-10-17。按用户明确授权创建公开仓库、推送源码，并发布两端试用安装包。

- 仓库：[jlau-ice/floatleaf](https://github.com/jlau-ice/floatleaf)，默认分支 `main`，协议 MIT。
- 发布：[v0.1.0](https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.0)，已公开，标记为 Pre-release；发布时间为北京时间 2026-10-17 14:51。
- 版本标签指向 `149e9ffa4acfc46d7e41b24671fc231bb778430a`；两份安装包沿用此前核验的业务代码 `6e341c5`，发布准备仅更新说明与协议。
- 安装包的正式签名与公证仍未完成；发布为公开试用版不代表双平台验收完成。

## 上传文件与核验

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| FloatLeaf_0.1.0_aarch64.dmg | 2,986,687 | `89da12640a036d33e42c744d33f5d0af54887906053432c2a104d280fb7f0ddf` |
| FloatLeaf_0.1.0_x64-setup.exe | 2,374,682 | `7fa5165cc42fb3a377d8ad8e3dd7af6d993a40d1589402c382a916e866488cc1` |
| FloatLeaf_0.1.0_windows-x64_trial.zip | 2,395,100 | `7947467140c79cfc84e54d3e24d979639be45a1f45d81af436e93b5748c570ff` |
| FloatLeaf-Trial-Guide.docx | 40,238 | `af2e2ee7f5b66f9de47021114187452bc4f88baeaf8a14797cae3d87c7f3783a` |
| LICENSE.txt | 1,065 | `f527b3b2027179390d943c50ca81edc52a78b8258ebc007288598825a207c123` |
| SHA256SUMS.txt | 465 | `48e5a7218f9e161daea065fafc68a6f8b92c48b57f2816a393531b691f2eb7b3` |

使用 `gh release create --draft --prerelease` 上传，随后 `gh release download` 下载全部六个附件并逐一比较 SHA-256，全部与本地文件一致；再使用 `gh release edit --draft=false --prerelease --latest=false` 发布。`gh release view` 确认 `isDraft=false`、`isPrerelease=true` 与六个附件。

未登录请求访问公开仓库、发布页、DMG、EXE 和试用 ZIP 均返回 HTTP 200。`git ls-remote origin refs/heads/main` 与首次推送提交一致。下载核验明细保存在忽略目录 `.local/github-release-verification.json`，发布附件保存在 `.local/github-release/`。

完整阶段历史与依赖锁文件已推送；安装包通过 Release 分发，未纳入 Git。已检查历史文件范围和已知凭据模式，未提交依赖目录、编译缓存、真实小说、用户数据库或密钥。

## 云端检查

首轮 [Check](https://github.com/jlau-ice/floatleaf/actions/runs/39011286910) 的 Windows runner 通过环境检查、前端构建和 `pnpm check`，随后 Prettier 对 16 个文本文件报告格式问题；本机相同内容检查通过。仓库原本没有换行约定，新增 `.gitattributes` 固定文本 LF，避免 Windows checkout 转换为 CRLF。使用 `git -c core.autocrlf=true checkout-index` 模拟检出，16 个格式检查目标全部保持 LF；本机 `pnpm format:check` 通过。

修复提交 `c7ee2c86e29011b42b6636993668d3385194b233` 的 [复跑 Check](https://github.com/jlau-ice/floatleaf/actions/runs/39011651513) 已全部通过，首轮失败记录保留，不重写发布标签。修复未改动业务代码、依赖或已发布安装包。

| 环境 | 结果 |
| --- | --- |
| macOS 26.6.2（25G83），镜像 macos-26-arm64 | 环境、前端构建、类型、rustfmt、Clippy、Prettier、17 项 Rust 测试、5 项浏览器测试、debug 桌面构建全部通过 |
| Windows Server 2025 Datacenter 10.0.26100，镜像 windows-2025-vs2026 | 同上全部通过；Windows 格式问题已消失 |

对应命令为 `pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm format:check`、`pnpm test:rust`、`pnpm exec playwright install chromium`、`pnpm test:web` 和 `pnpm tauri build --debug --no-bundle`。运行元数据与完整日志保存在 `.local/github-ci-result.json`、`.local/github-ci-macos.log`、`.local/github-ci-windows.log`，不入 Git。GitHub API 确认仓库公开、默认分支为 main、协议识别为 MIT。

CI 的编译、单元测试和浏览器测试不替代 Windows/macOS 实际安装、系统快捷键、输入焦点、点击穿透和多屏验收；Windows Server CI 也不等于 Windows 11 用户桌面。这些仍按 MANUAL_VALIDATION.md 收集试用反馈。
