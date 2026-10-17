# FloatLeaf v0.1.0 · 首个公开试用版

浮页是一款离线 TXT 悬浮阅读工具：按住快捷键显示文字，松开隐藏。源码以 MIT 协议开放，本次为 Pre-release，欢迎反馈使用情况。

## 下载安装

| 平台 | 文件 | 说明 |
| --- | --- | --- |
| macOS Apple Silicon（M 系列） | `FloatLeaf_0.1.0_aarch64.dmg` | 打开后拖到“应用程序”；最低系统配置为 macOS 13.0，本轮测试系统为 27.0.1 |
| Windows x64 | `FloatLeaf_0.1.0_x64-setup.exe` | 当前用户安装；本轮面向 Windows 11 x64 |
| Windows 完整试用包 | `FloatLeaf_0.1.0_windows-x64_trial.zip` | 含同一安装器、试用说明、UTF-8/GBK 合成示例、反馈表和包内校验清单 |

另提供 Word 试用说明、`LICENSE.txt` 和 `SHA256SUMS.txt`。请从本 Release 的 Assets 下载，GitHub 自动生成的 Source code 是源码，不是安装包。Intel Mac 和 Windows ARM64 暂无对应安装包。

Windows 需要 WebView2：已有运行时可直接使用，缺少时安装器会联网下载；日常阅读不需要网络。macOS 包为 ad-hoc 签名且未公证，Windows 包未正式签名；系统可能显示来源未验证或未知发布者提示，请确认文件来源后按系统提示处理。

## 已实现

- UTF-8 / 常见 GBK TXT 导入预览、书架、独立阅读进度，本地 SQLite 保存正文和设置。
- 透明置顶悬浮窗口，按住 / 切换 / 固定显示，独立位置锁定与布局编辑。
- 字体、字号、颜色、透明度、行距、字距、宽度和完整行数调整。
- 手动与自动翻页；隐藏时暂停、手动翻页重置计时、到结尾停止。
- 自定义全局快捷键、紧急隐藏、托盘 / 菜单栏入口、找回位置与单实例运行。

默认显示快捷键为 **Ctrl + Alt + Space**，紧急隐藏为 **Ctrl + Alt + Esc**；Mac 上 Alt 对应 Option。完整操作见 [试用说明](https://github.com/jlau-ice/floatleaf/blob/main/docs/TRIAL_GUIDE.md)。

## 验证范围与已知限制

- macOS：DMG 中应用的实际启动、重复启动和退出已验证；原生窗口属性与阅读闭环回归通过。
- Windows：在 macOS 完成 MSVC release 交叉编译、Windows 目标 Clippy 与 NSIS 包完整性核验；尚未在 Windows 实机安装或运行。
- 本机已有 17 项 Rust 测试和 5 项浏览器测试通过。这些检查不代替系统键鼠验收。
- 两端真实组合键释放顺序、输入焦点、点击穿透、锁屏 / 休眠、多屏 / 缩放与全屏场景仍待反馈，不能宣称双平台验收完成。
- 当前支持 TXT，暂不支持 EPUB、PDF、云同步和自动更新。性能数据和测量范围见 [性能记录](https://github.com/jlau-ice/floatleaf/blob/main/docs/PERFORMANCE.md)。

两份安装器沿用已核验的 0.1.0 构建，应用业务代码基于 `6e341c5`；发布准备只更新文档和协议，没有修改业务代码或依赖锁文件。

## 反馈

请通过 [Issues](https://github.com/jlau-ice/floatleaf/issues) 提供系统版本、芯片架构、输入法、显示器数量、复现步骤、预期与实际结果。优先测试导入、进度恢复、释放隐藏、不抢焦点、点击穿透和紧急隐藏。无需上传真实小说或私人数据。
