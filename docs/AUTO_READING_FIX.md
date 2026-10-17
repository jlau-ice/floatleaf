# v0.1.2 自动翻页控制

日期：2026-10-17。用户要求自动翻页时间可调，并报告按住模式无法操作自动翻页开关。

## 原因和改动

原设置按钮的禁用条件包含 `!visible`，Rust 的 `reader_action("auto")` 和暂停/恢复快捷键也要求窗口可见。按住模式一松开组合键，阅读窗隐藏，设置按钮就被禁用，无法提前开启或暂停。

v0.1.2 允许隐藏时通过设置按钮或暂停/恢复快捷键切换自动翻页。实际倒计时仍受 `visible`、编辑状态和分页就绪状态控制，隐藏期间不消耗时间、不推进进度。手动上一页/下一页仍仅在可见时有效。

将原“文字与布局”中的 2～60 秒间隔滑块移到阅读控制按钮旁，使用既有 SQLite 设置保存路径，不添加第二套配置。改变间隔后从新间隔重新计时，不会自动开始已经暂停的阅读；隐藏时显示“显示后继续 · 剩余 N 秒”。

## 验证

- 本机 `pnpm build`、`pnpm check`、`pnpm format:check` 通过；19 项 Rust、5 项浏览器测试通过。
- 新增阅读控制器测试：改变间隔重置倒计时，保留开启/暂停意图，隐藏时不推进，显示后按新间隔翻页。
- SQLite 已有重启测试补充非默认间隔，检查设置持久化。
- `scripts/native-auto-smoke.mjs .local/auto-012/native` 在真实 macOS debug 应用的 WKWebView/IPC 中通过：隐藏的按住模式下按钮可开关；界面改为 3 秒后 Rust 和 SQLite 同步；等待超过 3 秒隐藏进度仍不变；显示后自动翻页；再次隐藏冻结剩余时间。
- 应用使用独立测试标识、独立数据库和合成文本，不接触用户书库。界面截图检查通过；本轮不宣称 Windows 11 物理键鼠验收通过。
- 真实应用正常退出并重新启动后，间隔仍为 3 秒，阅读锚点 55 恢复，窗口初始隐藏；原始结果 `.local/auto-012/native/restart-result.json`。
- 两端普通 release 安装包已构建。Windows 首次在下载 NSIS 工具时遇到网络 EOF，重试成功；包 `7zz t` 通过。macOS app/DMG 生成成功，未使用正式签名或公证凭据。
- Windows 目标 release Clippy all-targets（`-D warnings`）通过；macOS `codesign --verify --deep --strict`、`hdiutil verify` 和 app 版本/标识检查通过，普通包无调试探针。

安装器及说明保存在 `.local/github-release-0.1.2/`：

| 文件 | 字节 | SHA-256 |
| --- | --- | --- |
| `FloatLeaf_0.1.2_aarch64.dmg` | 2,995,550 | `d8b814dc4e13ae15591b6d04200fced19b2039a57131603bd7dda2c71161e531` |
| `FloatLeaf_0.1.2_x64-setup.exe` | 2,385,393 | `68804b0055ef60a90525dfbd6458756f40a6dd7b2203c64ef5f2224aa347f2c0` |
| `FloatLeaf_0.1.2_windows-x64_trial.zip` | 2,371,189 | `aa926ce8eaecfaafbc2a3435066214ab0042bf8d889e9658d61eb23a1bd0b069` |

## 上一轮 Windows CI

运行 `39013867420` 的 macOS 检查通过。Windows 的类型/格式/Clippy、18 项 Rust、5 项浏览器测试和普通 release/NSIS 构建通过；v0.1.0 包的六种松键顺序、紧急隐藏及旧组合键残留失败被自动化复现。

该轮新包验证没有执行到窗口断言：静默安装返回 0，但测试从指定目录启动 EXE 时报告 `WinError 2`。工作流已将混用分隔符的安装路径改为 PowerShell `Join-Path`，并打印安装注册信息和目录；需以复跑结果确认，不能将这一轮说成新包原生验收通过。

v0.1.2 实现提交 `edb350a`，复跑编号 `39019344546`，提交时结果仍在进行中。
