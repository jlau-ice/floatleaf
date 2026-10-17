# FloatLeaf v0.2.2 · 快捷键录制更清楚

Windows x64 和 macOS Apple Silicon 同步更新。

- 每项快捷键都有明确的「录制快捷键」按钮，点击后高亮显示「正在录制」。
- 录好后提示「尚未保存」，保存成功或失败在操作区域反馈。Esc 取消、Tab 或点击其他控件离开录制。
- 包含 Windows 暖白原生标题栏、EPUB/TXT 章节目录、自动翻页间隔和日志入口。

先从托盘或菜单栏退出旧版，再安装新包；无需删除书库。升级后在设置页底部「日志位置」确认版本为 0.2.2。

请复测快捷键录制、保存后实际触发、不同松键顺序和紧急隐藏；遇到问题可从「日志位置」取回 .log 文件。Windows 11 物理键鼠、不同缩放、多屏与系统场景仍待用户设备验收。

验证结果：

- [源码双平台 CI](https://github.com/jlau-ice/floatleaf/actions/runs/39042534515) 通过，两端各 27 项 Rust / 10 项浏览器测试及类型、格式、Clippy 检查通过。
- [Windows 安装包回归](https://github.com/jlau-ice/floatleaf/actions/runs/39044669420) 实际下载安装本页 EXE；Windows Server 2025 的 43 项系统合成按键、显隐/焦点、窗口控制与标题栏像素检查通过。
- macOS DMG 校验、复制应用后的实际启动、单实例与正常退出通过。
- 六个发布附件公开下载后与本地产物逐字节及 SHA-256 核对一致。

Windows 云端系统合成输入检查不代替 Windows 11 物理键鼠验收。两端安装包仍未使用正式发布证书，macOS 使用 ad-hoc 签名且未公证。
