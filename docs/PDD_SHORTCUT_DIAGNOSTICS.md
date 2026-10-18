# Windows 商家工作台前台快捷键诊断

日期：2026-10-18。本轮是增加必要诊断的 v0.2.3 候选版，尚未证明商家工作台兼容性已修复。

## 用户反馈与旧日志

- Windows 的拼多多商家工作台在前台时无法唤起浮页，其他软件基本正常。
- 关闭商家工作台内的快捷键仍不恢复；保持商家工作台运行，把微信放到前台后恢复。
- 用户提供的日志来自普通 v0.2.2 x64 包（`debug=false`），共 288 条记录。显示与紧急隐藏快捷键注册成功；默认上一页/下一页注册失败，提示被占用。这两处冲突不能解释全部显示失败。
- `Ctrl+Alt+V` 记录 22 组按下/释放，其中 13 组间隔不超过 2 ms；`Ctrl+→` 为 76 组中的 38 组，`Ctrl+←` 为 20 组中的 15 组。旧日志没有前台应用与实际采样，无法把这些事件精确归属到商家工作台。
- global-hotkey 0.8.0 的 Windows 实现收到 `WM_HOTKEY` 后发送 Pressed，再使用 `GetAsyncKeyState` 判断 Released。该 API 在桌面或 UIPI 权限限制下也可能返回零。因此“前台权限导致主键采样不可用”是需要验证的方向，尚非根因定论。

原日志保留在用户提供的位置，不复制到仓库、不上传公开附件。没有修改商家工作台、微信或系统权限，也不要求把浮页常驻为管理员。

## 新增记录

仅在配置的快捷键事件、实际显隐变化和按键监测导致隐藏时记录：

- `shortcut`：逻辑操作、按下/释放、主键/完整组合键/任一成员采样，以及处理前后的 held、等待全松/主键松开、紧急隐藏和显示状态。
- `windows.application` / `windows.foreground`：PID、可执行文件的文件名、完整性级别 RID；查询失败时保留 Win32 错误码，不把失败写成“普通权限”。常见 RID 为 4096（低）、8192（中）、12288（高）、16384（系统）。
- 原生窗口是否可见、置顶、不激活、穿透、DWM 隐藏状态、位置与前台窗口相对层级；`lastShowError` 为最近一次 SetWindowPos 的结果。
- `shortcut-watch-hide`：轮询发现组合键已不完整并隐藏的采样状态。重新设置快捷键的注册成功/失败也写入日志。

不记录窗口标题、网页 URL、完整程序路径、聊天、订单、小说正文或普通打字。沿用现有 1 MiB × 两份轮转，不增加常驻全键监听、键盘钩子或新的后台采样频率。日志入口仍为设置页底部「日志位置 → 打开日志文件夹」。

## 同学复测步骤

1. 从托盘退出旧版，安装 v0.2.3 诊断候选包；确认「日志位置」显示 v0.2.3，保留原书库。
2. 先在微信前台，按住当前显示快捷键约 2 秒并松开，连续三次。
3. 切到商家工作台前台，保持相同组合键和操作，连续三次。
4. 商家工作台保持运行，再把微信放到前台，重复一次。失败时额外按一次紧急隐藏。
5. 打开浮页日志文件夹，发送 `floatleaf.log` 和存在时的 `floatleaf.previous.log`，并注明各段大概时间与显示结果。不要上传订单截图或真实小说。

Windows 备用日志路径：`%APPDATA%\io.github.jlauice.floatleaf\logs`。本版目标是把故障环节定位清楚，不能仅凭诊断代码编译成功声称已解决。

## 验证范围

本机类型/rustfmt/Clippy、27 项 Rust 测试、前端构建/格式及 Windows 目标 release Clippy 通过。Windows 普通包原生回归增加四项断言：配置按键采样、前台进程名/PID、完整性级别、实际窗口属性；构建、远端执行和安装包状态以 HANDOFF.md 更新为准。没有商家工作台实机新日志之前，兼容性保持待验证。

参考：[Microsoft GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate)、[RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)、[完整性控制](https://learn.microsoft.com/en-us/windows/win32/secauthz/mandatory-integrity-control)。
