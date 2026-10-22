# Windows 截图 / 录屏保护

## 使用与范围

设置窗口底部的「截图 / 录屏保护」开关默认关闭，开启后立即作用于悬浮阅读窗口，选择保存在现有 SQLite 设置行中，重启后恢复。按住、切换、固定和编辑布局共用同一个受保护窗口；设置页（包括书名、目录与导入预览）不在保护范围内。

需要 Windows 10 2004 或更高版本。不需要管理员权限。首次使用时，请用实际使用的截图、录屏或屏幕共享软件测试：本地应仍能看见阅读文字，而捕获结果中应只剩下方桌面/其他软件；关闭保护后应能重新捕获文字。具体工具/捕获模式是否遵守系统限制必须分别验证。

这不是 DRM 或绝对防泄漏保证，无法阻止相机拍照，也不承诺覆盖所有第三方捕获方式。macOS 不提供此开关，不以 Windows 结果声称 macOS 防截图可用。

## 原生实现

- `ReadingSettings.capture_protection` 默认 false，旧数据库缺少该字段时沿用 false；没有增加表或独立配置文件。
- Rust 在每次应用阅读窗口状态、显示窗口之前，调用当前进程拥有的顶层 overlay HWND 的 `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`；关闭时恢复 `WDA_NONE`。`GetWindowDisplayAffinity` 已与期望一致时不重复设置。
- 锁定依赖 Tao 0.37.1 的 `set_content_protection` 会丢弃原生调用错误，因此这里复用项目现有 C/Rust 平台桥，核对设置返回值和可读回的值。设置失败或读回不一致进入现有窗口错误路径：隐藏、锁住旧按键事件并向设置页报告 Windows 错误码。不会把保护设置失败当成成功继续显示。
- 应用保护在 Tao 完成鼠标/焦点样式调整后执行。进入布局编辑时 Tao 移除 `WS_EX_LAYERED`；微软的查询接口要求分层窗口，因此读回可能失败。这不等于设置失败：`SetWindowDisplayAffinity` 的成功返回仍有效，查询失败在日志中记录为未知；不会仅因编辑状态下无法查询而锁住布局。读回成功时仍核对期望值。
- 窗口日志包含期望值 `captureProtection`；Windows 诊断包含 `captureAffinity`（17 表示排除捕获、0 表示关闭，查询失败为 null）与 `captureError`。日志不记录截图像素或正文。
- 当前 Wry 0.57.0 通过 WebView2 的 `ICoreWebView2Controller` 渲染，没有由 FloatLeaf 创建和持有的 DXGI 交换链。`DXGI_SWAP_CHAIN_FLAG_DISPLAY_ONLY` 是创建/调整交换链的参数，不能设置到 HWND，也不能通过叠加一个空白 DirectX 窗口来保护原 WebView。本次没有设置或声称支持这个标志；若未来替换阅读渲染器为自有 DirectX 交换链，需要单独设计与验证。

微软说明：[SetWindowDisplayAffinity](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity)、[GetWindowDisplayAffinity 的分层窗口限制](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowdisplayaffinity)、[DXGI_SWAP_CHAIN_FLAG](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/ne-dxgi-dxgi_swap_chain_flag)。旧版 Windows 会将排除标志按 `WDA_MONITOR` 处理，不能据此承诺完整排除；支持下限保持 Windows 10 2004。

## 自动检查

- SQLite 回归覆盖旧设置缺字段、开启后落盘/重启保留；界面回归覆盖开启、切换显示模式后保留、关闭，以及 macOS 隐藏入口。
- `scripts/windows-package-smoke.py --check-capture-protection` 仅允许在 GitHub Actions 的空书库中运行。它启动普通安装包，写入合成文本及测试设置，依次在关闭 → 开启 → 关闭三次重启后采集桌面像素。结束后恢复原设置并删除合成书籍，不用于用户书库。
- 捕获使用 GDI `BitBlt` 加 `CAPTUREBLT`，覆盖透明/分层窗口。关闭时必须捕获到洋红色测试文字作为阳性对照；开启时原生窗口必须仍可见、读回 0x11，而截图须与隐藏时的背景一致，不能仅凭标志判定通过。每次还检查焦点、穿透样式、松键和紧急隐藏。
- CI 保存六张开关/显示状态截图、JSON 断言和应用日志。此项不等于 OBS、会议软件、Windows Graphics Capture、Desktop Duplication 或所有工具实测；也不代替 Windows 11 实体设备验收。

## 当前验证记录

2026-10-22：本机环境、前端构建、类型检查、rustfmt、Clippy、29 项 Rust / 13 项浏览器测试、格式检查及 macOS debug/no-bundle 和 debug app 构建通过。实际启动隔离书库的 macOS app，WKWebView 确认平台能力为 false、无 Windows 开关；既有自动阅读原生回归的隐藏开关、间隔 SQLite 保存、隐藏冻结、显示推进、再次隐藏暂停共五项通过。证据 `.local/capture-protection/mac-probe/`；不算 macOS 捕获保护或新增物理按键验收。Windows 普通安装包的捕获像素回归已接入 CI，实际结果待本次运行后补充。

## Windows 11 手工复测

1. 普通权限运行新版，导入自己编写的测试文本，固定显示；先关闭保护截图，确认图中有文字。
2. 保持阅读窗口显示，开启保护，用同一方式再截图。桌面仍有文字、截图没有文字才算通过；设置页仍可被截取是预期行为。
3. 不重启直接关闭保护，再截图应恢复文字；再开启并退出重开，确认开关保存且保护仍生效。
4. 开启保护分别试按住/切换/固定、编辑布局、主键/修饰键先松、紧急隐藏、翻页和自动翻页；记录前台输入焦点和实际点击穿透。
5. 分别测 Windows 截图工具、实际使用的第三方截图软件、OBS 的具体捕获源及会议共享的具体模式。记录 OS 构建号、软件版本、捕获模式、实际图像和结果；失败时附浮页日志。使用合成内容，不必提交真实小说或聊天画面。
