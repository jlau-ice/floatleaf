# GitHub 首次公开发布记录

日期：2026-10-17。按用户明确授权创建公开仓库、推送源码，并发布两端试用安装包。

- 仓库：[jlau-ice/floatleaf](https://github.com/jlau-ice/floatleaf)，默认分支 `main`，协议 MIT。
- 发布：[v0.1.0](https://github.com/jlau-ice/floatleaf/releases/tag/v0.1.0)，已公开，标记为 Pre-release；发布时间为北京时间 2026-10-17 14:51。
- 版本标签指向 `149e9ffa4acfc46d7e41b24671fc231bb778430a`；两份安装包沿用此前核验的业务代码 `6e341c5`，发布准备仅更新说明与协议。
- 安装包的正式签名与公证仍未完成；发布为公开试用版不代表双平台验收完成。

## 上传文件与核验

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| FloatLeaf_0.1.0_aarch64.dmg | 2,986,687 | `89da12640a036d33e42c744d33f5d0af54887906053432c2a104d280fb7f0ddf` |
| FloatLeaf_0.1.0_x64-setup.exe | 2,374,682 | `7fa5165cc42fb3a377d8ad8e3dd7af6d993a40d1589402c382a916e866488cc1` |
| FloatLeaf_0.1.0_windows-x64_trial.zip | 2,395,100 | `7947467140c79cfc84e54d3e24d979639be45a1f45d81af436e93b5748c570ff` |
| FloatLeaf-Trial-Guide.docx | 40,238 | `af2e2ee7f5b66f9de47021114187452bc4f88baeaf8a14797cae3d87c7f3783a` |
| LICENSE.txt | 1,065 | `f527b3b2027179390d943c50ca81edc52a78b8258ebc007288598825a207c123` |
| SHA256SUMS.txt | 465 | `48e5a7218f9e161daea065fafc68a6f8b92c48b57f2816a393531b691f2eb7b3` |

使用 `gh release create --draft --prerelease` 上传，随后 `gh release download` 下载全部六个附件并逐一比较 SHA-256，全部与本地文件一致；再使用 `gh release edit --draft=false --prerelease --latest=false` 发布。`gh release view` 确认 `isDraft=false`、`isPrerelease=true` 与六个附件。

未登录请求访问公开仓库、发布页、DMG、EXE 和试用 ZIP 均返回 HTTP 200。`git ls-remote origin refs/heads/main` 与首次推送提交一致。下载核验明细保存在忽略目录 `.local/github-release-verification.json`，发布附件保存在 `.local/github-release/`。

完整阶段历史与依赖锁文件已推送；安装包通过 Release 分发，未纳入 Git。已检查历史文件范围和已知凭据模式，未提交依赖目录、编译缓存、真实小说、用户数据库或密钥。

## 云端检查

首轮 [Check](https://github.com/jlau-ice/floatleaf/actions/runs/39011286910) 的 Windows runner 通过环境检查、前端构建和 `pnpm check`，随后 Prettier 对 16 个文本文件报告格式问题；本机相同内容检查通过。仓库原本没有换行约定，新增 `.gitattributes` 固定文本 LF，避免 Windows checkout 转换为 CRLF。使用 `git -c core.autocrlf=true checkout-index` 模拟检出，16 个格式检查目标全部保持 LF；本机 `pnpm format:check` 通过。

修复后的两端自动检查待复跑确认。CI 的编译、单元测试和浏览器测试不替代 Windows/macOS 实际安装、系统快捷键、输入焦点、点击穿透和多屏验收；这些仍按 MANUAL_VALIDATION.md 收集试用反馈。
