# P0 原生窗口验证记录

日期：2026-10-17；本机 macOS 27.0.1 (26A434)，Apple Silicon arm64，窗口缩放 2。代码阶段为 P0 原生控制器，实际提交可查看 `git log`。

## 已实现

- 预创建隐藏、透明、无边框、无阴影、置顶、不可聚焦的 overlay。
- macOS 使用 `orderFrontRegardless`；Windows 使用 `SW_SHOWNOACTIVATE` 和 `SWP_NOACTIVATE`。阅读窗口不调用 `set_focus`。
- 默认显示快捷键 `Control+Alt+Space`，紧急隐藏 `Control+Alt+Escape`。
- 插件负责全局注册和触发；按住期间按 12 ms 检查当前组合键成员的系统状态。开始按住时唤醒检查线程，避免空闲等待影响释放响应。空闲不扫描键盘。
- macOS `CGEventSourceKeyState`，Windows `GetAsyncKeyState`，只检查配置的快捷键成员；无全键盘记录或输入事件 tap。
- 紧急隐藏后必须完全松开组合键；旧 Pressed、主键仍按着时重按修饰键、切换模式前的按键都不能恢复显示。
- Windows 接入 WTS 锁屏/解锁和电源广播；macOS 接入 Workspace 睡眠/唤醒/会话失活与锁屏通知。超长事件循环间隔也安全隐藏。
- 三种显示模式、独立位置锁定、编辑时关闭穿透、拖动手柄、完成后恢复穿透；托盘提供设置、紧急隐藏与退出。尺寸调整与持久化属于后续工作。

## 检查命令与证据

`pnpm env:check`、`pnpm build`、`pnpm check`、`pnpm test:rust`、`pnpm tauri build --debug --no-bundle` 通过。Rust 状态机 5 项测试覆盖不同释放顺序、重复事件、紧急隐藏、模式变更和位置锁定。Clippy 使用 all-targets，警告视为错误。

锁定版本：tauri 2.12.4、tauri-plugin-global-shortcut 2.4.1、global-hotkey 0.8.0、tao 0.37.1、wry 0.57.0；完整依赖以 Cargo.lock 为准。

调试构建的原生探针必须显式设置环境变量，正式 release 构建不含文件命令入口：

```bash
mkdir -p .local/p0
FLOATLEAF_PROBE_DIR="$PWD/.local/p0" src-tauri/target/debug/floatleaf
# 另一个终端启动独立输入窗口；在浮页启动后启动，确保它位于前台
clang -fobjc-arc -framework AppKit scripts/native-fixture.m -o .local/native-fixture
.local/native-fixture
# 再一个终端执行
node scripts/native-smoke.mjs
```

当前执行环境的沙箱阻止 macOS 桌面服务，实际测试程序在沙箱外启动。默认用户终端直接运行即可。

探针读取真实 NSWindow：`opaque=false`、`level=5`、`keyWindow=false`、阅读态 `ignoresMouseEvents=true`、编辑态为 false。100 次真实显示/隐藏循环完成；紧急隐藏压过固定显示；编辑退出后恢复鼠标穿透属性。原生 WebView 截图已经逐张检查，设置页与阅读文本正常。截图是本应用 WebView 内容，不是桌面合成截图。

本地未跟踪证据：`.local/p0/native-smoke.json`、`main.png`、`overlay.png`。脚本会检查每次显隐前后前台进程一致，最终记录应与独立输入测试进程的 PID 对照。它不注入键盘或鼠标，不能证明输入字符未丢失或物理点击穿透。

## 尚未验证

- 本机没有授予测试进程辅助功能自动化及屏幕录制权限；未自动注入物理按键/鼠标。真实快捷键任意释放顺序、长按、快速按放、输入连续性和实际点击穿透保留待测。
- 锁屏/真实睡眠、外接显示器与全屏 Spaces 没有执行。不能把通知回调接入作为实机通过证据。
- 无 Windows 设备，Windows 适配未编译或运行；不勾选双平台验收。
- 没有屏幕像素的端到端显隐延迟数据；不以窗口 API 耗时、文件探针往返或编译时间冒充 P95。
- 没有空闲 5 分钟、长按 60 秒、自动翻页 10 分钟的性能结论。后续使用 release 构建，关闭调试文件采样后测量。

## 实机复现顺序

1. 打开任意编辑器持续输入，按下显示组合键，先松 Space，再先松 Ctrl/Alt；分别测试左右修饰键。
2. 只松一个修饰键再按回、主键保持按下，应保持隐藏；主键松开后新按一次正常显示。
3. 按住显示时触发紧急隐藏；保持旧主键/任意修饰键，不得复活；全部释放后新按正常。
4. 在阅读窗口文字覆盖处点击并拖选底层编辑器；开启编辑后拖动阅读区域，完成编辑后复测穿透。
5. 切换/固定模式分别测试紧急隐藏；关闭设置，从托盘重新打开。
6. 按住或固定显示时锁屏/睡眠，恢复后保持隐藏；记录全屏、桌面切换和多屏行为。

参考：Tauri [窗口接口](https://v2.tauri.app/reference/javascript/api/namespacewindow/)、[全局快捷键](https://v2.tauri.app/plugin/global-shortcut/)。具体行为以本仓库锁定的本机依赖源码与实机结果为准。
