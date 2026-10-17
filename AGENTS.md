# FloatLeaf 开发约定

## 开始工作

- 先读 README.md、HANDOFF.md、TASKS.md，再读 docs/PRODUCT_PLAN.md 和 docs/ARCHITECTURE.md。
- 修改前检查当前分支、git status 和已有变更；保留用户未提交的工作。
- 文档主要使用中文，代码标识使用英文。Vue 单文件组件使用 PascalCase，普通 TS 文件使用 kebab-case，Rust 模块使用 snake_case。

## 环境和验证

- Node 由 nvm 管理，版本见 .nvmrc；包管理器为 pnpm，版本见 package.json。
- Rust 必须由 rustup 管理，版本和组件见 rust-toolchain.toml；不要改成 Homebrew Rust。
- 使用 pnpm desktop:dev 启动桌面，pnpm dev 仅供网页预览。脚本会为子进程补充 rustup PATH，不修改全局 shell 配置。
- 验证命令：pnpm env:check、pnpm build、pnpm check。改动窗口/打包配置后增加 pnpm tauri build --debug --no-bundle。
- Windows/macOS 体验分别验收；网页通过、Rust 编译通过、安装包构建通过不能代替实机快捷键/焦点/穿透测试。
- 不为文档或可逆的简单样式变化增加无意义测试。阅读控制器、分页和存储实现后增加覆盖关键边界的测试。

## 代码与存储

- 存储统一使用最简单的嵌入式 SQLite：小说正文、进度与设置保存在本地数据库，直接 SQL，不另建 JSON/正文文件存储，不引入 ORM 或通用仓储抽象。
- 遵循现有开发规范，避免冗余实现、重复校验和假设性防御编程；保留产品明确要求的状态边界与正常错误提示。
- Vue/TypeScript 使用 Prettier；Rust 使用 rustfmt 和 Clippy。

## 产品边界

- 本地离线优先；首版 TXT；透明置顶、按住显示和释放隐藏为首要能力。
- 阅读窗口不抢焦点，隐藏时暂停自动翻页。紧急隐藏高于固定显示和旧快捷键事件。
- Rust 持有系统窗口的权威状态。按键事件直接控制原生窗口，不只依赖前端 keyup。
- macOS 完全透明按独立安装包路线实现；App Store 路线改变时先更新架构决策。
- 快捷键、全屏和虚拟桌面的系统差异必须实测并写清限制，不凭接口名称声称已支持。

## 交付记录

- 每个任务完成后更新 TASKS.md 和 HANDOFF.md，附执行命令与结果。
- 单独标注已实现、已编译、已实测、待验证。无 Windows 实机证据不得勾选双平台验收。
- 依赖锁文件纳入 Git；不得提交 node_modules、target、.env、证书私钥、真实小说或用户数据。
- 当前仅本地仓库；远端创建、推送和发布按用户后续指示执行。
