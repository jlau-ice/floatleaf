# 构建与安装说明

## macOS 本地试用

当前构建面向 Apple Silicon，最低系统版本配置为 macOS 13.0；实际测试系统为 macOS 27.0.1。其他版本仍需验证。

`pnpm desktop:build` 生成 `src-tauri/target/release/bundle/macos/FloatLeaf.app` 和 `bundle/dmg/FloatLeaf_0.1.0_aarch64.dmg`。打开 DMG，将 FloatLeaf 拖到“应用程序”后启动。试用包使用 ad-hoc 本地签名，不使用任何签名证书，也未进行 Apple 公证。它不是正式签名发行包；从网络转移到另一台 Mac 后可能触发系统安全提示，需要在“系统设置 → 隐私与安全性”确认来源并允许打开。不建议关闭系统安全检查。

退出请使用菜单栏的“退出浮页”。卸载时先退出，再将应用移到废纸篓；书库仍保留在 `~/Library/Application Support/io.github.jlauice.floatleaf/`。需要清除书库时先确认备份，再手工移除此目录。

正式 Developer ID 签名与公证需要另行提供账号并授权。`tauri.macos.conf.json` 的 `signingIdentity: "-"` 是本地试用配置，正式发行时用受控覆盖配置替换，凭证不写入仓库。

## Windows 构建路径

已在 macOS 交叉构建 Windows x64 NSIS 试用包，并通过 Windows 目标 Clippy 和安装包静态核验；尚无 Windows 实机运行结果。可将 `.local/release/FloatLeaf_0.1.0_windows-x64_trial.zip` 发给 Windows 11 x64 试用者，解压后运行安装程序。说明、示例和反馈表已包含在 ZIP 中，构建命令与哈希见 WINDOWS_BUILD.md。

如需在 Windows 本机构建，安装 Visual Studio C++ 构建工具与 Windows SDK、rustup、项目要求的 Node 和 pnpm 后执行：

```powershell
pnpm install --frozen-lockfile
pnpm env:check
pnpm build
pnpm check
pnpm test:rust
pnpm exec playwright install chromium
pnpm test:web
pnpm desktop:build
```

Rust 版本由 `rust-toolchain.toml` 固定；不要使用其他来源的 Rust。Windows 配置生成当前用户安装的 NSIS 安装程序，支持简体中文和英文。产物位于 `src-tauri/target/release/bundle/nsis/`。

目标机需要 WebView2。已安装运行时则直接使用；缺少运行时的机器会通过安装器下载 Bootstrapper 并安装，因此首次安装可能需要网络。应用正常阅读不需要网络。纯离线部署需预装 Microsoft WebView2 Evergreen Runtime，再安装浮页。

卸载使用 Windows“设置 → 应用”，本地书库默认保留在 `%APPDATA%\io.github.jlauice.floatleaf\`。设置界面显示的实际数据路径优先。正式签名、SmartScreen 提示与干净设备安装待验证。

## 本地验证与诊断

普通 release 构建不带文件命令入口。`pnpm tauri bundle --bundles dmg` 可能清理中间的 `.app`；需要单独应用目录时可从 DMG 复制，或执行 `pnpm tauri bundle --bundles app`。`diagnostics` 是显式 opt-in 的本地原生测试 feature，不能作为面向用户的试用包：

```bash
pnpm tauri build --features diagnostics --no-bundle
cp src-tauri/target/release/floatleaf .local/floatleaf-diagnostics
node scripts/native-performance.mjs .local/performance .local/floatleaf-diagnostics
# 测完重新构建普通安装包，确保未传 --features diagnostics
CI=true pnpm desktop:build
```

诊断版才接受 `FLOATLEAF_DATA_DIR` 和 `FLOATLEAF_PROBE_DIR`；使用隔离目录、合成文本，不运行在真实书库上。性能脚本用 macOS `ps` 统计主进程及两个本应用 WebContent 进程的 CPU/RSS，未包含共享 GPU/网络服务，RSS 可能重复计算共享内存。文件探针的开销包含在结果里。

CI 工作流定义 macOS 与 Windows 两端自动检查；云端检查结果应与本机结果分开记录。用户已授权 GitHub 仓库创建、推送与公开试用发布，发布说明见 RELEASE_v0.1.0.md。正式签名和公证仍需签名账号及额外授权。
