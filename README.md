# SealyLauncher

Rust + GPUI 的便携式 Minecraft 启动器，目标平台为 Windows、macOS 和 Linux。

> 本项目与 Mojang Studios、Microsoft 没有隶属关系；Minecraft 是 Mojang Studios 的商标。

## 当前状态

- 阶段：Phase 1 — 设计系统、导航、页面骨架与便携发布流程
- Rust：`1.99.1`（MSVC toolchain，`rust-toolchain.toml` 固定）
- 实施计划：[docs/PLAN.md](docs/PLAN.md)
- 变更日志：[CHANGELOG.md](CHANGELOG.md)

Phase 0 已安装 Rust + Visual Studio 2022 Build Tools（C++ 桌面开发 + Windows SDK），
并建立 `sealy-core` / `sealy-ui` / `sealy-app` workspace。
Phase 1 已接入 portable 路径/设置/日志/单实例、GPUI 设计系统、NavRail、
Home/实例/下载/账号/设置页面，以及 Windows/macOS/Linux 的 CI 与 portable 发布 workflow。
Windows 本地 `cargo fmt` / `clippy` / `test` / `build` 与窗口 smoke 已通过；
macOS/Linux 目前只配置构建与打包任务，测试按约定暂不执行。

## 目标

- 便携运行：所有启动器数据位于程序目录的 `SealyData/`，不写注册表、不写全局 `.minecraft`、不与其它启动器争用文件。
- 三种登录：离线、Microsoft、authlib-injector。
- 下载源：官方源、BMCLAPI，自动选择并支持回退。
- 原版、Fabric、Quilt 安装与启动；可链接已有 `.minecraft` 实例。
- 自定义 GPUI 界面与动效，不使用装饰性的小标题/副标题堆砌层级。
- 为后续 OEM 远程统一控制、服务器整合包自动更新和 Node.js 管理端预留接口。

## 构建前置（Phase 0 起）

- Windows：Rust stable、Visual Studio 2022 Build Tools（C++ 桌面开发）、Windows SDK。
- macOS：Rust stable、Xcode / Command Line Tools。
- Linux：Rust stable，以及 GPUI 所需的 X11/Wayland、Vulkan、字体开发库。
- 精确版本在 `rust-toolchain.toml` 固定，依赖由 `Cargo.lock` 锁定。

## 设计方向验证

```powershell
cargo run -p sealy-app
```

打开真实窗口：主页为磁贴布局，设置页可切换 4 套主题（纸张 / 墨色 / 陶土 / 青瓷），
选择立即写入 `SealyData/settings.json`。
静态预览在 [design/preview/home-v4.html](design/preview/home-v4.html) 与
[design/preview/oobe-v4.html](design/preview/oobe-v4.html)，界面规范见
[docs/PLAN.md §9](docs/PLAN.md)。

## CI / 发布

- [CI](.github/workflows/ci.yml)：Windows 运行 fmt/clippy/test/build；macOS/Linux 只构建。
- [Release](.github/workflows/release.yml)：**每次推送到 `main` 自动发版**
  1. 读取 `Cargo.toml` 里的版本号，patch +1，提交 `chore(release): vX.Y.Z [skip ci]` 并打 tag；
  2. 三平台并行构建：Windows x64 `.zip`、macOS universal `.zip`（含 `.app`）、Linux x86_64 `.tar.zst`；
  3. 生成 `SHA256SUMS` 并发布到 GitHub Release。
- 也可以手动触发，`bump` 可选 `patch` / `minor` / `major` / `none`。
- 图标：`packaging/windows/sealy.ico` 由 `crates/sealy-app/build.rs` 在检测到 `rc.exe` 时嵌入 exe；
  macOS 用 `SealyLauncher.icns`，Linux 附带 `sealy-launcher.png`。
- 产物包含 `README.md`、`LICENSE`、`THIRD_PARTY_NOTICES.md`、`OEM.conf.example`。
- 不产出 MSI/DMG/DEB/RPM/AppImage 安装包。

## Microsoft 登录返回 403

`api.minecraftservices.com` 只放行通过 Minecraft API 审核的 client id。收到 403 时服务端仍未认可当前应用。按顺序排查：

1. **确认启动器实际用的 ID**：设置页「Microsoft 客户端 ID」一行就是运行时真实值。若这里是别的值（例如旧版 `SealyData/settings.json` 里残留的自建应用 ID），改 `auth.microsoft_client_id` 或删掉 `auth` 块即可回到默认 `e95460ea-8dcc-4dbc-bc2c-b823b9035ccb`。
2. **对照官方启动器**：用同一个微软账号登录官方启动器。
   - 官方能登录、这里 403 → 问题在应用（ID 不对 / 审批未生效）。
   - 官方也失败 → 问题在账号（封禁、地区、儿童账号等），与 client id 无关。
3. **核对邮件措辞**：Mojang Enforcement 的「approved」邮件才会放行；「request received / under review」还不是通过。审批生效通常数小时到一天，偶尔更久。
4. **看服务端原话**：登录失败会在消息中心留下一条记录，包含 `客户端 ID：…` 与 `服务端返回：FORBIDDEN: …`。把这一行发出来即可判断是应用还是账号。
5. **日志**：设置页打开「调试日志」后重试，`SealyData/logs/launcher.log.*` 会记录每一步的 `microsoft: …` 行（状态码 + 脱敏后的响应体），其中 `microsoft: minecraft login_with_xbox finished` 的 `status` 就是 403 的来源。

请求本身已带 `User-Agent`、`Content-Type`、`Accept`；用无效 token 探测得到的是 401 而不是 403，所以 403 不是请求格式或网络层问题。客户端其他环节（设备码、Xbox Live、XSTS）失败会给出各自错误码，不会落到 403。
## 文档

- [实施计划](docs/PLAN.md)
- [变更日志](CHANGELOG.md)
- [第三方声明](THIRD_PARTY_NOTICES.md)
- [GPL-3.0 许可证](LICENSE)

## 许可

项目采用 `GPL-3.0-or-later`，见 [LICENSE](LICENSE)。HMCL 仅作为协议与行为参考，不复制其源码。

## 变更记录约定

从 Step 0 开始，每个代码步骤都必须先更新 [CHANGELOG.md](CHANGELOG.md)，再提交。提交信息使用 Conventional Commits。
