# 组件化盘点（2026-11-15）

目标：应用内所有标准控件、浮层与输入都使用 GPUI Kit / gpui-component 提供的现成组件；
账户列表与实例列表支持滚动且行高不被压缩。

## 控件映射

| 界面能力 | 使用组件 | 位置 |
| --- | --- | --- |
| 文本输入 / 记住输入 | `Input` + `InputState` | `crates/sealy-ui/src/inputs.rs`（账号、代理、游戏目录、内存、JVM、窗口尺寸、重命名等） |
| 普通按钮 | `Button`（`components::UiButton` 适配层） | `primary_button` / `quiet_button` / `choice_button` / `icon_button` / `chip_button` / `nav_button` / `card_button` / `tile_button` / `value_button` / `prompt_button` |
| 开关 | `Switch` | 设置页：背景音乐、启动检查更新、调试日志 |
| 单选组 | `RadioGroup` + `Radio` | OOBE「音乐与音效」 |
| 下拉选择 | `Select` + `SelectState` | 设置页下载源（自动 / 官方 / BMCLAPI），随语言重建 |
| 进度 | `Progress` | 设置页更新下载、消息中心任务进度 |
| 加载状态 | `Spinner` | 消息中心运行中任务图标 |
| 头像 | `Avatar` | 账号列表行 |
| 通知 / 撤销 | `Notification` + 组件 `Button` | 全局 toast，含「撤销」action |
| 侧滑面板 | `Sheet` | 消息中心、实例启动设置、游戏日志、崩溃分析 |
| 模态确认 | `Dialog` | 删除实例确认（三按钮自定义 footer） |
| 分区导航 | `TabBar` + `Tab` | 实例详情：概览 / 模组 / 资源包 / 光影 |
| 滚动 | `ScrollableElement::overflow_y_scrollbar` | 账号、实例、下载、设置、消息、日志、崩溃等列表/面板 |
| 窗口标题栏 | `TitleBar`（组件自带窗口控制按钮） | 顶部品牌、铃铛与窗口按钮 |
| 内存滑杆 | gpui-base `Slider` | 设置页 Java 内存上限 |
| 图标 | `Icon` / `IconName` | 全站图标 |

## 列表滚动（目标第 2 项）

- 账号列表、实例列表、下载列表、设置内容、消息、日志、崩溃报告均使用 `overflow_y_scrollbar`，滚动条可见。
- 列表行加 `flex_shrink_0()`，固定 72px/60px 行高，内容超出时滚动而不是压缩。
- 实测：10 个账号滚轮滚动、13 个实例滚轮滚动，行高保持正常，滚动条可见。
- 证据：`design/preview/gpui-components-scroll.png`、`target/audit-accounts-scroll.png`、`target/audit-instances-scroll.png`。

## 说明

- 账号行、实例行、下载版本行属于应用自定义的列表行内容（选择态 + 元数据 + 操作按钮），不是标准控件；
  行内所有按钮均为组件 Button，滚动行为由组件滚动容器提供。
- OOBE「需要继续安装 Minecraft 核心吗？」是内联文字链接样式，不是按钮控件；其跳转逻辑仍保留。
- 页面内联表单（添加账号、外置登录、新建实例）不是浮层，内部输入与按钮均为组件。

## 验证

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`（80 通过 / 2 忽略）
- Windows 实机逐项操作截图见 `design/preview/gpui-components-*.png`。
