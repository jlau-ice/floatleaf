# 浮页 FloatLeaf

像桌面歌词一样显示小说的离线阅读工具，计划支持 Windows 和 macOS。按住快捷键显示文字，松开隐藏；字体、透明度、显示行数及翻页方式可调。

当前已实现 **Tauri 2 + Vue 3 + TypeScript + Rust + SQLite 的 TXT 阅读闭环**：导入预览、UTF-8/GBK、书架、分页、独立进度、文字样式、自动翻页及原生悬浮控制。macOS 原生阅读流程已验证，Windows、真实键鼠和多屏等验收仍在推进。

## 开始开发

```bash
cd ~/jlau-ice/floatleaf
nvm use
pnpm install --frozen-lockfile
pnpm env:check
pnpm desktop:dev
```

Rust 由 rustup 管理，rust-toolchain.toml 固定项目版本并声明 rustfmt/clippy。项目启动脚本自动为子进程加入 rustup shims；手动使用 cargo 时先运行 `source "$HOME/.cargo/env"`。

## 常用命令

| 命令 | 作用 |
| --- | --- |
| pnpm desktop:dev | 启动原生桌面应用与前端热更新 |
| pnpm dev | 浏览器预览，不能验证全局快捷键和系统窗口行为 |
| pnpm env:check | 检查 Node、rustup、Rust 组件和系统工具 |
| pnpm build | Vue 类型检查与前端生产构建 |
| pnpm test:rust | 控制器、阅读状态和 SQLite 单元测试 |
| pnpm test:web | 实际浏览器分页测试 |
| pnpm format:check | Vue/TS 格式检查 |
| pnpm check | 类型、Rust 格式和 Clippy 检查 |
| pnpm tauri build --debug --no-bundle | 编译本机桌面程序，不生成分发安装包 |
| pnpm desktop:build | 构建当前平台安装包，正式发布另需签名配置 |

## 接续入口

1. [HANDOFF.md](HANDOFF.md)：当前环境、验证结果和下一步。
2. [TASKS.md](TASKS.md)：带编号、依赖与验收条件的任务。
3. [产品计划](docs/PRODUCT_PLAN.md)：需求、交互、选型、排期和参考资料。
4. [架构约定](docs/ARCHITECTURE.md)、[环境指南](docs/DEVELOPMENT.md)、[验收矩阵](docs/ACCEPTANCE.md)。
5. [Word 计划书](docs/FloatLeaf-项目计划书.docx)：可分享的计划书。

P0 macOS 原生窗口探针已运行，完整键鼠与双平台实机验收仍待补。具体证据见 [原生验证记录](docs/P0_NATIVE_VERIFICATION.md)，阅读闭环证据见 [P1 验收记录](docs/P1_READ_VERIFICATION.md)。下一步完善多屏、尺寸编辑、快捷键设置和试用构建。

## 分发前提

macOS 完全透明的 Tauri 实现使用私有 API，当前计划为独立安装包分发。初始应用图标沿用生成器占位资源，品牌图标在发布前替换。没有远端仓库、发布凭证或服务端依赖。
