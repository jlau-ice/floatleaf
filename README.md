<p align="center"><img src="assets/floatleaf-icon.svg" width="96" alt="FloatLeaf 图标"></p>

# 浮页 FloatLeaf

像桌面歌词一样，在桌面上阅读本地小说。**按住快捷键显示文字，松开隐藏**，随时接着上次的位置阅读。

FloatLeaf 使用 Tauri 2、Vue 3、TypeScript 和 Rust，支持 macOS 与 Windows，书籍和进度保存在本机 SQLite，无需账号或数据库服务。

## 下载与安装

当前提供 **Windows / macOS v0.2.3 试用版（Pre-release）**。新增 Windows 前台快捷键诊断日志和 macOS 输入监控权限引导。包含 EPUB / TXT 章节目录、自动翻页、快捷键录制、Windows 标题栏和此前隐藏修复。**本版用于排查拼多多商家工作台前台无法显示的问题，尚未证明该兼容问题已修复。**

| 平台 | 下载 | 安装方法 |
| --- | --- | --- |
| macOS · Apple Silicon（M 系列） | [下载 DMG](https://github.com/jlau-ice/floatleaf/releases/download/v0.2.3/FloatLeaf_0.2.3_aarch64.dmg) | 退出旧版，打开 DMG，将 FloatLeaf 拖到“应用程序” |
| Windows · x64 | [下载安装程序](https://github.com/jlau-ice/floatleaf/releases/download/v0.2.3/FloatLeaf_0.2.3_x64-setup.exe) | 先从托盘退出旧版，再运行 EXE；无需删除书库 |
| Windows · 复测资料 | [下载试用 ZIP](https://github.com/jlau-ice/floatleaf/releases/download/v0.2.3/FloatLeaf_0.2.3_windows-x64_trial.zip) | 含安装程序、商家工作台复测步骤和校验清单 |

[全部版本与校验文件](https://github.com/jlau-ice/floatleaf/releases) · [试用说明](docs/TRIAL_GUIDE.md) · [完整使用说明](docs/USER_GUIDE.md)

- macOS 配置最低版本为 13.0，本轮测试机为 macOS 27.0.1；Intel Mac 尚无安装包。
- Windows 本轮面向 Windows 11 x64。需要 WebView2；缺少运行时时，安装器会联网下载，日常阅读可离线使用。
- 两端均未使用正式发布证书，macOS 包为 ad-hoc 签名且未公证。系统可能提示未知发布者或来源未验证，请确认下载来源后按系统提示处理。安装与卸载细节见 [安装说明](docs/INSTALL.md)。

## 开始阅读

1. 打开“我的书架”，点击“导入小说”，选择 EPUB 或 TXT，核对文字预览和章节数量。
2. 可展开“章节目录”搜索并选择章节，再选择“按住”显示模式，回到其他软件。
3. 按住 **Ctrl + Alt + 空格** 显示文字，松开任一组成键隐藏。Mac 的 Alt 对应 Option（⌥）。
4. 在“文字与布局”中调整样式；需要移动窗口时，解除位置锁定并进入“编辑布局”。完成编辑后恢复鼠标穿透。

macOS 后台快捷键需要输入监控权限。在「系统设置 → 隐私与安全性 → 输入监控」允许正在使用的 FloatLeaf；列表中没有时，点击「＋」添加该 `FloatLeaf.app`，然后退出并重新打开浮页。v0.2.3 会提示缺少权限并提供设置入口。开发启动与安装包的授权不能相互代替，见 [后台快捷键说明](docs/MACOS_BACKGROUND_INPUT.md)。

关闭设置窗口后，浮页仍在运行。通过系统托盘或菜单栏图标重新打开设置，也可以在那里紧急隐藏或退出。

v0.1.1 起，设置页底部提供“日志位置 → 打开日志文件夹”。复现异常后可发送其中的 `.log` 文件，日志不含小说正文或普通打字内容。Windows 备用路径为 `%APPDATA%\io.github.jlauice.floatleaf\logs`，详细步骤见 [复测说明](docs/RELEASE_v0.1.1.md)。

Windows 若仅在管理员权限的程序前台无法显示或翻页，可能是两者权限不一致。已确认拼多多商家工作台场景中，普通浮页收到了快捷键，但无法读取按键状态；同学手动以相同权限运行浮页后，显示、松键隐藏和翻页恢复。优先保持应用以普通权限运行；业务程序必须提高权限时，可先从托盘退出浮页，再手动以管理员身份运行本次会话，结束后退出。不要关闭安全软件或设置永久提权。详情见 [诊断记录](docs/PDD_SHORTCUT_DIAGNOSTICS.md)。

## 功能

- **本地书架**：支持 EPUB 2/3（文件上限 100 MiB）和 UTF-8 / GBK TXT（20 MiB）；EPUB 提取正文上限 20 MiB，以纯文本阅读，不保留图片和原版式。
- **章节导航**：EPUB 读取书内目录，TXT 识别常见章节标题；当前章节、目录搜索、选章跳转和前后章。旧 TXT 书库自动补目录，隐藏时选章保留窗口隐藏。
- **悬浮阅读**：透明、无边框、置顶；实现按住、切换、固定三种显示模式，位置锁定独立控制。
- **阅读操作**：上一页、下一页、自动翻页、暂停和恢复。隐藏或编辑布局时暂停计时，手动翻页重置计时，到结尾停止。
- **翻页间隔**：阅读按钮旁可设 2～60 秒并自动保存；按住模式隐藏时也能开关自动翻页，显示后继续计时。
- **文字排版**：字体、字号、颜色、文字与背景透明度、行高、字间距、宽度和完整行数可调。
- **独立进度**：每本书分别保存阅读位置，重排后保留锚点；导入后移动原文件 不影响阅读。
- **桌面控制**：自定义快捷键、紧急隐藏、鼠标穿透、拖动与调整尺寸、托盘找回窗口位置和单实例运行。

上述功能已实现，系统行为的实测范围见下方“验证状态”。当前不支持 PDF、MOBI、AZW3、加密正文、扫描图片书、云同步或自动更新。

## 默认快捷键

| 操作 | 快捷键 |
| --- | --- |
| 显示 / 隐藏 | `Ctrl + Alt + Space` |
| 上一页 | `Ctrl + Alt + ←` |
| 下一页 | `Ctrl + Alt + →` |
| 自动翻页暂停 / 恢复 | `Ctrl + Alt + P` |
| 紧急隐藏 | `Ctrl + Alt + Esc` |

按住模式下，保持显示组合键，再按方向键翻页或按 P 控制自动阅读。紧急隐藏优先于所有显示模式；完全松键后重新触发显示快捷键即可恢复。修改时，在「快捷键」页点击对应的「录制快捷键」，按下组合键后点击「保存快捷键」。Esc 取消录制，Tab 或点击其他控件离开；「尚未保存」表示改动还没生效。

## 本地数据

正文、阅读进度和设置统一保存在应用数据目录的 `floatleaf.sqlite3`，界面会显示实际路径。导入内容按纯文本处理，不执行其中的 HTML 或脚本。

备份前先从托盘或菜单栏退出浮页，再复制数据库文件。移除书籍只删除书库副本；卸载后如何处理本地书库见 [安装说明](docs/INSTALL.md)。

## 验证状态

| 项目 | 当前证据 |
| --- | --- |
| macOS Apple Silicon | v0.2.3 DMG 校验、复制后启动、单实例与退出通过；已授权独立调试包的后台显示、两种松键顺序和紧急隐藏由用户实测通过；新包授权需另行核对 |
| Windows x64 | v0.2.3 公开安装包在 Windows Server 2025 CI 实际安装，47 项原生断言通过；Windows 11 商家工作台已收到采样失败日志，兼容性仍未解决 |
| 单元与界面测试 | v0.2.3 源码两端各 27 项 Rust、11 项浏览器测试及类型/格式/Clippy 通过；公开 Windows 安装包的 47 项原生回归通过，范围见发布记录 |
| 仍待实机反馈 | Windows 商家工作台、macOS 原始开发进程权限、完整组合键释放顺序、输入焦点、点击穿透、锁屏 / 休眠、多屏 / 缩放与全屏场景 |

本版本用于试用反馈，尚未完成双平台验收。编译成功和窗口属性检查不能代替真实键鼠测试；性能测量的范围和限制见 [性能记录](docs/PERFORMANCE.md)，完整进度见 [验收矩阵](docs/ACCEPTANCE.md)。[章节与 EPUB 验证](docs/CHAPTERS_EPUB.md)。公开发布和安装包回归证据见 [发布记录](docs/GITHUB_RELEASE.md)。[Windows 商家工作台复测](docs/PDD_SHORTCUT_DIAGNOSTICS.md) · [macOS 后台快捷键](docs/MACOS_BACKGROUND_INPUT.md)。

## 开发

按 [开发环境指南](docs/DEVELOPMENT.md) 安装系统依赖。Node 使用 nvm 管理，版本见 `.nvmrc`；包管理器为 pnpm；Rust 必须通过 rustup 管理，版本见 `rust-toolchain.toml`。

```bash
git clone https://github.com/jlau-ice/floatleaf.git
cd floatleaf
nvm use
pnpm install --frozen-lockfile
pnpm env:check
pnpm desktop:dev
```

Windows 请使用适合本机的 Node 版本管理方式，并安装 Visual Studio C++ 构建工具与 Windows SDK。`pnpm desktop:dev` 启动真实桌面应用；`pnpm dev` 仅供网页预览。

| 命令 | 作用 |
| --- | --- |
| `pnpm build` | Vue 类型检查与前端生产构建 |
| `pnpm check` | 类型、Rust 格式与 Clippy 检查 |
| `pnpm format:check` | Vue / TypeScript 格式检查 |
| `pnpm test:rust` | 控制器、阅读状态、存储等单元测试 |
| `pnpm exec playwright install chromium` | 首次准备浏览器测试运行时 |
| `pnpm test:web` | 分页和导入界面测试 |
| `pnpm desktop:build` | 构建当前平台安装包 |

Windows 交叉构建命令见 [构建记录](docs/WINDOWS_BUILD.md)。普通 release 不包含测试探针；`diagnostics` feature 仅供本地验证。

## 反馈与贡献

欢迎通过 [Issues](https://github.com/jlau-ice/floatleaf/issues) 反馈。请提供系统版本、芯片架构、输入法、显示器数量、操作步骤，以及预期和实际表现；报错请保留原文。可以按 [手工验收清单](docs/MANUAL_VALIDATION.md) 逐项测试，无需上传真实小说或私人数据。

提交改动前请阅读 [开发约定](AGENTS.md) 与 [架构说明](docs/ARCHITECTURE.md)，运行相关检查；窗口行为需在真实桌面验证。项目路线与接续记录见 [产品计划](docs/PRODUCT_PLAN.md)、[任务清单](TASKS.md) 和 [交接记录](HANDOFF.md)。

## 开源协议

FloatLeaf 采用 [MIT License](LICENSE)。第三方依赖保留各自的开源协议。
