# P2 布局与快捷键验证

2026-10-17，macOS 27.0.1 (26A434)，arm64，内建屏 3420×2224 / scale 2，工作区 3420×2014。Windows 尚无编译和实机结果。

## 实现

- 编辑态右下角拖动调整宽度和完整行数；修改尺寸仍保留 UTF-16 阅读锚点。阅读态恢复鼠标穿透，位置锁定独立于显示方式。
- SQLite `window_layout` 单行保存显示器标识和相对工作区的逻辑坐标。移动事件更新内存，约一秒合并落盘，正常退出立即保存。
- 根据显示器工作区限制位置、宽度和完整行数；显示器变化、缩放变化触发布局恢复。找不到原显示器时回主屏。托盘及设置均有“找回位置”，不会绕过紧急隐藏。
- 可录入显示、紧急隐藏、上/下一页和自动翻页组合键；禁止重复与普通裸字母。新组合先注册，失败时旧组合不变；成功持久化后清除旧注册，并隐藏窗口等待新手势。
- 修复保存为固定模式后重启、再次点击“固定”不能显示的问题；明确选择模式与单纯修改样式分开处理。
- 翻页和自动翻页键按下去重，释放和原生键状态采样清理，避免长按重复切换暂停状态。

## 已验证

- `pnpm check`、`pnpm format:check`、`pnpm tauri build --debug --no-bundle` 通过。
- 17 项 Rust 测试通过，包括较小工作区约束、旧设置字段默认值、布局持久化和快捷键别名重复检测。
- 实际运行原生程序，重跑 `scripts/native-reading-smoke.mjs`：P1 阅读闭环通过。
- `scripts/native-layout-smoke.mjs` 通过：真实独立进程占用 Ctrl+Alt+F12 后注册失败并保留旧配置；修改到 Ctrl+Alt+R 后 SQLite 持久化且紧急隐藏；恢复原快捷键；尺寸 780×156 逻辑像素对应 1560×312 物理像素、4 个完整行；锚点未变；移动到 (240,220) 物理坐标后 SQLite 保存相对位置 (120,72)；找回位置后窗口处于工作区。
- 测试使用本应用真实 WebView IPC 和系统窗口接口，不等同于物理键鼠。未向其他应用注入输入。
- 100 次显隐、不激活、穿透属性和编辑态切换回归通过。真实重启恢复 (240,220) 物理位置和 780×156 逻辑尺寸通过；由此发现并修复 AppKit 调整高度导致的顶部漂移，重新跑布局测试通过。
- 将隔离库中显示器标识改成不存在的测试屏、位置改为越界、字号 48 / 行高 2.2 / 12 行，重启后回主屏工作区并约束为 9 个完整行，默认保持隐藏。此项是保存数据恢复模拟，不是外接屏拔插实测。

本地证据 `.local/p2/layout-result.json`、`ui-result.json` 和 SQLite，仅含合成小说，不纳入 Git。

## macOS 注册差异

锁定依赖 global-hotkey 0.8.0 使用 Carbon 共享注册。实际测试发现，即使另一进程独占同一组合，共享注册仍可能成功。因此 macOS 新增注册前的 Carbon 独占检查，现已捕获该场景；系统错误码 -9878 会显示为可理解的注册失败提示。Carbon 仍允许其他应用共享注册；事件拦截工具和部分系统保留组合不保证报告冲突。界面明确提示，如同时响应或系统拦截，应换一组组合键。不能声称能检测所有应用冲突。

## 复现

```bash
pnpm tauri build --debug --no-bundle
clang -fobjc-arc -framework AppKit -framework Carbon scripts/native-fixture.m -o .local/native-fixture
# 两个独立终端分别运行；测试目录务必隔离，不使用真实小说
FLOATLEAF_PROBE_DIR="$PWD/.local/p2" FLOATLEAF_DATA_DIR="$PWD/.local/p2/data" src-tauri/target/debug/floatleaf
.local/native-fixture --hotkey
# 阅读测试先导入合成样本，再运行布局测试
node scripts/native-reading-smoke.mjs .local/p2
node scripts/native-layout-smoke.mjs .local/p2
```

## 待验证

实际拖动手柄、真实组合键释放和点击穿透；多显示器不同缩放、外接屏拔插；真实锁屏和睡眠；全屏/虚拟桌面；全部 Windows 场景。软件模拟断开显示器的保存数据不能替代真实拔插。
