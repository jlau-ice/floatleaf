# FloatLeaf 开发交接

更新日期：2026-10-17。当前阶段：初始化完成，下一项为 P0-01。

## 已完成

- 项目命名：浮页 FloatLeaf；目录 `/Users/ice/jlau-ice/floatleaf`。
- 本地 Git 仓库，分支 `main`；初始提交内容包括骨架、依赖锁文件、文档和检查工作流。提交号使用 `git log -1 --oneline` 查看。
- Tauri 2 + Vue 3 + TypeScript + Pinia 项目，简洁启动页和真实 `runtime_info` IPC 命令。
- Rust global-shortcut 插件已接入 builder；尚未注册快捷键，也没有 overlay 窗口。
- 官方 rustup 安装与项目固定工具链：Rust 1.99.1、rustfmt、clippy。未修改 shell 配置；Node 22.22.3，pnpm 10.33.4。
- 全部依赖已安装。实际锁定：Tauri Rust/API 2.12.4、CLI 2.12.3、global-shortcut 2.4.1、Vue 3.5.44、Pinia 3.0.4、Vite 8.3.4、TypeScript 6.0.3。
- 文档：产品计划、架构约定、开发环境、验收矩阵、TASKS.md、AGENTS.md、六页 Word 计划书。
- Windows/macOS GitHub Actions 检查配置已写入，尚未在远端运行。

## 本次验证

环境：macOS 27.0.1，Apple Silicon arm64，Command Line Tools SDK。

| 检查 | 结果 |
| --- | --- |
| pnpm env:check | 通过；rustup 正确读取 rust-toolchain.toml |
| pnpm build | 通过；Vue 类型检查与 Vite 构建 |
| pnpm check | 通过；类型、rustfmt、Clippy all-targets，警告视为错误 |
| pnpm tauri build --debug --no-bundle | 通过；生成本机 arm64 Mach-O 程序 |
| 桌面启动冒烟 | 二进制启动后持续运行 6 秒，无输出错误；随后仅终止测试进程 |
| 网页预览 | 隔离 Chrome 检查标题、预览状态、禁用原生动作、640px 无横向溢出、无脚本错误；截图已目视检查 |
| Word 计划书 | 已渲染六页，逐页检查中文、表格、分页和参考链接 |

桌面启动冒烟只证明未立即退出，不等同于原生 IPC 按钮或悬浮功能验收。原生连接可启动后点击“检查桌面连接”手动确认。

## 尚未实现或验证

透明悬浮、按住显示、释放隐藏、鼠标穿透、托盘、TXT 导入、分页、样式与进度存储、自动翻页均待后续任务实现。Windows 未实机编译或运行；正式签名、公证、安装包发布和 CI 尚未完成。没有 Git 远端，也没有推送。

## 立即开始

```bash
cd /Users/ice/jlau-ice/floatleaf
nvm use
pnpm env:check
pnpm desktop:dev
```

项目脚本会为子进程补充 rustup PATH。直接使用 cargo 或 rustup 时先执行 `source "$HOME/.cargo/env"`。前端依赖已安装，换机器或重新检出时执行 `pnpm install --frozen-lockfile`。

阅读 TASKS.md，从 **P0-01 透明阅读窗口** 开始，然后完成 P0-02 按键释放语义和 P0-03 紧急隐藏。双平台 P0 验证通过后再进入 P1。实现时先读 docs/ARCHITECTURE.md，确保 Rust 直接控制显隐，不抢焦点，不仅用网页 keyup。

## 继续工作的提示词

> 请先阅读 AGENTS.md、HANDOFF.md、TASKS.md 和 docs/ARCHITECTURE.md，检查 Git 状态，然后从 P0-01 开始实现透明悬浮窗口。保持初始化以外的需求按任务逐项完成，区分 macOS 验证结果与 Windows 待测项。
