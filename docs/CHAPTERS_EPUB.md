# v0.2.0 章节与 EPUB

2026-10-17：用户确认优先支持 EPUB / TXT，目录选择是必要阅读功能。

## 行为

- EPUB 2 读取 NCX，EPUB 3 读取导航文档，正文按 OPF spine 顺序拼接；目录文件不混入正文。支持同一个 XHTML 中多个章节、嵌套目录、相对路径、百分号编码路径及章节 ID。
- 只提取文本，不执行 HTML/JavaScript、不加载外部链接、不解压到用户目录；图片、样式和原书版式不保留。正文加密、文件损坏或只有扫描图片时提示无法导入。
- 没有可用目录时按正文文件生成分节，并在导入预览中说明。目录目标不存在时保留能定位的章节；TXT 没有独立标题时不虚构章节。
- TXT 识别独立行的“第 X 章/回/节/卷/部/篇”、Chapter / Part / Book 编号、序章、楔子、尾声等常见标题。保留原文，不移除标题；不是任意 TXT 的通用语义识别器。
- 设置页显示当前章节、上一章/下一章；展开目录可搜索标题并选择。关闭目录时不创建章节列表 DOM。
- 主动选章可在隐藏时执行，只改变阅读锚点，保留显示模式、紧急隐藏与自动阅读开关；重置翻页间隔，隐藏期间仍冻结计时。跳章立即保存进度，重新排版保留位置。

## 存储与边界

继续使用单个 `floatleaf.sqlite3`，无 ORM 或额外小说文件。`chapters` 表按 `book_id, position` 保存标题、UTF-16 锚点和目录层级，随书籍删除级联清理。数据库 `user_version=2`；旧 TXT 书库在事务中补识别章节，原正文、进度、当前书籍与设置保持不变。重复导入保留进度和已保存目录。

TXT 上限 20 MiB；EPUB 压缩文件上限 100 MiB，单个正文/索引与总提取正文上限 20 MiB。ZIP 读取也受实际展开字节上限约束，不只依赖压缩包声明大小。这里只提供纯文字悬浮阅读，暂不支持 PDF、MOBI、AZW3、OCR 或加密正文。

## 验证

- `pnpm env:check`：Node 22.22.3 / pnpm 10.33.4 / rustup Rust 1.99.1，通过。
- `pnpm build`、`pnpm check`、`pnpm format:check`：通过。
- 27 项 Rust 测试通过：EPUB 2/3、嵌套目录、同文件片段、百分号路径、emoji 偏移、正文顺序、无目录回退、加密与展开大小拒绝；旧书库升级、重复导入、源文件移除及重启、目录级联清理；跳章计时和旧分页回报隔离。
- 7 项浏览器测试通过：分页、异步编码预览、模态错误、隐藏时章节导航、搜索结果仍使用原目录索引。
- macOS 27.0.1 arm64 debug 程序的真实 WKWebView → IPC → SQLite 流程通过，导入 UTF-8/GBK TXT、EPUB 2、EPUB 3，实际点击搜索结果和上一章，隐藏不显示/不计时，字号重排保留章节锚点。`scripts/native-chapter-smoke.mjs`，证据 `.local/chapters-020/native/ui-result.json`。首次启动方式未保留测试进程导致脚本等待超时，保持进程运行后完整复跑通过。
- 最终界面真实重启后，已移走源文件，仍恢复 EPUB 3 的 3 个章节、第二章锚点 937，启动保持隐藏。设置页和悬浮正文截图检查通过，正文从“第二章 山路”开始。最终目录按需渲染后再次完成真实点击回归。两端安装包已构建并核验；Windows CI 与用户设备结果单独记录，不将当前 macOS 结果外推。

原生功能回归使用 `scripts/create-chapter-fixtures.py` 生成的原创合成文本，未接触用户书库。另在隔离测试数据库检查 [W3C/IDPF 公开样例](https://github.com/IDPF/epub3-samples/releases/tag/20230704)：`moby-dick.epub` 141 个目录项、1,210,831 UTF-16 单元；`epub30-spec.epub` 314 个目录项、316,959 单元。两者均导入成功、目录锚点位于正文内、隐藏时选章成功，无导入警告。样例与临时数据库仅在 `.local/`，不提交、不随试用包分发；试用包只附原创合成示例。证据 `.local/chapters-020/native/official-sample-result.json`。

## 构建与交付

实现提交 `a566296`；CI [39035419034](https://github.com/jlau-ice/floatleaf/actions/runs/39035419034) 运行中。

- Windows 普通 release/NSIS 交叉构建及目标 Clippy all-targets（`-D warnings`）通过；`7zz t`、解包、0.2.0 版本字段核验通过。
- macOS 普通 app/DMG 构建通过，`codesign --verify --deep --strict`、`hdiutil verify` 通过。版本 0.2.0、正式应用标识一致。
- 两端普通 release 均确认不包含测试探针/测试数据目录入口。macOS ad-hoc，未公证；Windows 未正式签名。
- Windows ZIP 包含安装器、完整试用说明、先读我、MIT 协议和四份合成章节示例；独立 `Chapter-Samples.zip` 方便 macOS 使用。ZIP CRC 校验通过。
- 本地附件目录 `.local/github-release-0.2.0/`；本轮未重新测量性能目标，不把构建或合成输入结果视为用户设备完整验收。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| Chapter-Samples.zip | 2,797 | `9610a3c801556b679a140abeabbbaa3367b10da3cda309e3e2434f0e683b45f3` |
| FloatLeaf_0.2.0_aarch64.dmg | 3,746,467 | `8a4a3e4bfaedb59e9a91767678a4c59f8ab61e2bea8a41054b93180dfc1da585` |
| FloatLeaf_0.2.0_windows-x64_trial.zip | 2,974,658 | `b97e04f303c1244f9960f84be40745adac9ebb5275f3e326fe63c46bc98c93a1` |
| FloatLeaf_0.2.0_x64-setup.exe | 2,986,121 | `6838713576311e3a746574b2c61b00aa26d4414deefb12ba6ae51c9c55a92f3a` |
| Retest-Guide.txt | 994 | `8be828bc3328896eababd93a69dc924ebbc9b7fa0e859023e712dec18f505e3e` |
| SHA256SUMS.txt | 463 | `093df256d5c58b27454ea545629f32c44a5da3a13a32c976243063ff062564bd` |

参考：[EPUB 3.3 目录](https://www.w3.org/TR/epub-33/#sec-nav-toc)、[EPUB 3.3 阅读顺序](https://www.w3.org/TR/epub-33/#sec-spine-elem)。解析依赖版本纳入 Cargo.lock。
