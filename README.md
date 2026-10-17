# 浮页 FloatLeaf

像桌面歌词一样显示小说的离线阅读工具，计划支持 Windows 和 macOS。按住快捷键显示文字，松开隐藏；字体、透明度、显示行数及翻页方式可调。

当前已实现 **Tauri 2 + Vue 3 + TypeScript + Rust + SQLite 的 TXT 阅读闭环**：导入预览、UTF-8/GBK、书架、分页、独立进度、文字样式、自动翻页及原生悬浮控制。macOS 原生阅读流程已验证，Windows、真实键鼠和多屏等验收仍在推进。

使用方法见 [使用说明](docs/USER_GUIDE.md)，转发给试用者可使用 [试用说明](docs/TRIAL_GUIDE.md)。本地试用、Windows 构建与签名边界见 [安装说明](docs/INSTALL.md)。已接入编辑尺寸、位置恢复、自定义快捷键和单实例运行。

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

P0 macOS 原生窗口探针已运行，完整键鼠与双平台实机验收仍待补。具体证据见 [原生验证记录](docs/P0_NATIVE_VERIFICATION.md)、[P1 阅读验证](docs/P1_READ_VERIFICATION.md)、[P2 布局与快捷键验证](docs/P2_LAYOUT_VERIFICATION.md)、[P3 试用交付](docs/P3_DELIVERY_VERIFICATION.md)和[性能实测](docs/PERFORMANCE.md)。

## 分发前提

macOS 完全透明的 Tauri 实现使用私有 API，采用独立安装包路线。品牌图标源文件为 `assets/floatleaf-icon.svg`。macOS 本地包使用 ad-hoc 签名，正式签名与公证另需授权；Windows x64 未签名 NSIS 包已交叉构建，仍待 Windows 实测，见 [Windows 构建记录](docs/WINDOWS_BUILD.md)。没有远端仓库、发布凭证或服务端依赖。
