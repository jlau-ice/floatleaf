# FloatLeaf 开发交接

更新日期：2026-10-17。当前阶段：macOS 试用包已验证启动；Windows x64 NSIS 试用包已交叉构建并完成静态核验，等待实机反馈。用户已选择先做 macOS 手工验收，并准备让 Windows 同学试用；目前尚未收到手工通过或异常结果，持续目标尚未完成。

## 仓库与环境

目录 `/Users/ice/jlau-ice/floatleaf`，当前分支 `main`。用户已授权创建 `jlau-ice/floatleaf` 公开仓库、推送源码并发布两端试用安装包，采用 MIT 协议。公开 README、发布说明和附件已准备；GitHub 插件已确认账号，但没有建仓库和上传 Release 的工具，本机 GitHub CLI 尚需登录后执行。按功能阶段提交，提交号见 `git log`。

Node 22.22.3（nvm）、pnpm 10.33.4；Rust 1.99.1（rustup），rustfmt 与 Clippy。macOS 27.0.1 (26A434)，Apple Silicon arm64。已核对项目工具链；不改全局 shell 配置。

## 已实现

- 隐藏预创建 overlay，透明无框无阴影、置顶、不可聚焦、默认鼠标穿透。
- Rust `controller.rs` 是唯一显隐权威。按住/切换/固定模式、独立位置锁定、紧急隐藏、旧事件保护、编辑布局拖动。
- 默认 Ctrl+Alt+Space，紧急隐藏 Ctrl+Alt+Escape。插件注册加原生组合键成员检查，覆盖主键/修饰键释放顺序，按住检查线程可即时唤醒。
- macOS 不激活显示调用、Workspace/锁屏通知；Windows 不激活显示调用、WTS 和电源广播。Windows 代码已在 Mac 上交叉编译，未在 Windows 上构建或运行。
- 托盘/菜单栏入口：设置、找回位置、紧急隐藏、退出；关闭设置只隐藏。
- 自定义五组快捷键；编辑态右下角调整宽度和完整行数；SQLite 保存逻辑位置；工作区约束、显示器变化和缩放重排、原屏消失时主屏恢复。
- TXT 文件选择、UTF-8/GBK 预览与确认导入、书架与移除。直接 SQLite 保存正文、每书 UTF-16 锚点和设置，无 ORM。
- WebView 实际排版分页、完整行数、字体与透明度设置；手动翻页、自动计时、隐藏暂停、末尾停止、重排保持锚点。
- 设置和阅读界面、真实状态事件同步、注册冲突错误显示。
- 单实例运行、Dock 重新打开设置；自有图标和两端安装配置。macOS 普通 release 试用 DMG 已产出。
- debug 或显式 `diagnostics` feature 可开启文件探针，普通 release 不包含此入口。原生输入测试窗口和冒烟脚本已纳入仓库，本地产物在 `.local/`，不会提交。

## 已验证

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

P1 原生 IPC 冒烟通过：导入两本合成文本、分页/重排、隐藏暂停、手动重置、末尾停止、去重、独立进度和移走源文件后的重启恢复。修复重复分页回报造成的计时干扰，增加稳定 revision 回归检查。SQLite 使用 DELETE journal，退出后单文件备份。证据见 `docs/P1_READ_VERIFICATION.md`，本地数据 `.local/p1-final/`。

P0 证据：`docs/P0_NATIVE_VERIFICATION.md`。本地运行数据 `.local/p0/native-smoke.json`，截图 `.local/p0/main.png`、`overlay.png`（忽略提交）。原生测试不等于物理键鼠测试。

P3 证据与包哈希见 `docs/P3_DELIVERY_VERIFICATION.md`。试用包：`.local/release/FloatLeaf_0.1.0_aarch64.dmg`（2,986,687 字节），SHA-256 `89da12640a036d33e42c744d33f5d0af54887906053432c2a104d280fb7f0ddf`。最终原生回归数据在 `.local/p3-smoke/`，包启动证据在 `.local/release/package-smoke.json`。上述自动回归测试进程已退出。

Windows 新产物见 `.local/release/FloatLeaf_0.1.0_windows-x64_trial.zip`（2,395,100 字节），安装器单独保存在同目录。构建命令、工具版本、文件哈希、已知构建警告与核验范围见 `docs/WINDOWS_BUILD.md`。业务代码与锁文件未改变。macOS 手工验收已准备 `.local/manual-macos/` 示例及输入窗口，结果待用户反馈。

## 待验证与外部条件

- P2 程序路径已验证，见 `docs/P2_LAYOUT_VERIFICATION.md`。macOS Carbon 允许共享快捷键；独占冲突检查通过，不能保证检测所有其他应用拦截。
- 本机测试进程的辅助功能自动化和屏幕录制权限均未授予。真实键鼠释放顺序/输入连续性/点击穿透待手工或获授权的自动化验证。未修改系统权限。
- 锁屏、真实休眠、全屏/多桌面、外接屏未实测。Windows 已有交叉构建证据，仍无 Windows 本机构建、安装和运行证据。TASKS 的双平台项目不能勾选。
- CPU/RSS 已测隐藏空闲、固定静态与自动阅读；固定静态不是物理长按。端到端 P95 未测，不能宣称响应目标通过。采样使用优化诊断版，最终普通 DMG 未直接采样。
- macOS 包为 ad-hoc 本地试用包，Windows NSIS 未签名；正式签名、公证、干净设备安装和朋友试用结果待完成。

## 下一步

下一步由用户转发 Windows ZIP，同学在 Windows 11 x64 安装并试用；用户继续 macOS 手工验收。按 `docs/TRIAL_GUIDE.md` 的检查项先确认 P0 核心场景，收到异常后修复，再补齐 `docs/MANUAL_VALIDATION.md` 剩余验收。仅在正式分发时才需要签名账号和额外授权。

保持实现直接、简单：rusqlite bundled，无 ORM、额外仓储框架或独立小说文件。不得将当前本机程序验证外推为双平台通过。公开发布准备见 `docs/RELEASE_v0.1.0.md`，附件在 `.local/github-release/`，包含两端安装器、Windows 试用 ZIP、Word 说明、MIT 协议和 SHA-256 清单，不进入 Git。历史提交已检查，未包含依赖目录、编译缓存、真实小说、用户书库或已知凭据模式。当前发布尚待 GitHub CLI 登录，不能视为已推送。阶段提交见 `git log`。
```bash
nvm use
pnpm desktop:dev
# 直接用 cargo 前 source "$HOME/.cargo/env"；pnpm 脚本已为子进程补 PATH
```

原生验证脚本运行方法、精确限制及实机复现顺序见 `docs/P0_NATIVE_VERIFICATION.md`。临时测试进程应只按本任务 PID 退出，不结束用户其他应用。
