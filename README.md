# SealyLauncher

Rust + GPUI 的便携式 Minecraft 启动器，目标平台为 Windows、macOS 和 Linux。

> 本项目与 Mojang Studios、Microsoft 没有隶属关系；Minecraft 是 Mojang Studios 的商标。

## 当前状态

- 阶段：Step 0 — 仓库初始化
- 实施计划：[docs/PLAN.md](docs/PLAN.md)
- 变更日志：[CHANGELOG.md](CHANGELOG.md)

Step 0 只建立文档、变更日志和 Git 仓库；Rust 工具链、Cargo workspace 与 UI 代码从 Phase 0 开始。

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
- 精确版本在 Phase 0 验证后写入 `rust-toolchain.toml` 和 `Cargo.lock`。

## 文档

- [实施计划](docs/PLAN.md)
- [变更日志](CHANGELOG.md)

## 许可

项目计划采用 `GPL-3.0-or-later`。HMCL 仅作为协议与行为参考，不复制其源码。

## 变更记录约定

从 Step 0 开始，每个代码步骤都必须先更新 [CHANGELOG.md](CHANGELOG.md)，再提交。提交信息使用 Conventional Commits。
