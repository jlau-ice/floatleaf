# Windows v0.1.0 隐藏异常

日期：2026-10-17。来源：用户在 Windows 安装包中报告按住模式不能隐藏，紧急隐藏也无效；截图中设置页状态已显示“使用显示快捷键开始阅读”，桌面仍有阅读文字。用户同时反馈 macOS 开发环境正常。未取得 Windows 完整系统版本或原版运行日志，不记录用户小说内容。

## 根因与修复

仓库锁定 Tao 0.37.1。`native/windows.c::fl_show` 用 `ShowWindow(SW_SHOWNOACTIVATE)` 显示 overlay，绕过 Tao 的 `WindowFlags::VISIBLE` 缓存。overlay 初始隐藏，缓存一直为 false。

原 `runtime::apply` 隐藏分支调用 `WebviewWindow::hide()`。Tao Windows `window.rs::set_visible(false)` 将缓存置 false，`window_state.rs::apply_diff` 发现 flags 没有变化就直接返回，不会执行后面的 `ShowWindow(SW_HIDE)`。所以 Rust 控制器和界面认为已隐藏，系统窗口仍可见。紧急隐藏也经过同一原生应用路径，因此一起失效。这是 Windows 平台适配错误，不能归因于用户用安装包启动；Windows 开发构建也使用同一条代码路径。

修复：增加 Windows `fl_hide(SW_HIDE)`，从 `platform::hide` 调用。正常隐藏和窗口错误后的紧急隐藏均使用此路径；macOS 保持原隐藏实现，设置主窗口继续由 Tauri 管理。

## 日志

`logging.rs` 直接写入本地应用数据目录的 `logs/floatleaf.log`，超过 1 MiB 轮转为 `floatleaf.previous.log`。无新依赖，无远程上报；SQLite 仍独立保存正文、进度与设置。

每行 JSON 包含 Unix 毫秒时间、事件和详情。重点对照 `shortcut`、`window.requestedVisible`、`window.nativeVisible`、`window.emergencyLatched`、`window.focused`；窗口状态重复应用不重复写日志。日志同时记录版本、系统/架构、快捷键注册错误和原生窗口操作错误。设置页可打开日志文件夹。

## 验证记录

- `pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm format:check`：通过。
- `pnpm test:rust`：18 项通过，含日志轮转及重复窗口状态去重测试。
- `pnpm test:web`：5 项通过。首轮因沙箱禁止监听本机端口未能启动，授权环境重跑通过。
- Windows `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis`：普通 release 与 NSIS 构建成功，未启用 diagnostics。
- Windows 目标 release Clippy all-targets（`-D warnings`）：通过。安装包 `7zz t`、解包及版本/日志入口核验通过，普通包不含调试命令目录标记。
- macOS 独立测试标识 debug 构建、实际启动、100 次原生显隐与紧急隐藏/编辑状态回归通过。首轮因前台进程变化中断，稳定后完整复跑通过；不当作物理输入验证。实际 WKWebView/IPC 验证日志版本、路径显示、打开文件夹成功，日志所有显隐记录与系统值一致；设置截图检查通过。测试数据位于 `.local/fix-011/`，没有接触用户书库。
- Windows 安装器：2,385,539 字节，SHA-256 `8413541f46b3c06265e363e38324ccbef72f845bcea556eced918f9cf31d31bb`。
- `scripts/windows-package-smoke.py`：从外部进程输入系统组合键，以 `IsWindowVisible` 检查六种释放顺序、紧急隐藏、旧组合键保护和重新触发，同时检查前台窗口未改变。CI `39013867420` 已在 v0.1.0 包中复现松键隐藏和紧急隐藏失败；新包构建通过，但安装后指定路径未找到 EXE，未执行窗口断言。安装路径修正后，v0.1.2 在 Windows Server 2025 安装并通过 35 项原生断言，详见 [AUTO_READING_FIX.md](AUTO_READING_FIX.md)。该轮 CI 最后附件上传失败，已修正跨盘路径问题，功能测试结果保留在运行日志中。
- Windows 11 用户设备：v0.1.0 隐藏失败；最新 v0.1.2 待用户复测。实际点击穿透、锁屏/休眠、多屏和真实键鼠仍按验收矩阵保留待测。

复测操作和取日志方式见 [v0.1.1 说明](RELEASE_v0.1.1.md)。

## 发布

源码修复提交 `683fcfd`；公开 v0.1.1 标签指向文档提交 `0b71557`，应用代码一致。本次只发布 Windows 包，macOS 继续使用既有 v0.1.0 DMG。

[v0.1.1 Release](https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.1) 的四个附件已从公开 URL 重新下载，与本地文件逐字节一致。安装器 SHA-256 见上方；试用 ZIP SHA-256 为 `c265f26d018a12675daa777c0dd591aa8c84c7edcfae58921356af0fe17c9046`。本地发布文件在 `.local/github-release-0.1.1/`，不进入 Git。
