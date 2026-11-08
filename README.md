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

## CI / 发布

- [CI](.github/workflows/ci.yml)：Windows 运行 fmt/clippy/test/build；macOS/Linux 目前只构建，测试暂不执行。
- [Release](.github/workflows/release.yml)：推送 `v*` tag 时生成 portable 包：
  - Windows x64 `.zip`
  - macOS universal `.zip`（含 `.app`）
  - Linux x86_64 `.tar.zst`
- 发布产物包含 `README.md`、`LICENSE`、`THIRD_PARTY_NOTICES.md`、`OEM.conf.example`，并生成 `SHA256SUMS`。
- 不产出 MSI/DMG/DEB/RPM/AppImage 安装包。

## 文档

- [实施计划](docs/PLAN.md)
- [变更日志](CHANGELOG.md)
- [第三方声明](THIRD_PARTY_NOTICES.md)
- [GPL-3.0 许可证](LICENSE)

## 许可

项目采用 `GPL-3.0-or-later`，见 [LICENSE](LICENSE)。HMCL 仅作为协议与行为参考，不复制其源码。

## 变更记录约定

从 Step 0 开始，每个代码步骤都必须先更新 [CHANGELOG.md](CHANGELOG.md)，再提交。提交信息使用 Conventional Commits。
