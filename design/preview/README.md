# UI 方向预览

三个视觉方向的静态预览，用于在投入 Rust 实现前确认方向。

## 打开方式

直接双击 `index.html`，或：

```powershell
Start-Process .\design\preview\index.html
```

顶部胶囊按钮可在三个方向之间切换；点击导航、开始游戏、账号会展示对应状态。

## 五个方向

| 编号 | 名称 | 特征 | 参考气质 |
| --- | --- | --- | --- |
| 01 | 深海玻璃 | 全屏场景图 + 暗角 + 玻璃面板 + 单一青色强调 | 现代游戏启动器、沉浸式主界面 |
| 02 | 流畅霓虹 | 分层表面 + 亚克力质感 + 紫粉渐变强调 + 侧栏展开 | Fluent / 桌面客户端 |
| 03 | 杂志深色 | 纯排版驱动：大字重、细线、编号元数据、极少装饰 | Linear / Vercel |
| 04 | Fluent 2 | Windows 11 语言：Mica 底、4/8px 圆角、1px 描边、Segoe UI、accent 填充按钮、NavigationView 选中条 | Windows 11 原生应用 |
| 05 | Material Design 3 | 深色基线紫（seed #6750A4）：tonal surface、全圆角按钮、导航栏胶囊指示、FAB、state layer | Android / Material You |

## 说明

- 三份预览共用同一套 DOM，靠 CSS 变量与方向选择器切换，不代表最终代码结构。
- 图片部分用 SVG 与 CSS 渐变生成，不依赖网络资源，离线可看。
- 截图由 headless Edge 以 1280×800 生成：`direction-1.png` ～ `direction-5.png`。
- 确认方向后再落到 `sealy-ui`，并用 GPUI Kit 的主题 token（颜色/圆角/阴影/间距/动效）重建。
- `05 · Material 3` 使用的是 M3 深色基线配色（seed `#6750A4`）；实际实现时由 seed 生成 tonal palette。
