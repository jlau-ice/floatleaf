# Windows 标题栏优化（v0.2.1）

2026-10-17：用户反馈 Windows 深色系统标题栏与浅色设置页面不协调。

## 实现

- 仅 Windows 的 `main` 设置窗口固定为浅色主题，使用系统 DWM API 将标题栏设为页面同色 `#f4f2e9`，标题文字为 `#203b32`。
- 保留原生标题栏、系统按钮、边框、拖动、缩放、双击最大化和系统贴靠，不增加自绘窗口按钮或前端窗口权限。
- 原阅读窗口的显隐、透明、焦点和穿透路径不变；macOS 窗口外观不变。
- 标题栏应用结果写入现有日志的 `settings-appearance`，失败只记录错误，不阻止阅读。自定义标题颜色面向 Windows 11（Build 22000 起）；旧系统保留原生浅色外观，不承诺相同配色。

颜色与 `src/App.vue` 保持一致；原生实现位于 `src-tauri/native/windows.c`，Rust 平台层负责调用，未引入新依赖。

## 验证

- 本机 `pnpm env:check`、`pnpm build`、`pnpm check` 通过；Windows release 目标 Clippy all-targets（警告视为错误）通过。
- Windows 普通 NSIS 与 macOS debug 构建待完成。
- 扩展现有 Windows 安装回归：查询实际 DWM 配色、检查原生标题栏/缩放样式，执行最小化、最大化、恢复、关闭驻留；原 35 项悬浮显隐/焦点断言保留。
- CI 的临时 Windows runner 使用深色应用偏好，截图记录真实设置窗口；不改用户设备的系统主题。Windows CI 结果、截图目视核对及安装包发布待补。
- Windows 11 用户设备的实际标题栏、按钮悬停、拖动贴靠与不同缩放仍需复测。

参考：[微软 DWM 标题栏颜色与系统版本要求](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)、[DwmSetWindowAttribute](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmsetwindowattribute)。
