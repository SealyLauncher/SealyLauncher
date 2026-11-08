# SealyLauncher 实施计划（最终执行版）

> 状态：2026-11-08 已由用户确认。Step 0（仓库初始化）执行后暂停，等待用户指令“执行第一步”。
> 本文是实施与验收的单一事实来源；后续变更应先更新本文，并在 CHANGELOG 中记录。

## 0. 本次确认的落地约定

你已经确认整体计划。批准后先做**仓库初始化（Step 0）**，不安装 Rust、不写业务代码；完成以下内容后暂停，等你明确说“执行第一步”。

### 0.1 Step 0：仓库初始化

1. **保存计划**
   - 创建 `docs/PLAN.md`，保存本计划的完整内容。
   - 创建 `README.md`：项目简介、当前状态（bootstrap）、计划/CHANGELOG 链接、构建前置说明、非官方声明。
2. **建立 CHANGELOG 约定**
   - 创建 `CHANGELOG.md`，采用 Keep a Changelog 结构。
   - 从 Step 1 开始，**每一个代码步骤都必须先更新 CHANGELOG 再提交**。
   - 每条记录包含：日期（Asia/Shanghai）、阶段/步骤、变更文件、行为变化、验证命令与结果、已知问题、下一步。
   - 提交信息使用 Conventional Commits：`feat:`、`fix:`、`chore:`、`docs:`、`test:`、`refactor:`。
3. **初始化 Git**
   - `git init -b main`。
   - 创建 `.gitignore`：Rust `target/`、`SealyData/`、`dist/`、IDE/OS 文件、本地环境文件、签名文件；**保留 `Cargo.lock`**（应用项目应提交）。
   - 创建 `.gitattributes`：源码 LF；`.bat`/`.ps1` CRLF；二进制资源标记。
   - `git add` 上述文件，做首次提交 `chore: bootstrap repository and implementation plan`。
   - 若本机没有 `user.name`/`user.email`，**不伪造作者**：先 `git init` + `git add`，汇报并请你提供身份后再提交。当前已确认全局 Git 身份为空。
4. **暂停**
   - 不安装 Rust/MSVC，不创建 Cargo workspace，不写 UI 代码。
   - 汇报已创建的文件、git 状态、待你提供的 Git 身份；等待你说“执行第一步”。

### 0.2 后续“第一步”的定义

Step 0 完成后，你回复“执行第一步”，我再开始 **Phase 0 Step 1**：

- 安装 Rust stable + MSVC Build Tools + Windows SDK；
- 创建 Cargo workspace、`rust-toolchain.toml`、基础 lint/format 配置；
- 用 `gpui-kit = "=0.9.0"` 跑通 hello window、自定义标题栏、一个页面切换动效、背景图与 `reduce_motion`；
- 建 CI 骨架（Windows/macOS/Linux 编译 + fmt/clippy/test）；
- 更新 CHANGELOG、提交、汇报 gate 结果。

---

## 1. 已核实的环境与参考依据

### 1.1 本机环境

| 项 | 结果 |
| --- | --- |
| 工作目录 | `C:\Users\Haikuorenyuyue\Desktop\SealyLauncher`，空目录 |
| 架构 | Windows AMD64 |
| Rust | 未安装（rustc/cargo/rustup 均不存在） |
| MSVC | 未发现 vswhere/cl.exe/link.exe，需要 VS 2022 Build Tools + Windows SDK |
| Node | v26.13.0 / npm 11.20.0（Phase 6 Web 用） |
| Git | 2.56.0.windows.4，尚未 init，全局 user.name/email 为空 |
| OEM.conf | 不存在 |
| 磁盘 | C 盘剩余约 453 GB |

### 1.2 HMCL 参考点（只参考协议/行为，不复制源码）

| 文件 | 参考内容 |
| --- | --- |
| [MicrosoftService.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/microsoft/MicrosoftService.java>) | Xbox Live → XSTS → Minecraft Services → entitlements → profile；XErr 错误码；skin |
| [OAuth.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/OAuth.java>) | 授权码 + PKCE、设备码轮询、refresh、错误处理 |
| [OAuthServer.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCL/src/main/java/org/jackhuang/hmcl/game/OAuthServer.java>) | 本地回调端口 29111–29115；client id 由构建注入，不能复用 |
| [AuthlibInjectorAccount.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/authlibinjector/AuthlibInjectorAccount.java>) | `-javaagent:`、`authlibinjector.side`、`yggdrasil.prefetched` |
| [AuthlibInjectorServer.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/authlibinjector/AuthlibInjectorServer.java>) | `x-authlib-injector-api-location`、metadata 缓存、serverName/links/non_email_login |
| [OfflineAccountFactory.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/offline/OfflineAccountFactory.java>) | 离线 UUID：`md5("OfflinePlayer:" + name)` UUID v3 |
| [HMCLDownloadProvider.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCL/src/main/java/org/jackhuang/hmcl/download/HMCLDownloadProvider.java>) | BMCLAPI 根、源优先级、镜像 URL 表、`x-bmclapi-hash` |
| [GameLibrariesTask.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/download/game/GameLibrariesTask.java>) | 库规则、natives、SHA-1、重复下载判断 |
| [DefaultLauncher.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/launch/DefaultLauncher.java>) | classpath、natives、JVM/游戏参数、占位符、环境、日志 |
| [JavaManager.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCL/src/main/java/org/jackhuang/hmcl/java/JavaManager.java>) | 三平台 Java 搜索路径、官方运行时 |
| [MojangJavaDownloadTask.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/download/java/mojang/MojangJavaDownloadTask.java>) | Mojang JRE `all.json` |
| [ServerModpackManifest.java](<C:/Users/Haikuorenyuyue/Desktop/HMCL/HMCLCore/src/main/java/org/jackhuang/hmcl/modpack/server/ServerModpackManifest.java>) | 后续服务器整合包 `fileApi` + 文件哈希 + addons 数据模型 |
| [LICENSE](<C:/Users/Haikuorenyuyue/Desktop/HMCL/LICENSE>) | GPL-3.0-or-later WITH AdditionRef-HMCL；改名、保留版权声明 |

### 1.3 GPUI 版本事实

- 官方 `gpui` 最新 0.2.2；`gpui-kit 0.9.0` / `gpui-component 0.9.0` / `gpui-pre 0.3.11` 于 2026-11-02 发布。
- `gpui-kit` 统一重导出 GPUI 0.3.11、`gpui-base`、`gpui-component`、Lucide 资产；入口为 `gpui_kit::application()`、`gpui_kit::init(cx)`、`gpui_kit::open_window(...)`。
- GPUI 0.3.11 提供 `Animation`、`AnimationExt::with_animation` / `with_animations` / `with_spring`、`SpringAnimation`；`with_animation` 自动尊重 `reduce_motion`。
- `WindowOptions` 支持 `titlebar.appears_transparent`、`traffic_light_position`、`window_decorations`，适合自定义标题栏。
- 决策：精确 pin，`Cargo.lock` 锁定，不追最新；升级走专门分支回归。

---

## 2. 范围与成功标准

### 2.1 v1.0 必须完成

1. **平台**：Windows 10/11 x64；macOS 13+ universal（arm64 + x86_64）；Linux x86_64（X11 + Wayland）。Windows ARM64、Linux ARM64 不做。
2. **实例/版本**：原版安装/启动（1.7.10、1.12.2、1.16.5、1.20.1、1.21.x 及快照）；Fabric、Quilt；链接已有 `.minecraft`/HMCL 实例并启动已安装 Forge/NeoForge；共享 libraries/assets、实例独立 gameDir；校验/修复/取消/续传/磁盘预检。
3. **认证**：离线；Microsoft（浏览器回调 + PKCE、设备码兜底、refresh、多账号）；authlib-injector（多提供商、metadata 发现、Yggdrasil 登录/刷新、角色选择、jar 下载与注入）。
4. **下载**：官方 / BMCLAPI / Auto；元数据与文件源分开；失败回退；校验；缓存；并发限制。
5. **Java**：系统 Java 搜索；按版本选 major；Mojang 官方运行时下载；自定义 Java。
6. **UI**：自定义设计系统、完整页面、动效、空/错/加载/离线/锁定状态、键盘与无障碍、中英双语；移除不必要的小标题/副标题。
7. **便携**：所有数据在 `SealyData/`；不写注册表、不写全局 `.minecraft`、不创建全局关联、不覆盖其它启动器；Windows zip、macOS zip（含 `.app`）、Linux tar.zst；不产安装器。
8. **OEM 本地壳层**：根目录 `OEM.conf` 检测、品牌、默认服务器、默认 authlib 提供商；远程接口先冻结。
9. **工程**：CI 三平台构建/测试；fmt/clippy/test/deny；文档；许可证/第三方声明；CHANGELOG 每步更新。

### 2.2 v1 不做

- Forge/NeoForge 原生安装（v1.1；v1 仅启动已安装实例）。
- 整合包安装/导出/自动更新（Phase 6 接口先定）。
- OEM 远程同步、Node.js Web 管理端（Phase 6）。
- 启动器自更新、托盘、Discord RPC、崩溃自动上传。
- 离线皮肤在游戏内生效（v1.1 本地 Yggdrasil）。
- 皮肤/披风上传。
- 动态 APNG/GIF/视频背景（v1 静态图 + 交叉淡入）。
- Windows ARM64、Linux ARM64、Windows 7。
- MSI/DMG/DEB/RPM/AppImage。
- 遥测、广告、账号数据上传。

### 2.3 验收标准

| 维度 | 标准 |
| --- | --- |
| 功能 | 三种登录、两类下载源、原版/Fabric/Quilt、导入 Forge 实例全部可用 |
| 平台 | 三平台 portable 包可启动；Windows 本机实测；macOS/Linux CI + 一次真机 smoke |
| 便携 | 除 OS keychain 与用户选择的游戏目录外不越界写；复制文件夹可迁移 |
| 认证安全 | token 不进日志/UI/崩溃报告；OEM 签名失败拒绝；zip/path traversal 测试通过 |
| 动效 | 页面/背景/播放键/弹层完整；reduce_motion 生效；无 >32ms 长帧 |
| 性能 | 冷启动 <1.5s；空闲 CPU <1%；1000 版本列表流畅；zip <150MB |
| 设计 | 无装饰性小标题/副标题；设置无分组标题；空状态仅图标 + 一个动作 |
| 质量 | fmt/clippy -D warnings/test/deny 全绿；许可证 allowlist 通过 |
| OEM | 无 OEM.conf 时零 OEM 请求；有则仅本地生效；删除后下次关闭 |

---

## 3. 技术选型

| 领域 | 选择 |
| --- | --- |
| 语言 | Rust stable，edition 2024；`rust-toolchain.toml` 固定 |
| UI | `gpui-kit = "=0.9.0"`，默认 features（component + assets） |
| UI 隔离 | `sealy-ui/src/kit/` 薄封装；页面不直接依赖 gpui-kit |
| 异步 | `tokio` 专用 runtime 线程；UI 不阻塞 |
| UI 桥接 | `async_channel` + `tokio::sync::oneshot`；进度 10Hz 合并 |
| HTTP | `reqwest` + rustls、gzip/brotli、系统/手动代理 |
| 序列化 | `serde` / `serde_json` / `toml` |
| 校验 | `sha1`、`sha2`、`md-5`、`base64`、`uuid` |
| 压缩 | `zip`、`tar`、`flate2` |
| 密钥 | `keyring` + `secrecy`/`zeroize`；失败时受限权限文件 + 提示 |
| 图片 | `image`；背景预处理、预模糊、主色提取、皮肤头像裁剪 |
| 日志 | `tracing` + `tracing-appender`；脱敏层 |
| 错误 | `thiserror` + 稳定错误码；core 不用 anyhow |
| CLI | `clap`：`--dry-run`、`--install`、`--diagnostics` |
| 测试 | `tempfile`、`wiremock`、`insta`、`proptest` |
| 打包 | 自研 `xtask` + GitHub Actions |
| License | GPL-3.0-or-later；依赖 allowlist（Apache-2.0/MIT/BSD/ISC/OFL） |

**回退方案**：若 `gpui-kit 0.9.0` 在 Windows 有阻断问题，回退官方 `gpui 0.2.2` + 自研基础组件；代价 +1～2 周，页面结构与动效设计不变。

---

## 4. 总体架构

```
sealy-app (binary)
  ├─ 启动、CLI、单实例、日志、panic hook、平台初始化
  └─ sealy-ui (GPUI)
        ├─ 页面/组件/主题/动效
        ├─ 只消费 CoreHandle 的 DTO 与事件
        └─ 不直接做网络/文件/进程
              ↕ async_channel / oneshot
        sealy-core (纯 Rust，无 GPUI)
          ├─ paths/config/logging
          ├─ http/download/source
          ├─ meta/version/assets
          ├─ install/verify
          ├─ java
          ├─ auth (offline/microsoft/authlib)
          ├─ launch
          ├─ instance/import
          ├─ oem (local + future remote)
          └─ ext traits / event bus
```

- `sealy-core` 不得依赖 `gpui`（CI 用 `cargo tree -p sealy-core` 检查）。
- UI 不做 I/O；`sealy-app` 是唯一组合根，注入 paths/HTTP/SecretStore/ProcessSpawner/OemProvider。
- 纯函数（参数、规则、镜像映射、UUID、PKCE）独立单测；HTTP 用 mock；进程用 fake java；UI 用 gpui-kit test-support。

### 4.1 目录结构

```
SealyLauncher/
├─ Cargo.toml / Cargo.lock / rust-toolchain.toml
├─ README.md / CHANGELOG.md / LICENSE / THIRD_PARTY_NOTICES.md
├─ docs/{PLAN,ARCHITECTURE,DESIGN,MOTION,OEM-CONTRACT,MS-AUTH,PLATFORM,TEST-MATRIX}.md
├─ crates/
│  ├─ sealy-core/src/{lib,paths,config,http,meta,source,install,java,auth,launch,instance,oem,ext,events,api}.rs
│  ├─ sealy-ui/src/{kit,theme,motion,components,pages,app,bridge}.rs
│  └─ sealy-app/src/{main,cli,single_instance}.rs
├─ assets/{fonts,icons,branding,backgrounds}
├─ tests/fixtures/
├─ xtask/
├─ packaging/{windows,macos,linux}/
└─ .github/workflows/{ci,release}.yml
```

### 4.2 数据流

1. UI 发 `Command`（安装/启动/登录/刷新）。
2. `LauncherService` 在 tokio runtime 执行，返回 `RequestId`；结果通过 oneshot 或事件返回。
3. `TaskManager` 发 `CoreEvent::TaskProgress`；进度 10Hz 合并，满了丢旧进度；终态走可靠通道。
4. UI 用 `cx.spawn` 接收，`AsyncApp::update` 后 `cx.notify()`；不轮询。
5. 取消/失败/完成都产生终态事件。

---

## 5. 便携、路径、多启动器并存

### 5.1 根目录

- 普通平台：可执行文件所在目录。
- macOS：可执行文件在 `.app/Contents/MacOS` 时，根目录为 `.app` 所在目录。
- 进程 CWD 不参与判定，避免快捷方式误启用 OEM。
- `--root <dir>` 可显式指定；数据与 OEM 都以该目录为根。
- 根目录必须可写；不可写时明确报错并指引移动，**不静默退回 AppData**。

### 5.2 数据布局

```
<root>/
├─ SealyLauncher.exe / SealyLauncher.app / sealy-launcher
├─ OEM.conf
└─ SealyData/
   ├─ .lock
   ├─ settings.json
   ├─ state.json
   ├─ accounts.json
   ├─ providers.json
   ├─ instances/<slug>/{instance.json, icon.png, mods, config, saves, ...}
   ├─ game/{versions, libraries, assets}
   ├─ java/<component>/<platform>/<version>/
   ├─ cache/{meta,sha1,images}/
   ├─ runtime/authlib-injector/
   ├─ logs/{launcher,game}/
   └─ oem/{cache,assets}/
```

- 共享 libraries/assets/versions；每实例独立 gameDir。
- 导入已有 `.minecraft`：
  - **链接模式（v1 默认）**：`repository_root` 指向原 `.minecraft`，复用 libraries/assets/versions；gameDir 指向原目录；提示不要同时用两个启动器启动。
  - **复制模式（后续）**：复制版本文件，不复制 mods/saves。
- 全路径用 `PathBuf`/`OsString`；实例 slug 过滤保留名/分隔符；版本 ID 拒绝 `..`/绝对路径。

### 5.3 多启动器并存

- 唯一数据目录 `SealyData`；不写全局 `.minecraft`/注册表/文件关联/开机启动。
- 单实例锁只锁当前 `SealyData/.lock`；不同副本可同时运行。
- 第二实例通过 named pipe / Unix socket 激活已有窗口，失败提示“已有一个实例”。
- 删除文件夹即卸载；数据随文件夹迁移。
- 不做系统级 Java 安装、不改环境变量。

### 5.4 OEM.conf 作用域

- 只检测 `<root>/OEM.conf`（Linux 大小写敏感；可容忍 `oem.conf` 并警告）。
- 不读父目录/AppData/注册表。
- 删除后下次启动回普通模式；缓存仅 `SealyData/oem/`。

---

## 6. 游戏实例、版本安装与启动

### 6.1 版本 JSON

支持：`id`、`type`、`mainClass`、`inheritsFrom`、`jar`、`assets`、`assetIndex`、`libraries`、`arguments`（game/jvm）、`minecraftArguments`、`downloads.client`、`javaVersion`、`logging`、`complianceLevel`、`minimumLauncherVersion`、`releaseTime`。

- 递归解析 `inheritsFrom`，检测环/深度上限；子优先、父补充；libraries 合并去重；arguments 分别合并；`jar` 指向实际 client jar。
- 未知字段忽略但保留原始 JSON；安装保存原始版本 JSON。
- rules 支持 allow/disallow、os.name/version/arch、features；无规则默认允许。
- 库：解析 Maven 坐标；优先 downloads.artifact；natives 按 `[native-]os[-arch]` 匹配并替换 `${arch}`；native jar 单独下载、解压到临时 natives。
- classpath：按合并后顺序，仅加入环境适用、非 native、文件存在者；最后加 client jar；去重。
- 资源：assetIndex → assets/indexes；objects/<前2>/<hash>；legacy `virtual` 复制/硬链接到 `assets/virtual/<id>`；`map_to_resources` 复制到 gameDir/resources。
- logging：有 `logging.client` 则下载配置并传 `-Dlog4j.configurationFile`；否则用 `-Dlog4j2.formatMsgNoLookups=true`。
- complianceLevel/minimumLauncherVersion 只警告，不阻断。

### 6.2 安装流程

1. Resolve：拉清单/版本 JSON/loader profile；解析继承；生成 `InstallPlan`（URL 候选、目标、SHA-1、大小、native/extract）。
2. Preflight：磁盘空间、目标可写、实例/版本锁。
3. Execute：阶段下载；`.part` + 校验 + 原子 rename；全局/主机并发限制。
4. Verify：可选全量校验，坏文件重下。
5. Register：写 `instance.json`、版本记录、状态。
6. Cancel：CancellationToken；不留“已完成”的坏文件；`.part` 可续传。
7. Repair：重跑校验 + 补下载；不删用户 mods/saves。

进度权重：元数据 5%、client 10%、libraries 35%、assets 40%、Java 10%；速度滑动窗口平滑。

### 6.3 Loader

- Fabric：meta loader 列表 + profile JSON；保存 `versions/fabric-loader-...`；下载 libraries；校验 mainClass/arguments。
- Quilt：对应 v3 meta。
- Forge/NeoForge（v1）：只启动已安装实例；解析 version JSON/libraries/BootstrapLauncher/ModLauncher；旧 `-DignoreList` 等兼容问题只实现测试样本覆盖的修复；完整安装器 v1.1。
- OptiFine：v1 仅导入实例启动；安装/补丁 v1.1。

### 6.4 Java

- 每实例 `JavaSelection { Auto{major} / Custom{path} }`。
- 版本 JSON `javaVersion.majorVersion` 优先；否则按游戏版本启发（≤1.16.5→8；1.17–1.20.4→17；1.20.5+/1.21→21）。
- 探测：读 `release` 优先；否则 `java -XshowSettings:properties -version` 解析 stderr；缓存 path+mtime+size；超时 5s。
- 检查位数/架构；32 位不用于现代版本；macOS ARM 旧版本可能需要 x86_64 Java + Rosetta，检测提示。
- 搜索：`JAVA_HOME`、`PATH`、用户自定义；Windows Program Files/Adoptium/Zulu/Microsoft/官方 runtime；macOS JavaVirtualMachines/Homebrew/SDKMAN/官方 runtime；Linux `/usr/lib/jvm`、`/usr/java`、SDKMAN、`~/.jdks`。
- Mojang JRE：`https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json`；按平台 key + `javaVersion.component`；逐文件 SHA-1 + 解压 + 定位 bin/java；不把 JRE 打进 portable 包。
- 默认内存 4096 MB（不超过系统内存 1/2），最小 512 MB；32 位限制提示。

### 6.5 启动参数

- JVM：`-Xmx/-Xms`、`-Djava.library.path`、`-cp`、`-Dminecraft.launcher.brand/version`、`-Dminecraft.client.jar`、`-Dlog4j2.formatMsgNoLookups=true`、`-Dfile.encoding=UTF-8`、按 Java 版本设置 stdout/stderr 编码、代理、macOS dock；清理 `_JAVA_OPTIONS`/`JAVA_TOOL_OPTIONS`/`JDK_JAVA_OPTIONS`。
- 游戏占位符：`${auth_player_name}`、`${auth_session}`、`${auth_access_token}`、`${auth_uuid}`、`${user_type}`、`${user_properties}`、`${version_name}`、`${version_type}`、`${game_directory}`、`${assets_root}`、`${assets_index_name}`、`${natives_directory}`、`${classpath}`、`${launcher_name}`、`${launcher_version}`、`${resolution_width}`、`${resolution_height}`、`${library_directory}`、`${classpath_separator}`、`${primary_jar}`。
- 旧 `minecraftArguments` 按引号/转义 tokenize；新 `arguments.game` + 条件。
- `--username`、`--uuid`（无连字符）、`--accessToken`、`--userType msa`、`--version`、`--gameDir`、`--assetsDir`、`--assetIndex`、`--versionType`；分辨率条件；快速进服 v1.1。
- authlib 注入：`-javaagent:<jar>=<url>`、`-Dauthlibinjector.side=client`、`-Dauthlibinjector.yggdrasil.prefetched=<base64 metadata>`；仅 authlib 账号。
- 日志脱敏：记录命令前把 accessToken/prefetched 替换为 `<redacted>`；真实命令只给 ProcessBuilder。
- 进程：工作目录 gameDir；Windows CREATE_NO_WINDOW；合并 stdout/stderr；UTF-8 回退本地编码；日志抽屉保留最后 2000 行；完整日志按次写；退出码/时长事件；停止先 terminate 后 kill；默认启动器保持打开。
- 边界：旧 assets、继承环、缺 jar/mainClass、坏 SHA1、空格/中文/emoji、长路径、只读/磁盘不足、并发、32 位、macOS ARM、Linux 无 Vulkan、断网/镜像错误/重定向/校验失败、日志编码、大量输出。

---

## 7. 下载源与镜像

### 7.1 策略

- `metadata_source` 与 `file_source` 可分别设 `Auto` / `Official` / `Bmclapi`。
- Auto：系统 locale 为 `zh-*` 或时区 `Asia/Shanghai`/`Asia/Urumqi` → BMCLAPI 优先、官方回退；其他 → 官方优先、BMCLAPI 回退。
- Official：仅官方；Bmclapi：BMCLAPI 优先、官方回退。
- 不调用外部 IP 地理接口。

### 7.2 BMCLAPI 映射

| 官方前缀 | 目标 |
| --- | --- |
| `piston-meta.mojang.com` / `piston-data` / `launchermeta` / `launcher.mojang.com` | `<root>` 保留路径 |
| `libraries.minecraft.net` | `<root>/libraries` |
| `files.minecraftforge.net/maven` / `maven.minecraftforge.net` | `<root>/maven` |
| `maven.neoforged.net/releases/` | `<root>/maven/`；另加 `neoforged.forgecdn.net` |
| `meta.fabricmc.net` | `<root>/fabric-meta` |
| `authlib-injector.yushi.moe` | `<root>/mirrors/authlib-injector` |
| `resources.download.minecraft.net` | `<root>/assets` |
| Maven Central | 腾讯 Maven 镜像 |
| Modrinth API/CDN | `mod.mcimirror.top`（后续整合包） |

表驱动 + 单测；已指向 BMCLAPI 的 URL 不二次重写。

### 7.3 下载器

- 候选列表按源策略生成；`cache/sha1/<sha1>` 命中则硬链接/复制。
- 连接 10s、读 60s、最多 3 次指数退避 + jitter；支持 Retry-After。
- `.part` + Range 续传；完成后 SHA-1/SHA-256 校验；不匹配换候选。
- 默认并发 8（1–16）；每主机 6；元数据 4；Semaphore。
- 元数据缓存 + ETag/Last-Modified + TTL；`x-bmclapi-hash` 只作为缓存优化，最终仍按元数据校验。
- zip/jar 额外完整性检查；离线只用缓存/已安装文件；失败不静默成功。

---

## 8. 认证

### 8.1 统一模型

```rust
enum AccountKind { Offline, Microsoft, AuthlibInjector }

struct AccountRecord {
    schema: u32,
    id: String,
    kind: AccountKind,
    display_name: String,
    profile: Option<PlayerProfile>,
    provider: Option<ProviderId>,
    created_at, last_login_at,
    state: AccountState,
}
```

- `accounts.json` 只存元数据；token 存 keychain（`SealyLauncher/<root-hash>/<account-id>`）。
- `AuthSession` 不实现 `Debug` 或自定义脱敏；`SecretString`。
- 头像：从 skin 裁 8×8 脸 + 帽子层，缓存 PNG；无皮肤用渐变首字母。

### 8.2 离线

- 名称 3–16 字符 `[A-Za-z0-9_]`；重复名称提示 UUID 相同。
- UUID = `md5("OfflinePlayer:" + name)` 的 UUID v3。
- AccessToken 随机 32 位十六进制；`user_type=msa`；`user_properties={}`。
- 不请求网络；本地皮肤预览 v1；游戏内皮肤 v1.1。
- 不承诺可进正版验证服。

### 8.3 Microsoft

**前置条件**：需要你自己的 Azure 应用注册，不能复用 HMCL client id。Phase 0 配置：

- Entra ID → App registrations → 新注册；个人 Microsoft 账户。
- 平台“移动和桌面应用”；回调注册 `http://localhost:29111/auth-response` 至 `29115`。
- 开启 Allow public client flows（设备码）。
- 不建 client secret；构建时 `SEALY_MS_CLIENT_ID` 注入；未配置时入口禁用并提示。

**授权码 + PKCE**：

1. 绑定 29111–29115 之一；全忙走设备码。
2. 生成 verifier/challenge/state。
3. 打开 `https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize`，scope `XboxLive.signin offline_access`，`prompt=select_account`，S256。
4. 本地 server 只接受 localhost、`/auth-response`、校验 state；5 分钟超时；一次有效请求后关闭。
5. `POST .../token` 换 token（authorization_code + code_verifier）。
6. 失败/超时/端口占用 → 设备码：`POST .../devicecode`，展示 user_code，轮询 token；处理 pending/slow_down/expired；15 分钟超时。
7. Xbox：`user.auth.xboxlive.com/user/authenticate`（RpsTicket `d=<token>`）→ `xsts.auth.xboxlive.com/xsts/authorize`（RETAIL，rp=api.minecraftservices.com）；取 uhs。
8. 错误码：2148916233 无 Xbox 账号、2148916235 地区、2148916238 家庭、2148916227 封禁；给出可行动文案。
9. Minecraft：`authentication/login_with_xbox`（`XBL3.0 x=<uhs>;<xsts>`）→ `entitlements/mcstore` 校验拥有 → `minecraft/profile` 取 UUID/name/skins；404 区分无 license/无 profile。
10. 存储：MS refresh token + MC access token/expiry 进 keychain；启动前 5 分钟 refresh；新 refresh token 覆盖旧值；`invalid_grant`/`AADSTS70000` → NeedsReauth。
11. 皮肤：profile skins[0] 下载缓存，做头像；上传 v1.1。
12. token 不记录、不发 OEM、不进 URL。

### 8.4 authlib-injector

- 提供商发现：输入 URL → 补 https（localhost 开发允许 http 并警告）→ GET → 读 `x-authlib-injector-api-location` → 跟随并规范化尾 `/`；缓存原始 metadata + ETag。
- 解析 `serverName`、`links`、`feature.non_email_login`、`skinDomains`；UI 调整用户名/邮箱表单。
- 多提供商 `providers.json`；同 URL 去重；OEM 预置/锁定 Phase 6。
- Yggdrasil：
  - `POST /authserver/authenticate`（agent Minecraft/1、username、password、clientToken、requestUser:false）
  - `POST /authserver/refresh`（accessToken、clientToken、selectedProfile）
  - `POST /authserver/validate`（204/403）
  - `POST /authserver/invalidate`
  - `GET /sessionserver/session/minecraft/profile/<uuid>`
- 角色选择：无 selected 时列出 availableProfiles；选择后 refresh 绑定。
- 密码不落盘；只存 accessToken/clientToken/selectedProfile；启动前 validate → refresh → 重新登录。
- 工件：`latest.json`（官方或 BMCLAPI 镜像）；字段 `build_number`、`version`、`download_url`、`checksums.sha256`；下载到 `runtime/authlib-injector/`；SHA-256 校验；支持用户本地 jar。
- 注入参数见 6.5；prefetched 用原始 metadata 字节 Base64；不把 jar 打进包；About 显示来源/许可。
- 错误：凭据、服务器不可达、无角色、角色删除、封禁、metadata 格式错误。

### 8.5 密钥与隐私

- Windows Credential Manager / macOS Keychain / Linux Secret Service；不可用时写 0600 文件并提示。
- root-hash 命名空间，避免多副本共享 keychain。
- 日志/崩溃/诊断包脱敏；不遥测；OEM 请求仅在 OEM.conf 存在时发生。

---

## 9. UI/UX 设计

### 9.1 设计原则（可验收）

风格基线：**BakaXL 式桌面流程界面**（2026-11-08 实机调研），规范见
`.agents/skills/bakaxl-flow-ui/SKILL.md`。后续所有界面按该 skill 执行。

1. **一屏一个决定**：主操作用一句动词说清；不在一屏塞两个并列主按钮。
2. **三区固定**：`AppBar`（36px）/ 内容 / `ActionBar`（64px）；有主操作的页面动作带常驻，位置永不变。
3. **颜色即功能**：绿=继续/开始，琥珀=返回，蓝=外壳与选中；强调色可被 OEM 覆盖，语义不变。
4. **平面优先**：大色块 + 1px 发丝线；禁用渐变、玻璃、霓虹、投影堆叠。
5. **水印代替插图**：品牌线稿以 8–12% 不透明度铺满背景，不引入图片素材。
6. **面包屑当标题**：`父级 \ 当前步`，不再叠加小标题/副标题。
7. **展示优于描述**：选项用预览缩略图（如“标准/传统”），不用两段文字解释。
8. **不可逆/授权步骤加摩擦**：拖动、长按或按住空格确认，不用复选框。
9. **按钮成对且贴边**：动作带右侧按钮占满带高，绿在右、琥珀在左，宽度按文案 160–220px。
10. 所有用户可见字符串进 i18n；中英文不得溢出；默认 `paper` 主题，其它主题 token 同名。
11. **字体只用系统自带**：界面字体用系统 UI 栈（`system-ui` / Segoe UI / 苹方 / 微软雅黑），
    等宽用系统等宽栈；不打包、不分发任何字体文件，避免体积与授权问题。
12. **禁止装饰性小字**：网络延迟、Java 版本、上次游玩时间、构建号这类信息不进界面框架，
    必须有入口时用图标、状态点或收进设置/日志页。
13. **颜色全部经主题接口**：组件不得出现裸色值；换主题 = 换一组 token；OEM 只允许覆盖强调色。

### 9.2 视觉系统

主题基准为“纸张”：暖白底 + 纯黑作为唯一强调色，靠分格线与字号建立层级。

**主题 token（`ThemeTokens`，4 套内置预设）**

| token | paper | ink | clay | celadon |
| --- | --- | --- | --- | --- |
| `bg` | `#F4F2EE` | `#14161A` | `#F3ECE3` | `#EEF2EF` |
| `surface` | `#FFFFFF` | `#1B1F26` | `#FFFDF9` | `#FFFFFF` |
| `line` | `#DCD8CF` | `#2A3038` | `#E0D4C4` | `#D3DCD6` |
| `text` | `#101010` | `#EDF1F6` | `#1B1512` | `#10201B` |
| `muted` | `#6F6A61` | `#8B94A3` | `#766A5E` | `#5F6F68` |
| `block` / `block_text` | `#0D0D0D` / `#F4F2EE` | `#F2F4F7` / `#101318` | `#2B1C14` / `#F7F0E7` | `#0F3D33` / `#EEF5F1` |
| `accent` / `accent_text` | `#0D0D0D` / `#F4F2EE` | `#F2F4F7` / `#101318` | `#B4552A` / `#FFF8F2` | `#0F766E` / `#F2FBF8` |

- `accent` 只用于主操作（启动/继续），一屏最多一处；`block` 用于反色区块（公告块、当前项）。
- 状态色 `danger #C42B1C` 保留，仅用于关闭窗口与删除确认。
- 圆角：窗口 6、区块 2、按钮 2；不使用 8 以上的圆角。
- 字号阶梯：页面标题 42、区块标题 13–15、正文 13、等宽标注 9.5–10、动作按钮 20–22。
- 间距阶梯：4 / 8 / 10 / 14 / 18 / 24 / 32；磁贴内边距 14–18。
- 字体：系统 UI 栈 + 系统等宽栈，不打包字体文件。
- 图标：单一线性图标族（Lucide），描边 1.5–1.7，基准 20–24。

### 9.2.1 主题接口

- Rust：`sealy-ui::theme::ThemeTokens`（`Copy` 结构体 + `serde` 反序列化），
  `Settings.appearance.theme` 存预设名，`OEM.conf` 只允许 `accent` / `accent_text` 两项覆盖。
- 预览：`design/preview/home-v4.html` / `oobe-v4.html` 用 `[data-theme]` + CSS 变量复刻同一套 token，
  切换器仅用于预览，不进产品界面。
- 新增组件必须从 token 取色；code review 中见到裸色值即打回。

### 9.3 布局与页面

默认 1120×700，最小 1030×640（磁贴网格固定 950px，加两侧 32px 内边距）；自绘 36px 标题栏（左：品牌标 + 标题；右：反馈 / 帮助 / 窗口控件）。

**主导航**：顶部标签（主页 / 实例 / 下载 / 账号 / 设置），选中项绿色下划线；不设左侧图标轨。

**首页**（内容顺序固定）：
1. 启动区：当前实例名 + 版本/loader 元数据 + 绿色「开始游戏」（主按钮唯一）；
2. 最近实例：横向卡片，封面为主，右下角时间；
3. 公告：三行以内，日期 + 一句话；
4. 底部状态带：Java / 内存 / 下载源 / 网络延迟。

**实例页**：卡片网格；选中卡片显示 2px 强调描边；底部动作带「启动 / 编辑」。
**下载页**：搜索 + 分段筛选 + 虚拟列表；行内「安装」；进度进动作带。
**账号页**：横向账号卡 + 添加卡；添加走 `ActionBar` 三步（选择方式 → 凭据 → 角色）。
**设置页**：左侧列表 + 右侧表单；即时生效项用 Switch；危险项放最末分组。
**首次运行（OOBE）**：语言 → 外观/音频 → 完成；每步一屏、共用动作带。
实现位于 `crates/sealy-ui/src/pages/oobe.rs`：首次启动（`settings.json` 不存在）自动进入，
完成后写入设置；设置页提供「重新运行」入口。

### 9.4 组件

`AppBar`、`Breadcrumb`、`ActionBar`、`ActionButton`（green/amber/neutral）、`Watermark`、
`StepBody`、`OptionCard`（预览图 + 单选）、`AccountCard`、`SkinHead`、`InstanceTile`、
`NoticeRow`、`StatusBand`、`Segmented`、`Switch`、`Slider`、`TextField`、`Select`、
`Tooltip`、`Badge`、`Pill`、`Sheet`、`Dialog`、`ToastHost`、`EmptyState`、`ErrorState`、
`ProgressBar`、`Skeleton`、`ThemeProvider`、`MotionProvider`、`LocaleProvider`。

页面不直接用 `gpui_kit::component::Button` 默认样式，统一走 `kit` 封装。


### 9.5 无障碍与本地化

- AccessKit 语义、图标按钮 label、2px 焦点环、合理 Tab 顺序。
- 键盘：Tab/Shift+Tab、Enter/Space、Esc、列表方向键、`Ctrl/Cmd+,` 设置、`Ctrl/Cmd+Enter` 开始。
- 对比度 ≥4.5:1；100/150/200% 缩放；最小窗口不裁关键操作。
- reduce_motion 尊重系统；提供系统/始终/从不覆盖。
- zh-CN/en-US；默认跟随系统；错误本地化；authlib serverName 原样。
- 不做 RTL v1。

---

## 10. 动效系统

### 10.1 基础

- `DurationTokens`：instant 80ms、fast 120ms、base 200ms、page 240ms、slow 320ms、ambient 12s。
- `EasingTokens`：standard `cubic-bezier(0.2,0,0,1)`、enter `(0,0,0,1)`、exit `(0.3,0,1,1)`。
- `SpringTokens`：nav 420/32、sheet 380/34。
- 基于 `Animation` + `with_animation` / `with_spring`；页面不硬编码时长/曲线。
- `with_animation` 自动处理 reduce_motion；自建 timer 显式检查。
- 只动 opacity/transform/颜色/位置；避免每帧大列表布局。
- 失焦/最小化暂停环境动画；最多同时 2 个环境动画。

### 10.2 动效清单

| 场景 | 时长/曲线 | 说明 |
| --- | --- | --- |
| 启动淡入 | 200ms standard | 根 opacity + logo 0.98→1 |
| 页面切换 | 200ms in / 120ms out | 8px 位移 + 淡入；共享实例卡近似共享元素 |
| 导航选中 | Spring 420/32 | pill 位置/宽度；固定轨道 |
| 背景交叉淡入 | 1200ms | 双层 opacity；预加载 |
| Ken Burns | 20s | 1.02→1.06；失焦暂停 |
| 卡片 hover/press | 160ms / 90ms | 上移 2px / scale 0.99 |
| Play 呼吸光 | 3s repeat | 仅 Home 可见时 |
| Play hover/press | 160ms / 90ms | 1.03 / 0.96 + 扫光 |
| 启动中 | 1.2s repeat | 环形流光 + 阶段文字 |
| 启动成功 | 400ms | 光晕扩散收敛，不阻塞游戏 |
| Sheet/Dialog | 240ms spring / 160ms exit | 16px 上滑 + scrim |
| Toast | 200ms / 160ms | 4s 自动消失；错误不自动消失 |
| 进度条 | 200ms ease-out | transform scaleX，减少布局 |
| 不确定进度 | 1.6s repeat | 流光 |
| Skeleton | 1.4s repeat | shimmer |
| 账号头像 hover | 160ms | 轻微上浮/高光 |

### 10.3 性能与验收

- 60fps 下限、120fps 目标；p99 <16.7ms，无 >32ms 长帧。
- 空闲 CPU <1%；失焦停止环境动画。
- 背景预解码/预缩放；不直接塞 4K。
- 版本列表虚拟化；进度 10Hz。
- 重任务期间降级动效。
- reduce_motion 下 one-shot 直接终态，repeat 停起始安全态；页面切换无布局跳动。

---

## 11. OEM 远程统一控制接口

### 11.1 v1 本地壳层

```toml
schema = 1
id = "example-server"
display_name = "Example Launcher"
accent = "#4CC2FF"
background = "assets/oem-bg.webp"
logo = "assets/oem-logo.svg"
server_address = "play.example.com"
authlib_injector_url = "https://skin.example.com/api/yggdrasil"
links = { website = "https://example.com" }

# 远程预留；v1 不请求
api_base = "https://launcher.example.com/api/v1"
public_key = "base64:..."
```

- v1 只读本地品牌/默认服务器/authlib 提供商/链接；不发起 OEM 网络请求。
- 校验 schema/类型/URL/路径（必须在根目录内）；非法忽略并提示。
- OEM 可替换可见名称/logo；About 保留启动器名称、GPL 来源、第三方声明。
- 删除后下次关闭；无后台服务/全局状态。

### 11.2 远程签名清单（Phase 6 实现，v1 冻结）

```
OEM.conf → OemProvider::load()
  → GET {api_base}/manifest
  → 验签 + 反回滚 + 过期
  → OemSnapshot { branding, server, auth, downloads, content, messages, policy }
  → UI 主题 / 默认服务器 / authlib 提供商 / 更新入口
```

- 仅 HTTPS（localhost 开发除外）；规范 JSON + Ed25519；`signature`/`key_id` 不参与签名体；OEM.conf 固定公钥；支持 kid 轮换。
- 反回滚：记录最大 revision；低于拒绝；过期后不再应用安全/认证策略变更，仅保留品牌缓存并提示。
- 缓存 `SealyData/oem/cache/manifest.json`；网络失败用上次有效缓存；无缓存则禁用远程功能。
- 只下发数据，不下发脚本/命令；启动器不提供远程 shell。
- 清单字段：branding、server（name/address/status_url/news_url）、auth（allowed/authlib/lock）、downloads（source/allowed/lock）、content（server_pack manifest/auto_update/channel）、messages（level/title/markdown/expires）、policy（min_launcher_version/required/locks）。
- 资源 SHA-256 校验；图片限制尺寸/体积；SVG 安全渲染；token 永不发 OEM；OEM 只能看到启动器版本/平台/revision（可选且告知）。

### 11.3 服务器整合包自动更新（Phase 6）

- 服务端发布 revision + 文件清单（path、size、sha256、url）+ 删除列表；整体签名。
- 启动器：验签 → 对比 → 临时下载 → 校验 → 备份 → 原子替换 → 失败回滚。
- 只操作实例目录内、清单声明路径；拒绝 `..`、绝对路径、符号链接、清单外删除。
- 不执行整合包脚本；loader 只通过受信任来源 + 既定流程。
- 更新前提示/备份；可暂停自动更新；required 不得删除用户存档/截图等非托管数据。
- OEM authlib provider 自动加入/锁定；Microsoft/离线策略可锁定。

### 11.4 后续 Node.js Web 管理端（Phase 6）

- Node 24 LTS + TypeScript + Fastify；Drizzle + SQLite/PostgreSQL；本地磁盘或 S3/MinIO；sharp；Argon2id；Ed25519 私钥在 env/KMS；React + Vite 管理界面。
- 功能：品牌/背景/Logo/强调色；资讯/服务器状态/公告；authlib provider；整合包上传/清单/revision/发布/回滚；签名密钥轮换/审计/管理员/2FA/API token；公开 `/api/v1/manifest`、`news`、`server/status`、`/assets/<sha256>`。
- 安全：鉴权/CSRF/限流/上传校验/图片重编码/审计；私钥不入库/不进前端。
- 唯一契约是签名清单；不直接控制运行中的启动器，不做远程命令。
- 等你明确开始 OEM 时再细化；v1 先冻结接口/schema/签名方式。

---

## 12. 数据、配置、迁移

### 12.1 `settings.json`

```json
{
  "schema": 1,
  "language": "zh-CN",
  "appearance": { "theme": "dark", "accent": "auto", "background": { "kind": "instance_or_default", "slideshow": true, "custom": null } },
  "download": { "metadata_source": "auto", "file_source": "auto", "bmclapi_root": "https://bmclapi2.bangbang93.com", "concurrency": 8 },
  "java": { "auto_download": true, "default_path": null, "memory_mb": { "min": 512, "max": 4096 } },
  "launch": { "close_after_start": false, "fullscreen": false, "width": 854, "height": 480 },
  "proxy": { "mode": "system", "url": null },
  "motion": "system"
}
```

### 12.2 `instance.json`

```json
{
  "schema": 1,
  "id": "fabric-1.20.1",
  "name": "Fabric 1.20.1",
  "kind": "fabric",
  "version_id": "fabric-loader-0.16.5-1.20.1",
  "repository_root": "game",
  "game_dir": "instances/fabric-1.20.1",
  "java": { "mode": "auto", "major": 17, "path": null },
  "memory_mb": { "min": 512, "max": 4096 },
  "loader": { "type": "fabric", "game": "1.20.1", "version": "0.16.5" },
  "icon": "icon.png",
  "created_at": "2026-11-08T00:00:00Z",
  "last_played_at": null,
  "server_pack": null
}
```

### 12.3 写入/迁移

- 同目录临时文件 → fsync → 原子 rename；Windows 可重试 replace；保留 `.bak`。
- 读取失败：`.bak` → 默认 + 备份坏文件；不静默丢数据。
- schema 高于支持：拒绝加载并提示升级；不降级写回。
- 迁移前备份、迁移后校验；实例/账号少，启动时同步完成。
- secrets 0600；普通配置 0644；Windows 用户 ACL。
- 日志保留 7 天/50MB；游戏日志按次；缓存可清理并显示占用。
- v1 不引入 SQLite；JSON 可检查/迁移。

---

## 13. 扩展接口

```rust
pub trait DownloadSourcePlugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn candidates(&self, req: &ResourceRequest) -> Vec<Url>;
    fn priority(&self) -> i32;
}

#[async_trait]
pub trait AuthPlugin: Send + Sync {
    fn kind(&self) -> AccountKind;
    async fn login(&self, input: LoginInput) -> Result<AuthSession>;
    async fn refresh(&self, session: &AuthSession) -> Result<AuthSession>;
    async fn logout(&self, session: &AuthSession) -> Result<()>;
}

#[async_trait]
pub trait OemProvider: Send + Sync {
    async fn load(&self, ctx: &OemContext) -> Result<OemSnapshot>;
}

#[async_trait]
pub trait ContentUpdater: Send + Sync {
    async fn check(&self) -> Result<Option<UpdatePlan>>;
    async fn apply(&self, plan: &UpdatePlan, progress: ProgressSink) -> Result<()>;
}

pub trait JavaProvider: Send + Sync;
pub trait SkinProvider: Send + Sync;
```

- `PluginRegistry` 组合根注册；v1 只编译期注册，不做 `dlopen`。
- DTO 放 `sealy-core::api`，`#[non_exhaustive]`，semver；UI 只依赖 DTO。
- CLI 是自动化/后续 Web 的稳定入口：`--root`、`--offline`、`--install`、`--launch`、`--dry-run`、`--diagnostics`、`--version`。
- 未来 Web 只通过签名清单同步；不开放入站端口；“远程启动”另行设计认证 API，v1 不做。

---

## 14. 打包、平台、发布

### 14.1 产物（只 portable）

| 平台 | 产物 | 内容 |
| --- | --- | --- |
| Windows x64 | `SealyLauncher-<ver>-windows-x64.zip` | exe、assets、LICENSES、README、OEM.conf.example |
| macOS universal | `SealyLauncher-<ver>-macos-universal.zip` | `.app`、LICENSES、OEM.conf.example |
| Linux x86_64 | `SealyLauncher-<ver>-linux-x86_64.tar.zst` | binary、assets、.desktop（可选）、图标、LICENSES、OEM.conf.example |

- 不做 MSI/EXE 安装器/DMG/DEB/RPM/Snap/Flatpak/AppImage。
- 解压即用；首次运行创建 `SealyData/`；不写系统目录。
- 附 `SHA256SUMS`；可选 minisign/cosign。
- 不包含 JRE/authlib-injector/Minecraft 素材。
- 包含 GPL、第三方声明、非官方声明。

### 14.2 平台

- **Windows**：MSVC + Windows SDK；嵌入 ico/VERSIONINFO/DPI manifest/app_id；`SealyData` 在 exe 同目录；CREATE_NO_WINDOW；长路径/Unicode；可选 Authenticode。
- **macOS**：aarch64 + x86_64 `lipo`；`.app` + Info.plist（bundle id、最低 13.0、HighResolution、图标）；根目录 `.app` 所在目录；ad-hoc codesign；Developer ID/notarization 可选；Gatekeeper 说明；Metal；Rosetta 提示。
- **Linux**：Ubuntu 22.04 构建兼容 glibc；Vulkan 1.0+；X11/Wayland 至少一个；依赖 xkbcommon/wayland/X11/fontconfig/freetype/Vulkan；tar.zst；无显示时 CLI 可用；文档列依赖。

### 14.3 CI

- `rust-toolchain.toml` 固定；`Cargo.lock` 提交；`--locked`。
- `xtask dist`：构建、组装、zip/tar.zst、SHA256SUMS；macOS lipo/codesign；Windows 资源；Linux 可执行位。
- `ci.yml`：fmt、clippy -D warnings、test、cargo deny；三平台矩阵；Linux apt 依赖；缓存。
- `release.yml`：tag `v*`；三平台构建；macOS universal；上传 artifacts/checksums；可选签名。
- 质量门：fmt/clippy/test/deny/`cargo tree -p sealy-core` 不含 gpui/三平台构建/portable smoke。
- 不做自更新 v1；用户替换文件夹，`SealyData` 保留；OEM 后续可提示最低版本 + 下载链接。

---

## 15. 测试与验收

### 15.1 单元

- 版本继承/环/libraries 合并/rules/natives/classpath。
- 参数/旧 tokenizer/authlib 注入/脱敏。
- legacy assets/对象路径/SHA1/zip 完整性。
- 镜像表/无二次重写/query/Auto 优先级。
- 重试/超时/404/500/校验失败/Range/缓存/取消。
- PKCE/state/设备码/Xbox 错误/entitlements/profile/refresh 失败。
- authlib metadata 重定向/non_email_login/角色/validate/refresh/jar hash/prefetched。
- 离线 UUID 已知向量/名称校验。
- 路径安全（`..`/绝对/保留名/符号链接/zip slip）。
- 便携根（exe/.app/Linux/只读）。
- OEM schema/路径/签名（Phase 6）/反回滚/过期。
- 配置/实例迁移/原子写。

### 15.2 集成

- wiremock：版本清单/JSON/libraries/assets；BMCLAPI 错误回退；MS 全链路；authlib；Java all.json。
- 临时目录安装 1.20.1：文件树/哈希/重复安装不重复下载/取消续传。
- `--dry-run` 黄金参数：1.7.10/1.12.2/1.20.1/1.21.x；三种账号占位符。
- Fake Java：args/env/工作目录/环境清理/日志/退出码。
- 链接 `.minecraft` 样本：最小 version JSON/libraries。

### 15.3 UI

- gpui-kit test-support 渲染页面；断言角色/文本/焦点。
- reduce_motion 终态；键盘全流程；960×640/1280×800/2560×1440 @100/150/200%；中英。
- 不存在 SectionTitle/Subtitle；最多一个 DisplayTitle；空状态一个主按钮。
- 可选截图快照；跨 OS 不做像素强断言。

### 15.4 手工矩阵

- Windows 10/11 x64 本机全功能。
- macOS 13+ arm64/x86_64（CI + 你提供实机）：Metal/启动/authlib/portable。
- Linux Ubuntu 22.04/24.04 Wayland+X11、Fedora（CI + 实机）：Vulkan/输入法/CJK/启动。
- 真实 Microsoft 测试账号；authlib 公共测试提供商；离线进离线服。
- 版本 1.7.10/1.12.2/1.16.5/1.20.1/1.21.x；Fabric/Quilt；导入 Forge 1.20.1。
- 网络官方/BMCLAPI/Auto、断网、404、校验失败、token 过期、端口占用。
- 便携复制/只读/双副本/OEM.conf 存在删除损坏。
- 安全 token 日志/zip slip/路径穿越/签名错误/代理。
- 性能冷启动/空闲 CPU/页面切换/1000 列表/背景内存。

### 15.5 Gate

- 自动化全绿；三平台构建；Windows 全功能；macOS/Linux 至少一次真机 smoke。
- 无 P0/P1；已知问题记录；设计评审；动效性能；无冗余标题；安全/许可证。

> 现实约束：本机只有 Windows。Phase 0 起 CI 构建 + 无头测试；Phase 5 前需要你提供 macOS/Linux 实机或 CI 自托管机做真实启动验收，否则只能声明“构建通过”。

---

## 16. 分阶段实施

| 阶段 | 时间 | 交付 | Gate |
| --- | --- | --- | --- |
| **Step 0 仓库初始化** | 今天，批准后立即 | docs/PLAN.md、README、CHANGELOG、.gitignore/.gitattributes、git init/首次提交；**不装工具链、不写代码** | 文件存在；git 状态清晰；提交完成或等待你给 git 身份；暂停 |
| **Phase 0 工具链/可行性** | 2–3 天 | Rust/MSVC；workspace；hello window；标题栏；背景+页面切换动效；CI 骨架；Azure 说明 | Windows 60fps；CI 三平台编译；无阻断；否则回退决策 |
| **Phase 1 壳层/设计系统** | 5–7 天 | paths/config/logging/events/task；单实例；kit 封装；主题 token（`bakaxl-flow-ui`）；AppBar/ActionBar/Breadcrumb/Watermark/OptionCard；页面骨架；组件；i18n/无障碍 | 可点击壳层；动效；reduce_motion；960×640；无 SectionTitle |
| **Phase 2 启动核心** | 8–12 天 | 版本/继承/下载/vanilla/assets/libraries/natives/Java/args/进程/日志/CLI/Fabric/Quilt/导入 | Windows 启动多版本；取消/续传/校验；CLI 黄金参数；导入 Forge |
| **Phase 3 认证** | 6–8 天 | 离线/MS/authlib/账号 UI（账户流三屏）/头像 | 三种登录端到端；真实服；token 无泄漏；注入参数正确 |
| **Phase 4 UI 产品化** | 6–9 天 | 首页/实例/下载/设置 + OOBE 五步；全状态；OEM 本地；键盘/缩放/双语 | 按 `references/checklist.md` 评审；无冗余标题；性能；无障碍 |
| **Phase 5 打包发布** | 4–6 天 | xtask/三平台包/资源/checksums/CI release/文档/诊断/许可证 | 干净 Windows；mac/Linux smoke；便携/只读/多副本；OEM |
| **v1.0** | — | 全部 | 验收标准 |
| **Phase 6 OEM 远程 + Node Web** | 后续 2–4 周 | 签名清单/同步/背景/资讯/状态/authlib/整合包更新/管理端 | 签名/反回滚/路径安全；端到端；回滚 |
| **Phase 7 v1.1+** | 后续 | Forge/NeoForge 安装；整合包导入导出；皮肤上传；自更新；崩溃分析；托盘 | 按功能 |

总估算 5–7 周；GPUI 回退 +1–2 周；Forge 提前 +1–2 周。Phase 6 若依赖 Forge，先补 v1.1。

---

## 17. 风险与备选

| 风险 | 缓解 |
| --- | --- |
| GPUI pre-1.0/kit bug | pin + kit 封装 + Phase 0 门 + 回退官方 gpui |
| 无 mac/Linux 本机 | CI + 真机 smoke；不虚报已验证 |
| 无 Azure client id | 前置说明；未配置禁用；不复用 HMCL |
| authlib 提供差异 | 多提供商测试；metadata 缓存；错误分层 |
| BMCLAPI 变化 | 表驱动；官方回退；root 可覆盖 |
| Forge 复杂 | v1 只启动已装；v1.1 安装；Phase 6 前补齐 |
| macOS Gatekeeper | ad-hoc + README；可选公证 |
| Linux GPU 依赖 | 文档/检测/明确错误；Ubuntu 22.04 构建 |
| GPL 边界 | 不复制 HMCL；GPL-3.0；NOTICE |
| OEM 滥用 | 签名数据；无远程命令；反回滚；hash/路径限制；token 不出本地 |
| 包体大 | 字体子集/系统回退；背景压缩；<150MB |
| 下载/磁盘 | 信号量/预检/.part/清缓存/进度合并 |
| Windows 命令行长度 | 记录风险；大实例测试；后续优化 |
| 用户期待安装器/自更新 | 明确 portable；OEM 提示新包；自更新后续 |

---

## 18. 默认假设

1. 许可证 GPL-3.0-or-later；不复制 HMCL 源码。
2. OEM.conf 在启动器根目录，只对该根目录生效；不读 CWD/AppData/注册表。
3. v1 原版 + Fabric + Quilt 安装；Forge/NeoForge 只启动已安装；原生安装 v1.1。
4. Microsoft 登录需要你提供 Azure client id；未配置禁用。
5. 产物仅 Windows zip、macOS zip、Linux tar.zst；无安装器。
6. 数据在 `<root>/SealyData`；根目录必须可写；不回退 AppData。
7. v1 深色 + zh-CN/en-US；浅色/更多语言后续。
8. v1 无自更新/遥测；OEM 内容更新 Phase 6。
9. 本机 Windows；mac/Linux 依赖 CI + 你提供实机。
10. UI 默认 gpui-kit 0.9.0；Phase 0 失败回退官方 gpui。

以上默认值已足够开始；若某项不符合，只需在审阅时改该项，不影响整体架构。

---

## 19. 批准后动作

1. 立即执行 **Step 0**：创建 `docs/PLAN.md`、`README.md`、`CHANGELOG.md`、`.gitignore`、`.gitattributes`；`git init -b main`；`git add`；有 Git 身份则首次提交，否则汇报等待你提供 name/email。
2. 更新 `CHANGELOG.md` 记录 Step 0；不安装 Rust/MSVC，不写业务代码。
3. 汇报文件、git 状态、待确认项，然后暂停。
4. 等你说“执行第一步”，再开始 Phase 0 Step 1（工具链 + workspace + hello window + CI + 动效验证）。

**CHANGELOG 从 Step 0 起生效；此后每个代码步骤都必须先写 CHANGELOG 再提交。**
