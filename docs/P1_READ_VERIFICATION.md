# P1 阅读闭环验收记录

日期：2026-10-17。环境：macOS 27.0.1 arm64；Tauri 2.12.4、WebKit；Rust 1.99.1、rusqlite 0.40.2（bundled SQLite）。Windows 仍无实机证据。

## 实现范围

- 原生 TXT 文件选择、导入前预览、自动/手选 UTF-8 与 GBK；GBK 提示核对文字，空文件/无效编码明确报错。20 MiB 上限。
- `floatleaf.sqlite3` 的 `books` 表包含正文、内容 SHA-256、标题、编码、UTF-16 长度和阅读位置；`app_state` 保存当前书及设置。直接 SQL，无 ORM。SQLite 默认回滚日志，导入事务提交；相同内容去重且保留进度。
- 进度最多每秒刷新，切书与正常退出立即保存。原始 TXT 导入后可以移动；移除只删除数据库中的书籍。
- 实际 WebView 字体/宽度/行数测量分页，按 grapheme 边界切分，缓存有限相邻页面。Rust/JS 统一 UTF-16 偏移，改排版保持锚点。
- 原生手动/自动翻页。隐藏和编辑时暂停倒计时；重新显示继续剩余时间；手动翻页重置；末尾停止。仅可见时接受翻页。
- 字体、字号、颜色、文字/背景不透明度、行高、字间距、宽度、1～12 行、自动间隔均可保存；阅读窗口高度随完整行数变化。
- 书架、导入预览、进度、排版及显示模式界面均接真实 IPC，没有业务占位按钮。

## 已执行

1. Rust 单元测试 14 项：显示状态、释放顺序、紧急隐藏、自动翻页时序、旧分页回报、UTF-16 边界、解码、SQLite 事务回滚、重复导入、各书进度、设置、删除和重启。
2. 隔离 Chrome 3 项 DOM 测试：中英文/emoji/组合字符/空行/长词连续分页，重排前后锚点和前一页衔接，数百万字符文本只测量有限 DOM 范围并清理节点。大文本布局一次诊断值 5 ms，仅是该机 Chrome 排版时间，不是系统显隐延迟或性能目标通过。
3. **实际 macOS WebView + Rust + SQLite**：`native-reading-smoke.mjs` 通过 UTF-8/GBK 合成 TXT 导入、重复内容去重、前后翻页、排版重排、两本书独立进度、隐藏暂停、恢复计时、手动重置、紧急隐藏和末尾停止。静止页面的 revision 保持不变，复验消除了分页回报循环。
4. 真实退出、移动原始合成 TXT、重启同一数据库：正文仍可排版，当前书锚点 3913 恢复，另一本书位置 20 保留，设置恢复，启动保持隐藏。

本地证据：`.local/p1-final/ui-result.json`、`restart-result.json`、`status.json`、测试数据库；均不纳入 Git。没有使用真实小说或用户数据。

## 复现

```bash
pnpm test:rust
pnpm test:web
pnpm tauri build --debug --no-bundle
mkdir -p .local/reading-run
FLOATLEAF_DATA_DIR="$PWD/.local/reading-run/data" \
FLOATLEAF_PROBE_DIR="$PWD/.local/reading-run" \
src-tauri/target/debug/floatleaf
# 另一个终端；每次完整测试使用新的测试目录
node scripts/native-reading-smoke.mjs .local/reading-run
```

`FLOATLEAF_DATA_DIR` 和探针文件命令只在 debug 构建启用。测试脚本通过本应用 WebView 发真实 IPC，调用相同导入和阅读业务，不伪造分页结果或跳过 SQLite。它不模拟物理键鼠。

## 尚待验收

原生文件选择器的手工点击、按住期间真实翻页组合键、不同平台字体视觉边界、物理点击穿透、锁屏睡眠、多屏及 Windows 实机保留待测。P0 的输入与平台限制仍适用，不能把本阶段的 IPC 验证记为双平台系统快捷键通过。

完整 CPU/内存趋势和屏幕端到端显隐 P95 尚未测量。后续还需编辑尺寸、屏幕恢复、快捷键自定义、试用安装包和使用说明。
