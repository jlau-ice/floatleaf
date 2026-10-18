# macOS 后台快捷键与输入监控

日期：2026-10-18。用户报告：浮页在前台时快捷键有效，切换到微信或其他普通窗口后无效；不是仅微信、全屏或置顶场景。

## 已确认的证据

在同一独立测试进程中，用户实际按键的日志显示：

| 状态 | 收到全局按下事件 | 输入监控 | 主键采样 | 结果 |
| --- | --- | --- | --- | --- |
| 浮页前台 | 是 | 未授权 | true | 原生显示，松键隐藏 |
| 浮页后台 | 是 | 未授权 | false | 控制器未执行显示 |

未授权时修饰键仍可能被读取（`anyDown=true`），不能据此判断完整组合键可用。原始记录在 `.local/background-input/data/logs/floatleaf.log`，第一次前后台对照摘要 `.local/background-input/before-permission.json`。测试只检查配置的快捷键，不读取其他应用界面或普通输入；使用独立 SQLite 和原创测试文字，用户原书库不变。

`CGEventSourceKeyState` 返回主键 false 后，`Controller::press(all)` 按现有规则拒绝显示；不是窗口被其他应用遮盖。`CGPreflightListenEventAccess` 同时返回 false。

同一个独立调试包由用户添加并开启输入监控后，重启进程 PID 14393 读到权限 true；后台全局事件的主键和完整组合键均变为 true，并有原生显示/隐藏记录。用户实测确认三项正常：先松空格隐藏、先松 Option 隐藏、紧急隐藏保持隐藏且全部松键后可恢复。证据 `.local/background-input/after-permission.json`。测试平台为 macOS 27.0.1（26A434）arm64；不代表 Windows 或其他系统场景通过。

另行使用 `pnpm desktop:dev --no-watch` 验证：原始开发进程 PID 14725 未继承 `.app` 的授权。用户将 `src-tauri/target/debug/floatleaf` 单独加入输入监控并开启后，再次启动 PID 15370 仍返回 false，证据 `.local/background-input/dev-after-permission.json`。后续固定同一二进制对照见下方；不引导扩大终端/Codex 权限。已恢复通过实测的独立调试包供试用，原始开发进程停止。开发模式的替代验证方式见 DEVELOPMENT.md。

### 同一二进制的启动方式对照

固定已获用户授权的 `.local/background-input/FloatLeaf.app`，期间不重新编译、不改授权，两次均使用独立空书库。其签名要求为 `cdhash H"aec0f7376ac09cac07dfc5cd7dbe3ad3884f3b2a"`。

| 启动方式 | PID | 输入监控查询结果 |
| --- | --- | --- |
| 直接作为子进程执行 `FloatLeaf.app/Contents/MacOS/floatleaf` | 21368 | false |
| `open -n --env … FloatLeaf.app`，由 LaunchServices 启动 | 21613 | true |

两次都正常启动、由调试探针正常退出。原生日志保存在 `.local/background-input/launch-direct/data/logs/` 和 `launch-services/data/logs/`。这次对照排除了重新编译、应用路径与签名变化，证明本机授权结果受到启动方式影响；不能据此断言 TCC 内部具体把权限归给了哪个父进程。没有新增物理按键测试，也没有修改权限数据库或授权终端/Codex。

窗口加载完成后另行复核：直接启动 PID 22520，经真实 WebView 调用 `input_access` 仍为 false；LaunchServices 启动 PID 22579 同一路径返回 true。报告在 `launch-direct-settled/probe/ui-result.json`、`launch-services-settled/probe/ui-result.json`，汇总 `launch-comparison.json`。一次未保持父进程的启动提前结束，不计入结果。两次有效复核正常退出后已恢复原授权 app 与书库。

本机后台快捷键调试采用下方文档中的 `.app` + `open` 路径。原始 `pnpm desktop:dev` 的后台输入仍未解决，不能标为已通过。

历史核查：`native/macos.m` 中的 CombinedSessionState 采样自 `e6072f7`（P0）已存在，近期录制界面提交 `4fbc4a0` 没改这条采样路径。无法仅凭现有日志确定用户此前正常、现在失效的变化点；不能声称某次 UI 修改或 macOS 更新就是原因。开发进程、安装包及重新构建后的授权状态需要分别验证。

## 本次改动

- 设置窗口在 macOS 输入监控未授权时显示原因、授权入口和重启提示，回到窗口或点击「重新检查」刷新状态；Windows 不显示该权限要求。列表没有应用时提示手动添加，开发模式单独提示启动方式的限制。
- 仅点击「设置输入监控」才调用系统授权请求并打开系统设置；启动和状态查询不申请权限，也不会替用户授权。
- 日志记录应用前后台状态、输入权限，以及已配置组合键的实际采样值，帮助区分系统事件未收到、采样不可用和窗口显示问题。
- 保留组合键释放检查、旧事件防护和紧急隐藏规则，不通过跳过安全状态判断来掩盖后台采样失败。

## 验证

- `pnpm env:check`、`pnpm check`、27 项 Rust 测试通过。
- 11 项浏览器测试通过；新增权限引导测试覆盖 Windows 不提示、未授权不自动请求、用户点击才请求、未批准时保留提示及授权状态刷新。首次运行因沙箱禁止监听 1420 端口中断，获准在沙箱外复跑通过。
- `pnpm tauri build --debug --no-bundle` 和 `pnpm tauri build --debug --bundles app` 通过；测试包 ad-hoc 签名校验通过。真实 WKWebView 查询返回未授权、权限提示和按钮实际存在，证据 `.local/background-input/native/ui-result.json`。
- 授权后的独立 `.app` 后台按下/释放、修饰键先松、紧急隐藏：用户实测通过并与日志核对。原始开发启动仍受权限限制；已随 v0.2.3 试用包交付，不覆盖 v0.2.2。新 DMG 复制后启动、单实例与退出通过；本机新包仍提示未授权，旧调试包授权不等于新包已获准。

## 复测步骤

1. 在浮页设置页点击「设置输入监控」，允许 FloatLeaf，退出并重新打开应用。
2. 切到微信或其他普通应用，按住显示快捷键，再松开主键；重复一次先松修饰键。
3. 长按显示时触发紧急隐藏，保留旧显示组合键；确认不会重新出现。全部松开后重新触发应恢复。
4. 在其他应用输入文本，验证显示、隐藏、翻页未抢焦点；发送异常步骤与浮页日志。

系统授权由用户决定。若不授权，当前实现不能保证 macOS 后台组合键状态检测；不得自动切换显示模式或宣称按住功能正常。

参考：[Apple 输入监控说明](https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac)、本机 SDK 的 `CGPreflightListenEventAccess` / `CGRequestListenEventAccess` 定义及项目锁定的 global-hotkey 0.8.0 Carbon 实现。
