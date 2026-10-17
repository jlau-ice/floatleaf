# Windows x64 试用包构建记录

日期：2026-10-17。应用代码基于 `6e341c5`，本次未修改业务代码或依赖锁文件。按用户要求，从 macOS 交叉构建 Windows 安装包，交由 Windows 同学实机试用。本记录证明构建和静态核验结果，不证明 Windows 安装或运行通过。

## 构建环境与复现

- 主机：macOS 27.0.1 arm64，Apple M5。
- Rust：rustup 管理的 1.99.1，增加 `x86_64-pc-windows-msvc` 目标。
- 工具：cargo-xwin 0.23.1；LLVM/LLD 23.1.2；NSIS 3.13；7-Zip 26.03。
- xwin 缓存：项目 `.local/xwin`，含 MSVC 14.44 系列 CRT 和 Windows SDK 10.0.26100；不入 Git。

采用 [Tauri 官方交叉构建路径](https://v2.tauri.app/distribute/windows-installer/#build-windows-apps-on-linux-and-macos)：

```bash
# 一次性准备，Rust 保持由 rustup 管理
brew install llvm lld nsis
rustup target add x86_64-pc-windows-msvc --toolchain 1.99.1
cargo install --locked cargo-xwin --version 0.23.1

# 仓库根目录，PATH 仅作用于该进程
CI=true PATH="$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$PATH" \
  XWIN_CACHE_DIR="$PWD/.local/xwin" \
  pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis

PATH="$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$PATH" \
  XWIN_CACHE_DIR="$PWD/.local/xwin" RUSTFLAGS='-C target-feature=+crt-static' \
  cargo-xwin clippy --manifest-path src-tauri/Cargo.toml --locked --release \
    --target x86_64-pc-windows-msvc --all-targets -- -D warnings
```

原始构建输出在 `.local/windows-build.log`，Clippy 输出在 `.local/windows-clippy.log`。Homebrew 下载 LLVM 曾报告传输中断，但最终工具安装成功，版本命令与后续编译均通过；未修改全局 shell 配置或已有 Homebrew tap 信任设置。

## 核验结果

| 项目 | 结果 |
| --- | --- |
| 前端类型检查与生产构建 | 通过，使用现有 pnpm 锁文件 |
| Windows MSVC release 交叉编译 | 通过，含原生窗口 C 适配与 bundled SQLite |
| Windows 目标 Clippy all-targets | 通过，`-D warnings`；不等于执行 Windows 测试 |
| Rust 格式 | `pnpm rust:fmt` 通过 |
| NSIS 安装包 | 生成成功；当前用户安装，简中/英文，缺少 WebView2 时下载运行时 |
| PE 与依赖 | 包内主程序为 AMD64 Windows GUI；未导入 VCRUNTIME140.dll/MSVCP140.dll，仍需要系统 UCRT 与 WebView2 |
| 包内文件 | `7zz t` 通过；解出主程序，与构建输出仅存在 Tauri 预期的 `UNK` → `NSS` 包类型标记差异 |
| 诊断入口 | 包内程序未包含文件命令与测试目录覆盖标记 |
| 试用 ZIP | CRC 检查通过，包含 7 个文件；没有书库、用户数据、SDK 或编译缓存 |
| Windows 实机 | 尚未执行，待试用反馈 |

构建存在 SDK 静态库缺少 PDB 的 LNK4099 调试信息警告，以及非 Windows 主机不支持 NSIS `-OUTPUTCHARSET` 的提示。链接与安装包生成成功；没有隐去这些提示，也未据此声称运行正常。NSIS 安装器外壳为 32 位，包内应用为 64 位。程序和安装包均未正式签名。

## 产物

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| `FloatLeaf_0.1.0_x64-setup.exe` | 2,374,682 | `7fa5165cc42fb3a377d8ad8e3dd7af6d993a40d1589402c382a916e866488cc1` |
| `FloatLeaf_0.1.0_windows-x64_trial.zip` | 2,395,100 | `7947467140c79cfc84e54d3e24d979639be45a1f45d81af436e93b5748c570ff` |

两份文件均位于 `.local/release/`。包内主程序 SHA-256 为 `11fd7155c58e91282a4b6e03d79eeb5f649db200739a040597bcc66cecc822f5`。

ZIP 包含安装程序、三页 Word 试用说明、先读我、UTF-8/GBK 合成示例、反馈表和 SHA256SUMS。Word 已使用 bundled LibreOffice 渲染并检查全部三页；Markdown 原文见 TRIAL_GUIDE.md。可直接转发 ZIP，由同学解压后运行安装程序。未上传或公开发布。

下一步由 Windows 同学验证安装、TXT 导入、组合键释放、焦点、点击穿透、紧急隐藏和进度恢复；完整步骤见 MANUAL_VALIDATION.md。收到结果后再更新 ACCEPTANCE.md，双平台总目标仍未完成。
