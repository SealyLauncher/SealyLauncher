# 代码签名与分发可信度

本仓库当前产出的可执行文件**没有**商业代码签名：

- Windows：`sealy-launcher.exe` 未签名；首次运行或每次更新后可能触发 SmartScreen
  “Windows 已保护你的电脑”（用户需要点“更多信息 → 仍要运行”）。
- macOS：CI 使用 `codesign --force --deep --sign -`（ad-hoc 签名），只保证 bundle
  完整性，不提供 Gatekeeper 认可；首次打开仍可能提示“无法验证开发者”。
- Linux：无可执行签名惯例，发布物依赖 SHA-256 校验。

自更新流程已经会校验 `SHA256SUMS`，这能防止下载损坏或镜像篡改，但**不能替代**
操作系统级的开发者身份签名。下面是获得签名的常见途径与接入方式。

> 价格、资格与政策会变化，请以官方页面为准；下面给出的是选择与操作路径。

## Windows

### 选项一：OV / EV 代码签名证书（传统方式）

- 向 DigiCert、Sectigo、SSL.com、GlobalSign 等 CA 购买 **Code Signing Certificate**。
  - OV（Organization Validation）：约 200–500 美元/年，私钥可放软件/HSM；
  - EV（Extended Validation）：约 300–700 美元/年，通常要求硬件令牌或云 HSM，
    审核更严（通常要求公司主体与 2–3 年经营记录）。
- 个人开发者通常只能申请 OV（并且部分 CA 要求组织主体）；EV 基本要求公司。
- 使用 `signtool sign /fd SHA256 /tr <时间戳服务器> /td SHA256 /f cert.pfx /p <密码> sealy-launcher.exe`。
- SmartScreen 信誉：OV 签名仍需累积下载信誉；EV 签名（或 Azure Trusted Signing
  的“成熟”账户）通常能较快建立信誉。**签名不能保证立刻没有警告。**

### 选项二：Azure Trusted Signing（云端签名，推荐个人/小团队）

- Microsoft 的托管签名服务，按签名次数/月订阅计费（约 10 美元/月量级）。
- 需要 Azure 订阅、身份验证（个人通常需要 3 年以上可验证历史；公司需要注册资料）。
- 在 GitHub Actions 里用 `azure/trusted-signing-action` 签名，无需把私钥放进仓库。
- 优势：没有硬件令牌、证书自动轮换、CI 友好。

### 选项三：开源项目免费/低价方案

- **SignPath.io** 对开源项目提供免费代码签名（需申请并审核项目）。
- **Certum Open Source Code Signing** 对开源项目有低价证书（约 80 欧元/年量级）。
- 适合本项目（GPL-3.0）当前阶段：成本最低，但仍需完成 CA 审核。

### 接入 GitHub Actions 的流程（以 PFX 为例）

1. 生成/购买证书，导出 `.pfx` 与密码。
2. 在仓库 Settings → Secrets and variables → Actions 添加：
   - `WINDOWS_CERT_PFX_BASE64`（`[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))` 的结果）
   - `WINDOWS_CERT_PASSWORD`
3. 在 `release.yml` 的 Windows 打包步骤里、`Compress-Archive` 之前解码并签名：

   ```powershell
   $pfx = "$env:RUNNER_TEMP\cert.pfx"
   [IO.File]::WriteAllBytes($pfx, [Convert]::FromBase64String($env:WINDOWS_CERT_PFX_BASE64))
   & signtool sign /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com /f $pfx /p $env:WINDOWS_CERT_PASSWORD target/release/sealy-launcher.exe
   ```

4. 用 CI 的 `Release` 工作流只在 tag/主分支构建时签名，避免泄露风险。

## macOS

- 加入 **Apple Developer Program**（约 99 美元/年），申请 **Developer ID Application**
  证书；只有该证书签名并经过公证的 App 才能在 Gatekeeper 下正常分发。
- 签名（在现有 ad-hoc 签名位置替换）：

  ```bash
  codesign --force --deep --options runtime --timestamp \
    --sign "Developer ID Application: Your Name (TEAMID)" "dist/SealyLauncher.app"
  codesign --verify --deep --strict --verbose=2 "dist/SealyLauncher.app"
  ```

- 公证与装订（notarytool，Apple ID + app-specific password 或 App Store Connect API Key）：

  ```bash
  ditto -c -k --keepParent "dist/SealyLauncher.app" "dist/notarize.zip"
  xcrun notarytool submit "dist/notarize.zip" --apple-id "$APPLE_ID" \
    --team-id "$TEAM_ID" --password "$APP_PASSWORD" --wait
  xcrun stapler staple "dist/SealyLauncher.app"
  ```

- 然后再打 zip/tar.gz；CI 需要把证书 `.p12`（base64）与密码、Apple ID 凭据存到
  GitHub Secrets，并在 macOS runner 上导入临时钥匙串。
- 注意：自更新替换 `.app` 后，新 bundle 仍保留签名；更新脚本会执行 `xattr -cr`
  清除隔离属性。若签名证书改变，旧版本更新到新版本时会重新校验签名，属正常。

## Linux

- 常见做法是发布 **GPG 签名的 `SHA256SUMS`**（以及可选的 `.sig`）：

  ```bash
  gpg --armor --detach-sign --output SHA256SUMS.asc SHA256SUMS
  ```

- 把公钥发布到仓库 README 或 keyserver；打包脚本/用户可用
  `gpg --verify SHA256SUMS.asc SHA256SUMS` 校验再安装。
- 如果未来做 `.deb`/`.rpm`，可用发行版仓库签名（debsign/rpmsign），成本低但不是
  跨平台身份。

## 本项目建议的落地顺序

1. **现在即可做**：把 `SHA256SUMS` 用 GPG 签名并发布公钥（Linux/所有平台的完整性）。
2. **低成本下一步**：申请 SignPath / Certum 开源签名（Windows、macOS），接入
   `release.yml` 的 tag 构建；保持 SHA-256 校验作为第二道防线。
3. **有公司主体后**：Azure Trusted Signing（Windows）或 EV 证书；macOS 用
   Developer ID + notarytool。

签名只影响“首次信任”，用户仍可能看到 SmartScreen（取决于下载量/信誉）；
自更新流程的哈希校验、失败回滚与更新日志是本仓库已经实现的兜底。
