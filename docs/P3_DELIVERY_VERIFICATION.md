# P3 本机交付记录

日期：2026-10-17。基于 `ee339ed` 后的 P3 本地提交，精确提交见 `git log --oneline`。环境同 PERFORMANCE.md。这里仅记录 macOS 本机结果；Windows 没有编译或实机证据。

## 实现与回归

- 单实例运行；再次启动与 macOS Dock 重新打开均进入现有设置窗口。
- 自有图标、macOS app/DMG 与 Windows NSIS 配置。Windows WebView2 Bootstrapper 路径已配置但未运行。
- 导入预览处理异步结果顺序；确认失败在弹窗内显示，原生 HTML dialog 支持焦点约束与 Escape。快捷键录入不拦截 Tab，不重复录入长按事件。
- 移除非当前书保留当前自动阅读；移除当前书清空并隐藏。非排版设置不重建分页；保存相同翻页间隔不重置倒计时。
- 普通 release 不包含文件命令入口与测试数据目录覆盖。显式 `diagnostics` feature 仅用于隔离测试。

| 命令或验证 | 结果 |
| --- | --- |
| `pnpm env:check` | rustup、固定 Rust/Node、系统工具通过 |
| `pnpm build` / `pnpm check` / `pnpm format:check` | 类型、前端生产构建、Rust 格式、Clippy all-targets 与格式通过 |
| `pnpm test:rust` | 17 项通过，含计时、状态机、分页回报、持久化、编码和布局边界 |
| `pnpm test:web` | 5 项通过：3 项 DOM 分页、2 项导入交互 |
| `pnpm tauri build --debug --no-bundle` | 本机桌面程序构建通过 |
| `node scripts/native-reading-smoke.mjs .local/p3-smoke` | 实际 WebView/IPC/SQLite 阅读闭环回归通过 |
| `node scripts/native-final-smoke.mjs .local/p3-smoke` | 移除书籍、外观保存计时、快捷键界面录入与持久化通过 |
| 原生 debug 单实例 | 首实例 PID 59374 不变，第二进程退出码 0 |
| 本应用 WKWebView 截图 | 最终快捷键页面目视检查通过 |
| Word 计划书 | 更新 SQLite 与当前进度，用 LibreOffice 渲染并逐页检查全部 6 页 |

原生回归通过显式探针执行，不等同于物理键盘、鼠标或系统文件选择器人工验收。原始结果在 `.local/p3-smoke/reading-result.json`、`ui-result.json`、`single-instance.json`。

## 试用包

执行 `CI=true pnpm desktop:build`，普通 release，不使用签名凭证。沙箱内 DMG 挂载工具报“设备未配置”，在获准的沙箱外构建成功。

- `.local/release/FloatLeaf_0.1.0_aarch64.dmg`，2,986,687 字节。
- SHA-256：`89da12640a036d33e42c744d33f5d0af54887906053432c2a104d280fb7f0ddf`。
- `.app` 约 6.01 MiB；arm64；应用标识 `io.github.jlauice.floatleaf`；最低系统配置 13.0，只有 27.0.1 实测。
- `codesign --verify --deep --strict` 通过；`Signature=adhoc`，无 TeamIdentifier，未公证。此结果不是 Developer ID 正式签名。
- 二进制检查未包含 `FLOATLEAF_PROBE_DIR`、`FLOATLEAF_DATA_DIR`、`probe-result:` 诊断入口标记。

只读挂载 DMG，复制到项目内 `.local/trial-install/FloatLeaf.app` 后实际启动。`scripts/package-probe.m` 只读取该应用自身的运行与窗口元数据：PID 59663、应用标识正确、可见设置窗口 920×680。第二次启动退出码 0；通过 `NSRunningApplication.terminate` 正常退出，应用退出码 0，日志无错误。证据为 `.local/release/package-smoke.json`。

没有将应用安装到系统 Applications，也没有执行干净设备、网络转移后的 Gatekeeper 或真实菜单点击测试。这些仍需按 INSTALL.md 和 MANUAL_VALIDATION.md 验证。所有测试进程已退出，未删除用户书库。

## 尚未满足的交付条件

Windows 11 x64 构建、安装和系统键鼠验收；macOS 物理组合键释放/输入连续性/点击穿透、锁屏休眠、实际多屏拔插、全屏 Spaces；真实长按 CPU 与端到端 P95；干净设备安装和试用反馈。正式签名、公证仅在用户决定正式分发并另行授权后推进。

本机 CPU/RSS 的测量范围与原始数据见 PERFORMANCE.md。可执行的手工验收顺序见 MANUAL_VALIDATION.md。总目标保持未完成；没有创建远端、推送或公开发布。
