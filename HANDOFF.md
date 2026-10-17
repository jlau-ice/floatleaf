# FloatLeaf 开发交接

更新日期：2026-10-17。当前阶段：P1 SQLite 阅读闭环已实现并验证；继续 P2 窗口布局与快捷键。双平台系统验收仍待补。

## 仓库与环境

目录 `/Users/ice/jlau-ice/floatleaf`，当前分支 `main`，仅本地 Git，无远端、推送或发布。按功能阶段提交，提交号见 `git log`。

Node 22.22.3（nvm）、pnpm 10.33.4；Rust 1.99.1（rustup），rustfmt 与 Clippy。macOS 27.0.1 (26A434)，Apple Silicon arm64。已核对项目工具链；不改全局 shell 配置。

## 已实现

- 隐藏预创建 overlay，透明无框无阴影、置顶、不可聚焦、默认鼠标穿透。
- Rust `controller.rs` 是唯一显隐权威。按住/切换/固定模式、独立位置锁定、紧急隐藏、旧事件保护、编辑布局拖动。
- 默认 Ctrl+Alt+Space，紧急隐藏 Ctrl+Alt+Escape。插件注册加原生组合键成员检查，覆盖主键/修饰键释放顺序，按住检查线程可即时唤醒。
- macOS 不激活显示调用、Workspace/锁屏通知；Windows 不激活显示调用、WTS 和电源广播。Windows 代码未在 Windows 编译。
- 托盘/菜单栏入口：设置、紧急隐藏、退出；关闭设置只隐藏。
- TXT 文件选择、UTF-8/GBK 预览与确认导入、书架与移除。直接 SQLite 保存正文、每书 UTF-16 锚点和设置，无 ORM。
- WebView 实际排版分页、完整行数、字体与透明度设置；手动翻页、自动计时、隐藏暂停、末尾停止、重排保持锚点。
- 设置和阅读界面、真实状态事件同步、注册冲突错误显示。
- 调试构建可显式开启文件探针，release 不包含此入口。原生输入测试窗口和冒烟脚本已纳入仓库，本地产物在 `.local/`，不会提交。

## 已验证

| 检查 | 结果 |
| --- | --- |
| pnpm env:check | 通过，rustup 固定工具链 |
| pnpm build | 类型检查与前端生产构建通过 |
| pnpm check | 类型、rustfmt、Clippy all-targets 通过，警告视为错误 |
| pnpm test:rust | 14 项状态机、阅读计时、存储和解码测试通过 |
| pnpm test:web / pnpm format:check | 3 项实际 DOM 分页测试、格式检查通过 |
| pnpm tauri build --debug --no-bundle | 本机 Mach-O 构建通过 |
| macOS 原生运行 | 沙箱外实际启动成功，快捷键注册无错误 |
| 原生窗口探针 | 100 次显示/隐藏通过；最新运行前台始终为独立输入测试进程 42837，overlay 非 key window |
| 原生窗口属性 | opaque=false、level=5；阅读态鼠标穿透 true，编辑态 false，完成编辑恢复 true |
| 紧急隐藏 | 探针验证固定模式不能绕过隐藏锁；旧键事件保护另由状态机测试覆盖 |
| 界面检查 | 本应用 WKWebView 截图：设置页及编辑态阅读文本均已目视检查 |

P1 原生 IPC 冒烟通过：导入两本合成文本、分页/重排、隐藏暂停、手动重置、末尾停止、去重、独立进度和移走源文件后的重启恢复。修复重复分页回报造成的计时干扰，增加稳定 revision 回归检查。SQLite 使用 DELETE journal，退出后单文件备份。证据见 `docs/P1_READ_VERIFICATION.md`，本地数据 `.local/p1-final/`。

P0 证据：`docs/P0_NATIVE_VERIFICATION.md`。本地运行数据 `.local/p0/native-smoke.json`，截图 `.local/p0/main.png`、`overlay.png`（忽略提交）。原生测试不等于物理键鼠测试。

## 待实现与待验证

- 窗口拖动位置持久化、边缘尺寸调整、多屏恢复、自定义快捷键仍待实现。阅读区宽度/行数目前由设置调整。
- 本机测试进程的辅助功能自动化和屏幕录制权限均未授予。真实键鼠释放顺序/输入连续性/点击穿透待手工或获授权的自动化验证。未修改系统权限。
- 锁屏、真实休眠、全屏/多桌面、外接屏未实测。没有 Windows 设备，Windows 无编译和运行证据。TASKS 的双平台项目不能勾选。
- 没有符合目标的 CPU、内存或端到端 P95 测量，不能宣称性能达标。
- 安装包、正式签名、公证、干净设备安装和朋友试用仍待完成。原图标为生成器资源。

## 下一步

继续 P2：编辑态尺寸手柄、逻辑位置持久化、监视器工作区恢复、快捷键冲突处理；再完成 P3 本机试用包与性能测量。保持核心实现直接、简单，不增加 ORM 或无需求的抽象。Windows 和物理系统操作单列待验证，不停止其他工作。
```bash
nvm use
pnpm desktop:dev
# 直接用 cargo 前 source "$HOME/.cargo/env"；pnpm 脚本已为子进程补 PATH
```

原生验证脚本运行方法、精确限制及实机复现顺序见 `docs/P0_NATIVE_VERIFICATION.md`。临时测试进程应只按本任务 PID 退出，不结束用户其他应用。
