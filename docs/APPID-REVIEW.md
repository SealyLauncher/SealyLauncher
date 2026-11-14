# AppID Review 表单填写参考

表单地址：<https://aka.ms/mce-reviewappid>
（跳转到 Microsoft Forms，标题 **AppID Review**，共 9 个问题，每周审核一次）

> 本文件只是填写参考。提交前请确认你**有权管理该 App 注册**（表单会用联系人邮箱去 Azure Portal 交叉核对）。

## 表单结构与逐项答案

| # | 问题 | 是否必填 | 建议填写 |
| --- | --- | --- | --- |
| 1 | What type of request is this? | 必填 | **New AppID for Approval**（首次申请；被举报才选 Existing AppID for Review/Report） |
| 2 | Application Name | 必填 | `SealyLauncher`（不得包含 Mojang / Minecraft / Microsoft / Live / Xbox / Discord / Hypixel） |
| 3 | Application ID | 必填 | `e95460ea-8dcc-4dbc-bc2c-b823b9035ccb`（一次只能提交一个 AppID） |
| 4 | I verify that I have read and understood the EULA and all Usage Guidelines | 必填 | **Yes** |
| 5 | Associated website or domain | 必填 | `https://github.com/SealyLauncher/SealyLauncher` |
| 6 | Justification | 必填 | 见下方「Justification 文案」 |
| 7 | Tenant ID | 新申请必填 | 见下方「如何找到 Tenant ID」 |
| 8 | Contact Information (Valid email address) | 必填 | 你的邮箱，且**必须与 Azure 账号 / 应用所有者一致** |
| 9 | Any other information that we should be aware of? | 选填 | 见下方「补充说明文案」 |

## Justification 文案（英文，可直接粘贴）

```text
SealyLauncher is a free, open-source (GPL-3.0) Minecraft: Java Edition launcher for
Windows, macOS and Linux, published as portable builds from
https://github.com/SealyLauncher/SealyLauncher. The source code, releases and issue
tracker are public.

Microsoft sign-in uses the standard OAuth 2.0 device-code flow against the consumers
tenant with the XboxLive.signin scope, then Xbox Live
(https://user.auth.xboxlive.com/user/authenticate), XSTS
(https://xsts.auth.xboxlive.com/xsts/authorize, relying party
rp://api.minecraftservices.com/) and finally
POST https://api.minecraftservices.com/authentication/login_with_xbox with the
XBL3.0 identity token. The resulting Minecraft access token is used only to read
GET https://api.minecraftservices.com/minecraft/profile and to start the official
game client files for the account that signed in.

We request access because without an allow-listed AppID every sign-in attempt is
rejected with HTTP 403 "Invalid app registration", so users cannot play with the
Microsoft account they already own and pay for.

The launcher does not bypass, disable or weaken authentication, ownership or licence
checks; it does not distribute or patch game files; it requires a Microsoft account
that owns Minecraft for online play. Local/offline profiles exist only for
third-party servers that do not use Mojang authentication.
```

## 补充说明文案（英文，可选但建议填）

```text
This AppID was already included in a previous review round: the review-complete
notification ("AppID Review Complete (11.10.2026)") arrived on 2026-11-10 17:45 UTC.

As of 2026-11-14 13:36 UTC the API still rejects it:

POST https://api.minecraftservices.com/authentication/login_with_xbox
403 {"path":"/authentication/login_with_xbox","errorMessage":"Invalid app registration,
see https://aka.ms/AppRegInfo for more information"}

For the same Microsoft account the preceding steps succeed:
- https://user.auth.xboxlive.com/user/authenticate -> 200
- https://xsts.auth.xboxlive.com/xsts/authorize -> 200 (user hash present, XErr 0)

So the account is in good standing and only the AppID is rejected. The application
registration has not been renamed, recreated or reconfigured since the previous
submission: it is a public client (device-code flow works, client id accepted by
login.microsoftonline.com/consumers) with personal Microsoft accounts enabled.

We are resubmitting the same AppID unchanged to have it added to the allow list.
```

## 如何找到 Tenant ID

Azure Portal → **Microsoft Entra ID** → 概览（Overview）→ **Tenant ID / Directory ID**，
或者：App registrations → 目标应用 → 概览页，第一行就是 **Directory (tenant) ID**。

如果不是你创建的应用，你多半拿不到它的 tenant id，也过不了表单的联系人核验——
这种情况见下面「提交前必须确认的一件事」。

## 提交前必须确认的一件事

表单第 8 项会用**邮箱去 Azure Portal 交叉核对应用所有者**。所以：

- **情况 A：`e95460ea-…` 是你自己创建/有权限的应用**
  按上表填，Tenant ID 用该应用所在目录的 ID，邮箱用该 Azure 账号的邮箱。

- **情况 B：`e95460ea-…` 是项目自带的应用，你没有权限**
  你不能替别人提交，也不该提交别人的 AppID。正确做法是：
  1. 自己注册一个 Azure 应用（个人 Microsoft 帐户、允许公共客户端流）；
  2. 用**你自己的 client id** 去提交审核；
  3. 审核通过后，把启动器指到它：
     `SealyData/settings.json` →
     `"auth": { "microsoft_client_id": "<你的 client id>" }`
     设置页的「Microsoft 客户端 ID」一行可以核对是否生效。

## 其他注意事项

- 表单顶部明确：**每周审核一次**，重复提交不会加快进度，反而可能被忽略。
- 应用名不得包含 Mojang / Minecraft / Microsoft / Live / Xbox / Discord / Hypixel。
- 必须能说明「验证游戏所有权、要求先登录才可进入游戏」；绕过或关闭认证/授权检查的应用不会被批准。
- 审核方不提供 API 访问的技术支持，回信追问一般无效，唯一动作是重新提交。
- 提交后如果又收到 `AppID Review Complete`，仍以 API 是否放行为准：
  `POST /authentication/login_with_xbox` 返回 200 才算通过；403 `Invalid app registration` 表示仍未加入白名单。
