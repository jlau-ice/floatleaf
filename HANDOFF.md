# FloatLeaf 开发交接

更新日期：2026-10-17。当前阶段：P0 原生实现和 macOS 窗口探针完成；完整双平台实机验收待补，继续 P1 阅读闭环。

## 仓库与环境

目录 `/Users/ice/jlau-ice/floatleaf`，当前分支 `main`，仅本地 Git，无远端、推送或发布。按功能阶段提交，提交号见 `git log`。

Node 22.22.3（nvm）、pnpm 10.33.4；Rust 1.99.1（rustup），rustfmt 与 Clippy。macOS 27.0.1 (26A434)，Apple Silicon arm64。已核对项目工具链；不改全局 shell 配置。

## 已实现

- 隐藏预创建 overlay，透明无框无阴影、置顶、不可聚焦、默认鼠标穿透。
- Rust `controller.rs` 是唯一显隐权威。按住/切换/固定模式、独立位置锁定、紧急隐藏、旧事件保护、编辑布局拖动。
- 默认 Ctrl+Alt+Space，紧急隐藏 Ctrl+Alt+Escape。插件注册加原生组合键成员检查，覆盖主键/修饰键释放顺序，按住检查线程可即时唤醒。
- macOS 不激活显示调用、Workspace/锁屏通知；Windows 不激活显示调用、WTS 和电源广播。Windows 代码未在 Windows 编译。
- 托盘/菜单栏入口：设置、紧急隐藏、退出；关闭设置只隐藏。
- Vue 原生验证界面、真实状态事件同步、注册冲突错误显示。
- 调试构建可显式开启文件探针，release 不包含此入口。原生输入测试窗口和冒烟脚本已纳入仓库，本地产物在 `.local/`，不会提交。

## 已验证

| 检查 | 结果 |
| --- | --- |
| pnpm env:check | 通过，rustup 固定工具链 |
| pnpm build | 类型检查与前端生产构建通过 |
| pnpm check | 类型、rustfmt、Clippy all-targets 通过，警告视为错误 |
| pnpm test:rust | 5 项状态机边界测试通过 |
| pnpm tauri build --debug --no-bundle | 本机 Mach-O 构建通过 |
| macOS 原生运行 | 沙箱外实际启动成功，快捷键注册无错误 |
| 原生窗口探针 | 100 次显示/隐藏通过；最新运行前台始终为独立输入测试进程 42837，overlay 非 key window |
| 原生窗口属性 | opaque=false、level=5；阅读态鼠标穿透 true，编辑态 false，完成编辑恢复 true |
| 紧急隐藏 | 探针验证固定模式不能绕过隐藏锁；旧键事件保护另由状态机测试覆盖 |
| 界面检查 | 本应用 WKWebView 截图：设置页及编辑态阅读文本均已目视检查 |

证据：`docs/P0_NATIVE_VERIFICATION.md`。本地运行数据 `.local/p0/native-smoke.json`，截图 `.local/p0/main.png`、`overlay.png`（忽略提交）。原生测试不等于物理键鼠测试。

## 待实现与待验证

- TXT 导入、书架、实际排版分页、UTF-16 锚点、每书进度与设置存储、手动/自动翻页、字体样式、尺寸重排、多屏恢复、自定义快捷键仍待实现。
- 本机测试进程的辅助功能自动化和屏幕录制权限均未授予。真实键鼠释放顺序/输入连续性/点击穿透待手工或获授权的自动化验证。未修改系统权限。
- 锁屏、真实休眠、全屏/多桌面、外接屏未实测。没有 Windows 设备，Windows 无编译和运行证据。TASKS 的双平台项目不能勾选。
- 没有符合目标的 CPU、内存或端到端 P95 测量，不能宣称性能达标。
- 安装包、正式签名、公证、干净设备安装和朋友试用仍待完成。原图标为生成器资源。

## 下一步

按用户本次持续目标，在保留外部待验证项的同时推进 P1，不等待 Windows 设备而停止其他工作。先实现 TXT 预览/解码/复制、版本化原子存储，再接 WebView 实际测量分页、进度、翻页与样式。修改后运行相关测试、`pnpm build`、`pnpm check`，原生窗口变更追加桌面构建与运行。

```bash
nvm use
pnpm desktop:dev
# 直接用 cargo 前 source "$HOME/.cargo/env"；pnpm 脚本已为子进程补 PATH
```

原生验证脚本运行方法、精确限制及实机复现顺序见 `docs/P0_NATIVE_VERIFICATION.md`。临时测试进程应只按本任务 PID 退出，不结束用户其他应用。
