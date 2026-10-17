# FloatLeaf 开发环境

## 固定版本

| 工具 | 管理方式 | 项目配置 |
| --- | --- | --- |
| Node 22.22.3 | nvm | .nvmrc |
| pnpm 10.33.4 | packageManager | package.json |
| Rust 1.99.1 | rustup | rust-toolchain.toml |
| rustfmt 与 clippy | rustup 组件 | rust-toolchain.toml |

JavaScript 与 Rust 的精确依赖分别由 pnpm-lock.yaml 和 src-tauri/Cargo.lock 固定。升级时显式修改版本并重新验证；不要在 CI 中无条件升级到 latest。

## 本机 macOS

本机为 Apple Silicon，已有 Xcode Command Line Tools 和 MacOSX SDK。Rust 通过官方 rustup 安装，标准目录为 ~/.cargo 与 ~/.rustup，没有修改 .zshrc 或 .zprofile。

```bash
cd ~/jlau-ice/floatleaf
nvm use
source "$HOME/.cargo/env"
rustup show active-toolchain
pnpm install --frozen-lockfile
pnpm env:check
pnpm desktop:dev
```

项目 pnpm 的桌面和 cargo 脚本会为子进程添加 ~/.cargo/bin，因此运行这些脚本不依赖 shell 配置。直接运行 rustup/cargo 时需要 source 上述环境文件，或自行配置 PATH。脚本尊重自定义 CARGO_HOME。

## 新 Windows 开发机

安装 Node 22.22.3 和 pnpm 10.33.4、Visual Studio Build Tools 的 C++ 桌面开发工作负载及 Windows SDK、WebView2 Runtime，并使用官方 rustup 安装 MSVC 工具链。重新打开终端后，在仓库运行以下命令：

```powershell
rustup show
pnpm install --frozen-lockfile
pnpm env:check
pnpm desktop:dev
```

rustup 会根据仓库 rust-toolchain.toml 选择或下载指定工具链。pnpm env:check 仅检查可用工具，不能证明 C++ 链接器、WebView2 或系统窗口行为均正确；必须实际构建和启动。

## 构建与检查

```bash
pnpm build
pnpm check
pnpm tauri build --debug --no-bundle
```

首次编译需下载 crates 并构建系统依赖，耗时明显长于后续增量编译。编译缓存位于 src-tauri/target，前端产物位于 dist，均忽略提交。

GitHub Actions 配置提供 Windows 与 macOS 检查，但只有推送远端并实际运行后才产生 CI 证据；当前不宣称 CI 已通过。Windows 安装包必须在对应环境验证，不以 macOS 结果替代。

## 常见问题

- cargo 找不到：手动 source ~/.cargo/env，或使用项目提供的 pnpm 脚本。
- 端口 1420 已占用：确认是否已有本项目进程；不要结束其他项目服务。
- 只有网页没有桌面：使用 pnpm desktop:dev；pnpm dev 不启动 Rust。
- 快捷键无效：检查设置中的注册错误。默认 Ctrl+Alt+Space，紧急隐藏后须全部释放再重新按下；网页模式不支持系统快捷键。
- 正式分发：macOS 签名、公证与 Windows 签名在 P3 配置，凭证不入库。

官方前置依赖：https://v2.tauri.app/start/prerequisites/

## 代码与数据约定

Vue/TypeScript 使用 Prettier（`pnpm format:check`），Rust 使用 rustfmt 与 Clippy。Vue 组件 PascalCase、普通 TS 文件 kebab-case、Rust 模块 snake_case。实现以具体功能为单位，避免无需求的抽象层和重复校验。

本地小说、进度和设置写入 `floatleaf.sqlite3`；SQLite 通过 rusqlite bundled 编译进应用，不要求用户安装数据库服务。设置作为一行 JSON 字段保存在 SQLite 内，没有独立 JSON 存储文件。运行期的阅读状态由 Rust 控制器持有，前端通过 Vue 组合函数订阅，没有额外 Pinia 状态副本。

`pnpm test:rust` 覆盖显示/阅读状态和 SQLite 持久化；`pnpm test:web` 通过隔离 Chrome 实测分页。后者使用本机 Chrome，CI 配置下载 Chromium。原生使用 `scripts/native-reading-smoke.mjs`，需显式开启 debug 探针，详细步骤见 P1_READ_VERIFICATION.md。CI 尚未远端运行。
