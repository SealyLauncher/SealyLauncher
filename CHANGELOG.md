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

### Verified

- `git init -b main` 成功。
- 工作区文件清单：`README.md`、`CHANGELOG.md`、`docs/PLAN.md`、`.gitignore`、`.gitattributes`。
- 上述文件已 `git add` 暂存。
- 首次提交：`chore: bootstrap repository and implementation plan`，作者 `Tasuku Hachisuka <hachisukaauto@mingze.de>`（仓库本地配置，不修改全局 Git 身份）。

### Notes

- Git 作者身份已配置为仓库本地：`Tasuku Hachisuka <hachisukaauto@mingze.de>`；未修改全局配置。
- Step 0 不安装 Rust/MSVC，不创建 Cargo workspace，不写业务代码。
- 下一步：等待用户指令“执行第一步”，进入 Phase 0 Step 1（工具链 + workspace + GPUI hello window + CI 骨架 + 动效验证）。
