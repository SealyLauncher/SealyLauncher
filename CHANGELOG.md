# Changelog

本文件记录 SealyLauncher 的每一个实施步骤。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

每个代码步骤都必须先更新本文件，再提交。每条记录应包含：

- 日期（Asia/Shanghai）与阶段/步骤
- 变更文件
- 行为变化
- 验证命令与结果
- 已知问题
- 下一步

## [Unreleased]

### Added

- 2026-11-08 — Step 0：保存实施计划到 `docs/PLAN.md`；新增 `README.md`、`.gitignore`、`.gitattributes`；建立本变更日志。
- 2026-11-08 — 初始化 Git 仓库，默认分支 `main`。
- 2026-11-08 — Phase 0 Step 1：安装 Rust `1.99.1` 与 Visual Studio 2022 Build Tools（MSVC 14.44 + Windows SDK 10.0.26100）；创建 `sealy-core` / `sealy-ui` / `sealy-app` Cargo workspace；建立 `gpui-kit = "=0.9.0"` 的 GPUI hello window、自定义标题栏、分层背景与 240ms 页面切换动效；添加 `rust-toolchain.toml`、`rustfmt.toml`、`.github/workflows/ci.yml`、`LICENSE`、`THIRD_PARTY_NOTICES.md`。
- 2026-11-08 — 新增 `sealy-launcher --version` 与基础 `sealy-core` 版本测试；`sealy-core` 保持无 GPUI 依赖。
- 2026-11-08 — CI/CD：拆分 Windows 质量任务与 macOS/Linux 仅构建任务；新增 tag 触发的 portable 发布 workflow（Windows zip、macOS universal zip、Linux tar.zst、SHA256SUMS）；新增 macOS `Info.plist`、Linux `.desktop`、`OEM.conf.example`。
- 2026-11-08 — Phase 1：`sealy-core` 新增 portable 路径、设置读写、日志、单实例锁、事件/任务骨架；`sealy-ui` 新增 kit 封装、主题 tokens、动效 tokens、双语文案、ShellState/Route、共享组件、NavRail、Home/实例/下载/账号/设置页面与路由动效；设置页可切换中英文并持久化。
- 2026-11-08 — 设计探索：新增 `design/preview/index.html`，提供三个 UI 方向的静态预览（深海玻璃 / 流畅霓虹 / 杂志深色）及 headless Edge 生成的 1280×800 截图；待确认方向后再重做 `sealy-ui`。
- 2026-11-08 — 设计探索：新增两个设计语言预览 —— `04 · Fluent 2`（Mica 底、4/8px 圆角、1px 描边、40 字符 accent 按钮、NavigationView 选中条）与 `05 · Material Design 3`（seed #6750A4 深色基线、tonal surface、胶囊导航指示、FAB、state layer），并核实 GPUI 原生支持 `WindowBackgroundAppearance::MicaBackdrop` / `MicaAltBackdrop` / `Blurred`。
- 2026-11-08 — 原型验证：新增 `sealy-ui::m3`（`palette` 用 `material-colors 0.5.0` 从 seed 派生 24 个角色色、`widgets` 提供导航栏/按钮/卡片/chip/FAB/线性进度/ripple、`demo` 为可运行窗口）与 `cargo run -p sealy-app --bin m3_demo`；支持 4 个 seed 实时换色与亮/暗切换，截图见 `design/preview/gpui-m3-*.png`。
- 2026-11-08 — 设计探索：按 GPUI Kit 设计指南（原生、克制、精确）与 LumilioCL 的界面语言新增 `design/preview/direction-6.html` —— 暖白画布 + 单一像素叙事 hero + 抖动接缝 + 橙色强调 + 底部 dock，并附截图 `direction-6.png`。
- 2026-11-08 — 设计探索：新增 `design/preview/direction-7.html`，把像素叙事与杂志式排版合并：分栏 hero（左排版 / 右像素画）、等宽字体小标签、发丝分隔线、列表化“最近启动”、黑色“游戏记录”卡 + 像素点阵数字；截图 `direction-7.png`。
- 2026-11-08 — 调研：实际运行 BakaXL 3.5.1，记录其 OOBE 结构与视觉语言（三区布局、面包屑标题、底部操作带、选项卡预览、拖动/长按确认），截图存于 `design/preview/bakaxl-oobe-*.png`；首页被强制更新弹窗阻断，未能截取，改以内容结构参考。
- 2026-11-08 — 设计探索：推翻此前方向，新增首页草稿 `design/preview/home-v1.html`：左侧信息面板（实例 / 参数 / 世界 / 启动 / 账号）+ 右侧全幅夜景（程序生成）+ 公告浮层 + 最近启动磁贴；截图 `home-v1.png`。
- 2026-11-08 — 规范：新增项目级 skill `.agents/skills/bakaxl-flow-ui/`（SKILL.md + `references/tokens.md` / `patterns.md` / `checklist.md`），把 BakaXL 式三区布局、动作带、大色块、水印线稿、一屏一个决定固化为可执行规则；`docs/PLAN.md` §9 与 Phase 1/3/4 同步改写。
- 2026-11-08 — 设计探索：新增 `design/preview/home-v2.html`（BakaXL 磁贴式主页，中性深色外壳 + 绿色启动）与 `design/preview/home-v3.html`（同一布局的杂志排版版：纸白、发丝线、等宽小标签、纯黑区块），用于同构对比两套视觉语言。
- 2026-11-08 — 定稿方向：以 `home-v3` 的杂志排版为视觉基线，新增 `design/preview/home-v4.html` 与 `oobe-v4.html` —— 全部改用系统自带字体、颜色收敛到 `[data-theme]` 的 4 套 token（paper/ink/clay/celadon）、去掉网络延迟/Java 版本/上次游玩等装饰性小字；`docs/PLAN.md` §9.1/§9.2 与 skill 的 tokens/patterns/checklist 同步更新。
- 2026-11-08 — 重写 `sealy-ui`：移除 M3 原型（`m3` 模块、`m3_demo`、`material-colors` 依赖）；`theme.rs` 改为 `ThemeTokens` + 4 套预设；`AppBar`/`ActionBar`/`Tile`/`Breadcrumb` 组件落地；主页改为磁贴布局，设置页可切换主题与语言并持久化；窗口 1120×700，最小 960×640。
- 2026-11-08 — 修复与补全：最小窗口改为 1030×640（磁贴网格固定 950px）；`Assets`→`AllAssets` 修好缺失图标；标题栏跟随主题；`最近世界` 计数与右侧对齐；上一步按钮与主按钮同高。新增首启向导 `pages/oobe.rs`（语言 → 音频 → 完成），首次启动自动进入、完成后落盘，设置页可重新运行；主页与向导的中英文文案接入 `i18n::tr`。
- 2026-11-08 — 主页调整：标题带改为「SealyLauncher ↔ Hi，用户名」8 秒循环（顺序淡入淡出，不重叠），左下角固定显示待启动实例（图标 + 名称 + 版本）；设置页补上背景音乐、下载源、内存上限三项并即时保存。
- 2026-11-08 — 键盘与动效：窗口级快捷键 `Enter/Space` 触发主操作、`Esc` 在向导中返回上一步、`1`–`5` 切换页面（根元素持焦点，`track_focus` + `on_key_down`）；设置新增「动态效果：系统/始终/从不」，选「从不」时标题带停用轮播；设置页内容可滚动，行高收到 52。
- 2026-11-08 — 三页结构落地：`实例`（排序行 + 4 行列表 + 计数/编辑/启动）、`下载`（搜索框 + 版本行 + 安装 + 进度动作带）、`账号`（三张登录方式卡 + 当前账号行 + 切换）；删除已无用的 `placeholder.rs`，下载按钮改用索引 id 避免重复。
- 2026-11-08 — Phase 2 起步：新增 `sealy-core::manifest`（`version_manifest_v2.json` 的类型、`VersionKind` 过滤、BMCLAPI→官方回退），下载页改为真实数据 —— 后台线程拉取、加载/失败/重试三种状态、按 全部/正式版/快照 过滤、列表可滚动；共 922 个版本。`ureq` 采用 `native-tls`（SChannel），因为本机没有 C 编译器和 nasm，rustls/ring 构建失败。
- 2026-11-08 — Phase 2 下载器：新增 `net`（ureq 封装）、`version`（版本文档、规则过滤、classpath/plan）、`install`（拉文档 → 落盘 `<id>.json` → 下载 client jar 与全部 libraries → SHA-1 校验、相同哈希跳过、进度回传）；下载页「安装」调用真安装，动作带显示进度条与 `done/total · 文件名`，完成弹「已安装 …（76 个文件）」。实测安装 `26.4-snapshot-7`：客户端 jar 42 MB + 75 个库 84.6 MB，全部通过校验。
- 2026-11-08 — 消息中心 + 多线程下载 + 返回按钮：新增全局 `panel.rs`（标题栏铃铛 + 右侧滑出面板，列出运行中/已完成/失败任务，带进度条与取消）；安装状态从下载页动作带迁到消息中心（`Message` / `MessageTone` / `InstallTask`）；`install::run_tasks` 改为线程池并发（线程数取 `settings.download.concurrency`），支持 `CancelFlag` 协作取消；所有子页面标题带加「←」返回主页。
- 2026-11-08 — 游戏目录与 Java：`LauncherPaths.game` 改为 `<root>/.minecraft`（versions/libraries/assets/runtime 全落这里）；新增 `sealy-core::java` —— `probe`/`detect`（runtime 目录、JAVA_HOME、PATH、常见安装目录，解析 `java -version`）、`fetch_runtimes`（Mojang `products/java-runtime` 索引）、`pick_runtime`（按版本的 `javaVersion.component` + `majorVersion` 匹配）、`install_runtime`（按文件清单下载并校验）、`ensure`（先探测再按需下载）。安装流程改为两段：版本文件 → `准备 Java N`（实测该快照要求 Java 25，自动下载 `java-runtime-epsilon` 411 个文件 100 MB，`java -version` 输出 25.0.1）。设置页新增 Java 行显示本机已探测到的版本与路径。
- 2026-11-08 — 资源与启动（Phase 2 打通）：新增 `assets`（索引落盘 `assets/indexes/<id>.json`、对象按哈希落到 `assets/objects/xx/<hash>`、并发下载、已存在跳过）与 `launch`（classpath 拼装、`arguments` 的 rules 过滤与 `${}` 占位符替换、legacy 兜底、natives 解压、进程启动并把 stdout/stderr 写入 `logs/<id>-latest.log`、离线 UUID）。安装流程第三段为 `下载资源 N`；主页改为读取真实已安装版本，`启动游戏` 走完整链路。实测：5402 个资源对象 460 MB、natives 解压、启动 `26.4-snapshot-7` 后日志出现 `Setting user: Haikuorenyuyue` / Vulkan 初始化，游戏窗口正常出现（辅助功能欢迎页）。
- 2026-11-08 — 实例管理：新增 `sealy-core::instances`（`InstanceBook` 存 `SealyData/instances.json`，含选择、自动重名、删除后改选、`last_played` 时间戳）与 `launch_flow`（主页与实例页共用启动流程）；`LaunchSpec` 拆成 `root`（共享 `.minecraft`：versions/libraries/assets）与 `game_dir`（实例目录），每个实例在 `<game>/.minecraft/instances/<name>` 下拥有独立存档与设置。实例页改为真实列表：新建 / 选择 / 删除 / 启动；主页读取当前选中实例。修复 `--assetsDir` 仍指向实例目录导致客户端找不到资源索引的问题。移除 `bakaxl-flow-ui` skill，README 与 PLAN 改指向 `docs/PLAN.md §9`。
- 2026-11-08 — 账号与选版本：新增 `sealy-core::accounts`（`AccountBook` 存 `SealyData/accounts.json`，`Account::offline` 用 MD5 生成与官方一致的离线 UUID，测试对齐 `OfflinePlayer:Notch` 已知值）；账号页支持内联输入用户名创建离线账号（回车确认 / Esc 取消，长度 16、仅字母数字下划线）、选择、删除；启动流程改用当前账号的 `name` + `uuid`，主页问候语显示账号名。实例页「新建实例」改为版本选择器（列出已安装版本，点选后创建）。

### Verified

- `git init -b main` 成功。
- 工作区文件清单：`README.md`、`CHANGELOG.md`、`docs/PLAN.md`、`.gitignore`、`.gitattributes`。
- 上述文件已 `git add` 暂存。
- 首次提交：`chore: bootstrap repository and implementation plan`，作者 `Tasuku Hachisuka <hachisukaauto@mingze.de>`（仓库本地配置，不修改全局 Git 身份）。
- `rustc 1.99.1` / `cargo 1.99.1`；`rustup show active-toolchain` 为 `stable-x86_64-pc-windows-msvc`。
- `cargo fmt --all -- --check` 通过。
- `cargo check --workspace` 通过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。
- `cargo test --workspace` 通过（1 个测试）。
- `cargo build -p sealy-app` 通过；首次全量构建约 7m56s。
- `target\debug\sealy-launcher.exe --version` 输出 `SealyLauncher 0.1.0`。
- 窗口 smoke test：进程存活、有可见窗口；通过窗口消息点击页面按钮后切换到 `Phase 0 · Motion`，动效与状态切换生效。
- `cargo tree -p sealy-core` 不包含 `gpui` 依赖。
- Phase 1：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`（8 个测试）、`cargo build -p sealy-app` 全部通过。
- Phase 1 窗口 smoke：Home/下载/账号/设置页面可切换；中文文案、NavRail 图标、数据目录显示正常；语言切换为 English 后写入 `target/debug/SealyData/settings.json`。
- CI/CD 文件已静态检查；尚未在 GitHub Actions 上实际运行。

### Notes

- Git 作者身份已配置为仓库本地：`Tasuku Hachisuka <hachisukaauto@mingze.de>`；未修改全局配置。
- Step 0 不安装 Rust/MSVC，不创建 Cargo workspace，不写业务代码。
- Phase 0 Step 1 的 Windows 本地构建与窗口 smoke 已完成；`ci.yml` 已添加但尚未在远端 GitHub Actions 执行，macOS/Linux 真机 smoke 仍待完成。
- 当前为 debug 构建；未做 release 打包、签名或 60fps 正式帧时间测量。窗口截图保存在 `target/phase0-motion3.png`（未提交，避免包含桌面隐私）。
- Phase 1 的下载列表、账号入口、实例页使用静态/空状态占位；点击安装/登录只显示“Phase 2/3 接入”提示，尚未连接真实数据。
- 单实例目前只阻止第二个实例启动，尚未实现“激活已有窗口”的 IPC；设置保存采用 remove+rename 的尽力原子写，Phase 2 再加固。
- UI 目前使用系统字体与 GPUI Kit 默认图标子集；字体打包、完整图标目录和更细的 hover/focus 状态留待后续。
- 下一步：Phase 2（版本元数据、下载器、Java 检测、原版安装与启动、CLI dry-run）。
