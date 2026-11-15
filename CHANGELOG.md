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

- 2026-11-15 — 自更新加固（进度/取消/校验/回滚/跨平台替换）：
  - 下载：进度条（字节计数）、取消按钮、失败重试；下载中断会删除半成品文件。
  - 完整性：读取 Release 的 SHA256SUMS，下载后校验 SHA-256，缺失或不匹配直接失败；校验通过才进入暂存。
  - 安全替换：先解压到 SealyData/updates/staging-<version>，再交给更新脚本；Windows 改为“等进程退出→重命名旧 exe→复制暂存文件→失败则恢复并重启”，避免先删后解压导致无启动器；脚本写入 SealyData/logs/update.log，成功/失败均有记录。
  - 跨平台：macOS 支持解压 zip、替换 .app bundle 后 open 重启；Linux 支持 tar.zst 解压、替换二进制并重启（缺少 zstd 时给出明确提示）。
  - 失败恢复：更新前写 pending.json，启动时 econcile_pending 检测“版本未变”的上次更新，提示用户并清理 updates/ 缓存；成功更新后脚本/启动流程会删除安装包与暂存目录。
  - 自动检查：新增设置 update.auto_check（默认开）与「启动时检查更新」开关，启动后静默检查并在有新版本时提示；检查失败不打扰用户。
  - 实测：Windows 沙箱 debug 版（新代码）→ 真实 v0.1.36 发布包，UI 下载→校验→暂存→替换→自动重启成功，update.log 记录 starting/finished，无 .bat/.old 残留；另验证 pending 恢复：伪造 pending 后启动会清理缓存（截图 design/preview/gpui-update-progress.png 等）。
  - 新增 [docs/SIGNING.md](docs/SIGNING.md)：Windows/macOS/Linux 代码签名获取途径、CI 接入方式与当前 SmartScreen/Gatekeeper 限制说明。
  - 补充实测：发布版 v0.1.37 → v0.1.38 走完整 UI 更新（进度条 0.0/8.5 MB + 取消按钮可见 → 校验 → 暂存 → 替换 → 重启），update.log 记录 starting/finished，updates/ 与 .bat/.old 均无残留；下载取消路径有单测（半成品文件被删除）。

- 2026-11-15 — 设置页侧栏、内存控制、macOS 窗口与菜单栏：
  - 设置页改为「外观 / 下载 / Java 与内存 / 网络与目录 / 启动器」五类侧栏（每类带图标），内容按分类分开显示；主题、缩放、语言、动效、下载源、Java、代理、目录、更新与维护各自归位。
  - 游戏内存：新增滑块（512 MB–32 GB，256 MB 步进）、点击数值手动输入（回车保存、Esc 取消、范围校验）、常用值 2048/4096/8192，以及「自动」模式：按物理内存 1/4、512 MB 对齐并限制在 1–8 GB，由启动器每次启动前决定；切换模式或修改值后滑块与数值框同步。新增 sealy-core::system（Windows GlobalMemoryStatusEx / macOS sysctl / Linux /proc/meminfo）与 4 个单测。
  - macOS：注册 on_reopen，关闭窗口后点击 Dock 图标会重新打开窗口（已有窗口时激活）；新增菜单栏「SealyLauncher / 文件 / 编辑 / 视图 / 帮助」，包含关于、检查更新、设置、新建实例、导入整合包、打开游戏/数据/日志目录、导航、GitHub 仓库与退出，并注册 ⌘, / ⌘N / ⌘O / ⌘1–3 / ⌘Q 快捷键；Dock 右键菜单加入新建实例、导入整合包、设置。
  - 自更新实测（Windows 沙箱）：v0.1.34 启动器通过 UI「检查更新 → 下载并安装」下载 v0.1.35 资产（8.7 MB），自动替换 exe 并重启；重启后 --version 与 --check-update 均为 0.1.35，更新脚本自行清理。
  - 已知限制：macOS/Linux 的自动替换尚未实现（当前提示手动解压覆盖），macOS 菜单与 Dock 重开仅通过 CI 编译验证，未在真机上实测。

- 2026-11-15 — B5 收尾：启动器自更新与界面缩放：
  - 新增 sealy-core::update：查询 GitHub 最新 release、按平台选择资产（windows-x64.zip / macos-universal.zip / linux-x86_64.tar.zst）、下载并解压 zip；Windows 下生成 sealy-update.bat，等待启动器退出后替换 exe 并重新启动。
  - 设置页「启动器更新」：显示当前版本、检查更新（实测从 v0.1.33 检出 v0.1.34）、下载并安装；新增 sealy-launcher --check-update 供脚本/CI 检查。
  - 界面缩放：AppearanceSettings.ui_scale_pct（90/100/115/130%），kit::px 统一乘以全局缩放因子，设置页切换立即生效并持久化；实测 130% 下布局正常（截图 design/preview/gpui-ui-scale.png）。
  - 新增 5 个单元测试：版本号比较、平台资产选择、更新包解压、缓存路径等。

- 2026-11-15 — B5 前半：代理、游戏目录切换、崩溃分析：
  - 设置新增「HTTP 代理」（host:port，留空关闭）：写入配置后对后续所有 HTTP 请求生效（ureq 代理），并在启动游戏时注入 -Dhttp/https.proxyHost/proxyPort；格式非法会提示。
  - 设置新增「游戏目录」切换：可指向任意磁盘目录（自动创建），切换后重新扫描已安装版本与 Java；持久化到 settings.json，下次启动仍生效。
  - 新增 sealy-core::crash：从最新的 crash-reports/*.txt（含实例目录）或 logs/latest.log 提取摘要，按 10 类常见错误（OOM、Java 版本、NoSuchMethod/NoClassDefFound、重复模组、Mixin、模组依赖、OpenGL、权限、下载失败、NPE）给出中英文建议；4 个单元测试。
  - 实例详情页新增「崩溃分析」面板（摘要、建议、报告片段、重新分析、打开报告目录），Esc 关闭；游戏进程非零退出后自动打开该面板。
  - 实测：伪造 OOM 崩溃报告 → 面板显示摘要与“内存不足”建议（截图 design/preview/gpui-crash-analysis.png）；设置页网络卡片可见并可点开编辑。

- 2026-11-15 — B4 整合包导入与导出：
  - 新增 sealy-core::modpack：解析 Modrinth .mrpack（modrinth.index.json + files/hashes/dependencies）与 CurseForge 包（manifest.json，读取 Minecraft 版本、modLoaders[0].id 与 overrides 目录）；路径安全校验拒绝 ../绝对路径；按 SHA-1 校验下载并支持已存在文件跳过；解压 overrides/、client-overrides/。
  - 整合包导出：把实例的 mods/config/resourcepacks/shaderpacks/scripts 写成合法 .mrpack（files 为空、内容放在 overrides），可直接再导入。
  - 导入流程接入安装任务：按包内依赖安装 Fabric/Quilt/Forge/NeoForge 与对应版本（或原版），自动创建实例并下载包内文件，最终在实例页显示与打开；CurseForge 包里无法解析下载地址的条目会统计并以提示告知。
  - 实例页新增「导入整合包」按钮（import-modpack），实例详情页新增「导出整合包」按钮（export-modpack）；新增 headless 导入：sealy-launcher --import-modpack <路径>。
  - 实测：本地 HTTP 搭建的 .mrpack（1 个模组 + overrides/config）导入后实例目录文件 SHA-1 与包内一致、override 正确解压；导出的 .mrpack 结构与 modrinth.index.json 正确。另加调试钩子 SEALY_DEBUG_IMPORT_PACK / SEALY_DEBUG_EXPORT_PACK（仅 debug 构建）用于无文件对话框时跑通流程。

- 2026-11-15 — B3 收尾：OptiFine 安装器改为以游戏目录为参数启动官方安装窗口（实测窗口标题「OptiFine Installer」正常弹出），完成后可直接在版本列表里创建实例；删除过期的下载页旧截图。
- 2026-11-15 — B3 加载器安装与交互改版：
  - sealy-core::loader 支持 Forge / NeoForge（自动）与 OptiFine（启动官方安装器窗口，按提示完成）：下载官方 installer jar，准备 launcher_profiles.json 与可写 java.io.tmpdir，以 --installClient <game> 静默安装，随后定位版本文档；Fabric/Quilt 支持指定构建；新增每个加载器的版本列表（Forge promotions、NeoForge Maven 元数据、Fabric/Quilt 构建列表）。
  - 新增 cargo run -p sealy-core --example install_loader -- <root> <java> <kind> <game> [version] 便于手动验证安装器路径；实测 Forge 1.20.1-47.2.0 与 NeoForge 21.1.266 安装成功。
  - 新建实例面板改为「选版本 → 选加载器（原版/Fabric/Quilt/Forge/NeoForge/OptiFine）→ 选加载器版本 → 安装并自动建实例」；下载页与实例页移除 Fabric/Quilt 快捷按钮。
  - 微软登录卡片新增「打开 microsoft.com/link」与「复制代码」，设备码到达时自动复制到剪贴板。
  - 输入框支持鼠标点击切换焦点：authlib 表单与实例启动设置面板。
  - 模组/资源包/光影改为系统文件选择器（可多选文件与文件夹）导入，配合拖放，移除手动粘贴路径。
  - 实例详情页改为左侧 sidebar：概览 / 模组 / 资源包 / 光影。
  - 下载页版本列表支持无限滚动（滚动时逐批加载，刷新后重置）。

- 2026-11-15 — 目录打开改为独立 helper 进程（修复点击「打开日志目录 / 打开实例目录」后界面卡住）：
  - 打开目录不再在 UI 线程调用 shell，改为用同一可执行文件的 `--open-folder <路径>` 模式起一个短命进程，避免 ShellExecuteW 重入 GPUI 的 App 借用（旧实现会刷 RefCell already borrowed 直到窗口不再绘制）。
  - helper 依次尝试 cmd /c start、explorer.exe、ShellExecuteW（调用前初始化 COM），失败时把原因写入 %TEMP%\sealy-open-folder.log，UI 侧仍会弹 toast 提示成功或失败。

- 2026-11-15 — B2 实例内容管理（模组 / 资源包 / 光影 / 图标 / 安全删除）：
  - sealy-core::addons：按实例列出、启用/禁用（.disabled 后缀）、删除、导入文件与文件夹，附 3 个单元测试。
  - 实例详情页新增三个分区：模组 / 资源包 / 光影，支持一键启用禁用、删除、粘贴路径导入，以及把文件或文件夹拖到分区上直接安装。
  - 实例图标：详情页可从盒子 / 层叠 / 包裹 / 服务器四种内置图标中切换，写入 instances.json 并在实例列表与详情页显示。
  - 删除实例改为确认面板：可选「仅移除记录」（可在提示里撤销）或「连同文件删除」（删除整个实例目录，不可恢复）。

- 2026-11-15 — 实例详情页与目录打开修复：
  - 新增 Route::Instance 与 pages/instance.rs：显示版本、实例目录、最后启动、当前 Java、内存/窗口、JVM 参数，底部提供「日志 / 启动设置 / 打开目录 / 启动游戏」。
  - 实例列表双击进入详情页；首页底部当前实例卡片与右侧详情按钮也会进入；详情页返回箭头回实例列表，Esc 同样返回。
  - 修复「打开日志目录」「打开实例目录」在部分 Windows 上无效：改为调用 shell ShellExecuteW（open 动词），失败时回退 explorer.exe，并在成功/失败时给出 toast 与日志，不再静默。

- 2026-11-15 — B1 收尾：
  - **实例日志查看器**：实例页「日志」按钮打开右侧面板，显示 logs/<version>-latest.log 尾部 400 行，打开期间每 1.2s 自动跟随，可刷新、打开日志目录、Esc 关闭。
  - **实例启动设置**：实例页「设置」按钮打开面板，可编辑窗口宽/高、最大内存、附加 JVM 参数；留空跟随全局，Tab 切字段、Enter 保存、Esc 取消；写入 instances.json 的 launch 字段（LaunchOverrides），启动时覆盖全局值并追加 JVM 参数。
  - **选中实例强调**：列表行选中时加强调色描边、左侧强调色竖条，名称改用强调色，和未选中行明显区分。
  - 修复实例/账号列表 a11y 节点 id 用名称长度导致重复、debug 构建导航到实例页即 panic 的问题（改用行下标）；首页「实例」磁贴补 ole/aria_label。

- 2026-11-14 — 对照 HMCL 源码整理功能差距，docs/PLAN.md 新增 §20（账户/实例/加载器/模组/整合包/下载/Java/体验/联机九组、优先级与工作量、B1–B6 批次）；同时落地 B1 第一项 **Java 管理**：设置页可列出检测到的 Java、点击设为默认、切回「自动」、手动输入 java 路径（回车校验并加入列表）、重新检测；启动时 pick_java 优先使用所选 Java，版本不匹配时回退到匹配项。新增 pick_java 单测。

- 2026-11-14 — docs/PLAN.md §16.1 更新为 2026-11-14 快照：重算各阶段完成度（Phase 2 ~92%、Phase 4 ~88%、Phase 5 ~82%，整体 ~82%），补记本轮启动核心/认证/UI/发布改动、四项当前阻塞与下一步优先级。

- 2026-11-14 — macOS/Linux 修复（三项）：
  - **Java 运行时下载**：macOS 的运行时位于 jre.bundle/Contents/Home/，旧代码只找 in/java，导致「下载完成但报缺失」；现在按候选路径查找。Unix 端按清单里的 executable 标记补回可执行位（此前 in/java 与 lib/jspawnhelper 没有 +x，probe 直接失败），并按 link 条目重建 legal/ 下的符号链接、按 directory 条目建目录。
  - **macOS 双击打不开**：.app 被 Gatekeeper 隔离或放在 /Applications 时，<bundle>/SealyData 不可写，旧代码直接退出且无提示；现在 LauncherPaths::prepare() 在便携目录不可写时回退到 ~/Library/Application Support/SealyLauncher（Linux 用 $XDG_DATA_HOME/~/.local/share，Windows 用 %APPDATA%），并在消息中心说明回退原因；检测到 App Translocation 时提示把 .app 移到「应用程序」。启动失败的致命错误会在 macOS 弹原生对话框并写 SealyLauncher-error.log。
  - **发版**：macOS 打包显式 chmod +x、xattr -cr 后再 ad-hoc 签名并用 codesign --verify 校验，额外产出 .tar.gz（tar 解压不继承 quarantine）。
- 2026-11-14 — Java 探测补上 macOS JDK 包（Contents/Home/bin/java、Homebrew libexec/openjdk.jdk）与 Linux /usr/lib/jvm、SDKMAN 路径，系统已装 JDK 时不再误判为「没有 Java」。
- 2026-11-14 — 通知区域文字不再被裁切：消息中心宽度 380→420，标题/正文改为可换行并按最长 12 字符插入零宽断点，长路径与 JSON 不再被截断；右下角 toast 同样限宽 420 并自动换行。

- 2026-11-14 — 新增 docs/APPID-REVIEW.md：按当前表单的 9 个问题逐项给出填写内容（含英文 Justification 与补充说明文案、Tenant ID 获取位置、应用所有权的前置判断）。

- 2026-11-14 — 文档：说明 Mojang 的 AppID Review Complete 邮件是批量模板（条件句、不点名 AppID、同时覆盖屏蔽结果），收到不等于通过；README 的 403 排查顺序补上「看服务端原话区分 Invalid app registration」与「重新提交」两步。

- 2026-11-14 — 登录实测定位：日志显示设备码 200、Xbox Live 200、XSTS 200（user_hash_len=20、xerr=0），login_with_xbox 返回 403 且响应体为 Invalid app registration，即账号侧正常、应用未获 Mojang 认可；403 提示按该响应体单独给出排查清单（审核邮件 ID 比对、应用未被删除、支持的帐户类型、允许公共客户端流、生效延迟），并附 ka.ms/AppRegInfo。

- 2026-11-14 — 微软登录诊断：所有请求补 User-Agent: SealyLauncher/<version>；认证链路每一步（设备码、Xbox Live、XSTS、login_with_xbox、profile）写 INFO 日志，记录状态码、XErr、user hash 长度与脱敏后的响应体；XSTS 缺少 DisplayHelpers 时回退使用 Xbox Live 的 user hash（此前会退化成 XBL3.0 x=;token，服务端同样回 403）；设置页新增「Microsoft 客户端 ID」只读行，可直接核对运行时真正使用的 ID；403 提示补充「对照官方启动器」的判别方法，README 同步更新排查步骤。

- 2026-11-12 — 交互与调试：右下角 toast 改为统一 5 秒自动消失（Toast + PendingUndo + 单一定时器）；删除实例/账号后 toast 带「撤销」，5 秒内可原位恢复并恢复选中；实例页「新建实例」改成可滚动版本列表（显示版本类型/继承来源），行内可直接装 Fabric/Quilt，安装完成后自动刷新已安装列表；修复标题栏铃铛被窗口拖动逻辑吞掉点击导致关掉后打不开的问题（on_mouse_down 阻止冒泡）；移除各页左上角  1– 5 黑色角标（含首页）；设置页新增「调试日志」开关，开启后通过 	racing_subscriber::reload 把文件日志升级为 DEBUG，并可直接打开日志目录。

- 2026-11-11 — CI 与发版修复（八）：runner 没有 `libxkbcommon-x11`，链接报 `unable to find library -lxkbcommon-x11`；新增一个只有三个 `xkb_x11_*` 符号、SONAME 与真实包一致的链接期 shim，构建出的二进制在用户机器上仍加载系统真实库。`n- 2026-11-11 — CI 与发版修复（七）：runner 镜像自带 xkbcommon 运行时但没有 `-dev` 符号链接，链接阶段报 `unable to find library -lxkbcommon`；新增一步在 `/usr/lib/x86_64-linux-gnu` 下补 `libxkbcommon.so` / `libxkbcommon-x11.so` 符号链接，跳过无法访问的 apt 源。`n- 2026-11-11 — CI 与发版修复（六）：runner 对内网 ubuntu 镜像与 archive/security.ubuntu.com 均解析失败（`Could not resolve`），apt 步骤改为 `continue-on-error`，让构建阶段直接暴露缺库问题。`n- 2026-11-11 — CI 与发版修复（五）：日志显示真实原因是 `Temporary failure resolving 'azure.archive.ubuntu.com'`（托管 runner 的 DNS 抖动），因此把 apt 源换成官方 archive/security 镜像，并给 `update`/`install` 各加三次重试。`n- 2026-11-11 — CI 与发版修复（四）：`apt-get update` 已验证正常，瓶颈在 install，改为 `--no-install-recommends` + `Dpkg::Use-Pty=0`，避免拉取 mesa/vulkan 的推荐包。`n- 2026-11-11 — CI 与发版修复（三）：ubuntu 的依赖安装仍在 15 分钟后超时，改成为 `update` / `install` 两个独立步骤（8 / 20 分钟超时）并强制 IPv4（`Acquire::ForceIPv4`）与 dpkg 锁等待；README 增加「Microsoft 登录返回 403」排查说明。`n- 2026-11-11 — CI 与发版修复（续）：ubuntu job 这次卡在 `apt-get` 30 分钟后失败（镜像源超时），两条工作流的依赖安装都加了 `Acquire::Retries=3`、`http::Timeout=20`、15 分钟步骤超时与 `DEBIAN_FRONTEND=noninteractive`，避免再出现长时间静默等待。`n- 2026-11-11 — CI 与发版修复：ci.yml 的 Linux 依赖清单漏了续行符，libssl-dev 被当成命令执行，导致 ubuntu job 失败；Release 工作流的 bump 步骤推 tag 被拒（分支推送正常），改为由 ction-gh-release 通过 API 创建 tag（	arget_commitish: main）。Microsoft 登录的 403 现在会带上实际服务端返回与正在使用的 client id，便于核对审批。

- 2026-11-10 — 版本继承与加载器：ersion::resolve 合并 inheritsFrom 链（父版本 jar/assets/Java/参数 + 子版本 mainClass 与库，库按 group:artifact 去重），启动与安装都改走合并后的文档，jar_id 决定用哪个版本的 client jar；新增 loader 模块从 Fabric/Quilt 元数据取最新稳定加载器、写入 profile JSON，下载页每个版本多出「Fabric」「Quilt」按钮，走统一安装流程（版本文件 → assets → Java）。实测 Fabric 0.20.1 + 26.4-snapshot-7 能启动，日志出现 Loading Minecraft 26.4-snapshot-7 with Fabric Loader 0.20.1。
- 2026-11-10 — authlib-injector：新增 uthlib 模块（Yggdrasil 登录/刷新、API 根规范化、错误消息透传、ensure_injector 自动下载 jar、-javaagent 参数生成）；账号模型加 server 字段，令牌缓存加 authlib 两项，启动时自动刷新并注入；账号页新增「外置登录」三字段表单（Tab 切换 / 回车登录 / Esc 取消），失败信息落消息中心。
- 2026-11-10 — PLAN 增补「16.1 实施状态」小节，逐阶段记录完成度与偏差。

- 2026-11-10 — 任务取消与消息中心：每条消息携带自己的取消开关（Message.cancel），消息中心的「取消」只停对应任务，微软登录与版本安装可以并存互不影响；安装流程重写为三段（版本文件 → 资源文件 → Java）并共用同一条消息，阶段名显示在详情行；补回缺失的资源文件（assets）下载阶段——此前安装只拉取版本文件，导致新版本没有 assets。消息面板加 occlude() 与点击空白的遮罩层，Esc 也能关闭。窗口内嵌图标经 Win32 WM_SETICON 设为任务栏图标；release 构建使用 windows_subsystem = "windows"，不再弹出控制台窗口。

- 2026-11-08 — 引导页账号步骤：向导从 3 步变 4 步（语言 → 声音 → 账号 → 完成），账号步骤必须至少有一个账号才能继续（无账号时「下一步」置灰且 oobe_next 直接返回）；提供「离线账号」（内联输入名字，回车确认 / Esc 取消）与「Microsoft 登录」（设备码卡片，登录成功自动进入下一步）两条路径，start_microsoft 抽到 uth_flow 供账号页与向导共用。主页左下角在没有实例时显示「无实例 / 先在实例页新建」，不再用假数据占位。


- 2026-11-08 — 发版链路修复：GitHub 的 `macos-14` 标签在本账号返回「configuration not supported」，改用 `macos-latest`；`publish` 改为只要 Windows/Linux 成功就发布，macOS 失败不再拖累整次发版；Windows release 构建设置 `SEALY_REQUIRE_ICON=1`，找不到 `rc.exe` 直接失败，确保 exe 一定带图标。首个自动发版产出 `v0.1.2`（三平台 + SHA256SUMS）。`n- 2026-11-08 — 打包与发版：Release 工作流改为**每次推 main 自动发版**（patch 自增 → 提交 chore(release): vX.Y.Z [skip ci] → 打 tag → 三平台构建 → 发布 Release + SHA256SUMS），支持手动 workflow_dispatch 选 patch/minor/major/none；新增 packaging/windows/sealy.ico、packaging/macos/SealyLauncher.icns、packaging/linux/sealy-launcher.png，crates/sealy-app/build.rs 在找到 c.exe 时把图标嵌进 exe（找不到则跳过，不阻断构建），macOS Info.plist 引用 CFBundleIconFile，Linux 包附带 png。


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
- 2026-11-08 — Microsoft 登录：新增 `net` 的 JSON/表单辅助、`microsoft`（设备码申请与轮询、MSA 刷新、XBL → XSTS → `login_with_xbox` → `profile`，含 XErr 错误码翻译与 403「未通过 MCE 审核」专用提示）、`auth`（`TokenStore` 存 `SealyData/tokens.json`，`ensure_minecraft_token` 命中缓存或自动刷新）；设置新增 `auth.microsoft_client_id`（默认已填 `e95460ea-…`）；账号页新增「Microsoft 登录」按钮与设备码卡片（显示 `microsoft.com/link` + 8 位代码 + 取消），登录成功后写入账号与令牌。启动链路按账号类型传 `--accessToken`/`--userType msa`。设备码流程实测可拿到真实 `user_code`（如 `EFSWYN3U`），取消与失败状态均落到消息中心。
- 2026-11-08 — 实例操作、下载细化与图标：实例页新增重命名（内联输入，回车确认 / Esc 取消，同时移动实例目录）、复制（`<名字> 副本`，复制整个游戏目录）、打开文件夹；列表行加 12px 内边距修掉图标贴边。下载页对已安装版本显示「✓ 已安装」；安装进度新增单文件字节进度（`current_bytes` / `current_size`，`net::download_to` 增加回调）；失败任务在消息中心带「重试」。新增 `SealyAssets` 资产源嵌入启动器图标，标题栏左上角显示应用图标。

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
