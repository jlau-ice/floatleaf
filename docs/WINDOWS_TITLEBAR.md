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
- Windows 普通 NSIS 与 macOS debug（`pnpm tauri build --debug --no-bundle`）构建通过；NSIS 解包、CRC、0.2.1 版本与普通包不含诊断入口核验通过。安装器 2,987,232 字节，SHA-256 `d4357be5b6c6eb810f488a8a386ab0aef721eb23eed56046d14e382382536e48`。本地日志在 `.local/titlebar-021/`。
- [源码检查 39037365528](https://github.com/jlau-ice/floatleaf/actions/runs/39037365528) 的两端环境、类型、格式、Clippy、27 项 Rust / 7 项浏览器测试通过；macOS job 成功，Windows 普通 NSIS 构建通过。该轮 Windows 原生测试因设置窗口尚未就绪而失败，整轮状态仍为失败。
- [安装包原生回归 39040506097](https://github.com/jlau-ice/floatleaf/actions/runs/39040506097) 成功：Windows Server 2025（10.0.26100）、100% 缩放，从公开候选版下载上述普通 EXE 并实际安装启动，43 项断言全部通过。包括原 35 项六种松键顺序、紧急隐藏、旧组合键保护、前台焦点与窗口样式检查，以及浅色原生控件、边框样式、最小化/最大化/恢复、关闭驻留和两项标题栏像素检查。
- CI 临时 runner 使用深色应用偏好，截图中的标题栏为 `#f4f2e9`，标题存在 `#203b32` 像素；目视核对通过，日志 `settings-appearance.applied=true`。不修改用户设备主题。截图前仅将设置窗口临时移到前方，不能据此声称设置页始终置顶。证据在 `.local/titlebar-021/final-test/`。
- 测试脚本修复了窗口就绪等待、输入测试窗口聚焦和截图遮挡问题。DWM 的颜色读取接口返回 `E_INVALIDARG`，所以改为检查实际渲染像素；此前失败记录保留。最终测试脚本提交为 `30beba7`，应用实现仍为 `249c0c5`，未因测试调整更改 EXE。该轮报告末尾的旧 scope 文案尚未提到新增像素检查，实际检查列表和截图有效；随后仅纠正了报告说明文字。
- Windows 11 用户设备的实际标题栏、按钮悬停、拖动贴靠与不同缩放仍需复测。

## 交付

[v0.2.1 Windows 试用版](https://github.com/jlau-ice/floatleaf/releases/tag/v0.2.1) 已公开发布，标签 `140c974`。安装器、含合成 EPUB/TXT 的试用 ZIP、复测说明、真实 Windows 截图及 SHA256SUMS 五个附件经未登录公开 URL 重新下载，逐字节及 SHA-256 与本地一致，报告在 `.local/titlebar-021/download/verification.json`。macOS 安装包沿用 v0.2.0。

工作流保持 `contents: read`，通过公开候选版 `v0.2.1-rc.1` 验证安装器，没有扩大 CI 写权限。正式 v0.2.1 与候选版使用同一个已验证 EXE。

参考：[微软 DWM 标题栏颜色与系统版本要求](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)、[DwmSetWindowAttribute](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmsetwindowattribute)。

| 附件 | 字节数 | SHA-256 |
| --- | --- | --- |
| FloatLeaf_0.2.1_windows-x64_trial.zip | 2,975,861 | `58665e62787fa9a01a3f6d405fbcc28e53c214f0e7db5d160c65264dce31c373` |
| FloatLeaf_0.2.1_x64-setup.exe | 2,987,232 | `d4357be5b6c6eb810f488a8a386ab0aef721eb23eed56046d14e382382536e48` |
| Retest-Guide.txt | 912 | `03b55310e00e6bd177e1c5325bca920eb73cbc25392fcf97ca49705d413af192` |
| SHA256SUMS.txt | 370 | `b2ebc5dd97775d8ce50391fcfdaeb229deed68f9bb4658121aa64e5b6e3cfd43` |
| Windows-Titlebar.png | 25,451 | `5ac71a7b4890770b3d8144dbef155303ea26c1b7010daf5634b79d611930e78e` |
