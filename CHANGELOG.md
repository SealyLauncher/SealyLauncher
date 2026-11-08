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

### Notes

- Git 作者身份已配置为仓库本地：`Tasuku Hachisuka <hachisukaauto@mingze.de>`；未修改全局配置。
- Step 0 不安装 Rust/MSVC，不创建 Cargo workspace，不写业务代码。
- Phase 0 Step 1 的 Windows 本地构建与窗口 smoke 已完成；`ci.yml` 已添加但尚未在远端 GitHub Actions 执行，macOS/Linux 真机 smoke 仍待完成。
- 当前为 debug 构建；未做 release 打包、签名或 60fps 正式帧时间测量。窗口截图保存在 `target/phase0-motion3.png`（未提交，避免包含桌面隐私）。
- 下一步：完成 Phase 0 剩余 gate（远端 CI 三平台编译 + macOS/Linux smoke），然后进入 Phase 1。
