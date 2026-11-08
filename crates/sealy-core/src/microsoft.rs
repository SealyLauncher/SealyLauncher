//! Microsoft / Xbox Live / Minecraft authentication.
//!
//! Device-code flow so the launcher never needs a local HTTP server:
//! MSA token -> Xbox Live -> XSTS -> Minecraft services -> profile.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::net;

/// Endpoint that hosts the OAuth endpoints. Personal accounts only: the
/// `XboxLive.signin` scope does not work with work/school tenants.
const AUTH_HOST: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0";
/// Scopes required to reach Xbox Live and refresh silently.
const SCOPE: &str = "XboxLive.signin offline_access";

/// Pending device-code sign-in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCode {
    /// Long code polled by the launcher.
    pub device_code: String,
    /// Short code the user types in the browser.
    pub user_code: String,
    /// Page to open.
    pub verification_uri: String,
    /// Seconds between polls.
    pub interval: u64,
    /// Seconds until the code expires.
    pub expires_in: u64,
}

/// Result of one poll.
#[derive(Debug, Clone)]
pub enum Poll {
    /// User has not finished yet.
    Pending,
    /// Microsoft asks to slow down.
    SlowDown,
    /// Sign-in completed.
    Ready(MsaToken),
}

/// Microsoft account tokens.
#[derive(Debug, Clone)]
pub struct MsaToken {
    /// Bearer token for Xbox Live.
    pub access_token: String,
    /// Token used to refresh later; absent for some flows.
    pub refresh_token: Option<String>,
}

/// Minecraft services session.
#[derive(Debug, Clone)]
pub struct MinecraftSession {
    /// Bearer token the game uses.
    pub access_token: String,
    /// Seconds until it expires.
    pub expires_in: u64,
}

/// Player profile from Minecraft services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Undashed UUID.
    pub id: String,
    /// Current player name.
    pub name: String,
}

/// Starts a device-code sign-in.
pub fn request_device_code(client_id: &str) -> Result<DeviceCode, CoreError> {
    let url = format!("{AUTH_HOST}/devicecode");
    let response = net::post_form(&url, &[("client_id", client_id), ("scope", SCOPE)])?;

    if !response.is_ok() {
        return Err(CoreError::Network(error_message(&response.body)));
    }

    Ok(DeviceCode {
        device_code: string_field(&response.body, "device_code"),
        user_code: string_field(&response.body, "user_code"),
        verification_uri: string_field(&response.body, "verification_uri"),
        interval: response.body["interval"].as_u64().unwrap_or(5),
        expires_in: response.body["expires_in"].as_u64().unwrap_or(900),
    })
}

/// Polls the token endpoint once.
pub fn poll_device_code(client_id: &str, code: &DeviceCode) -> Result<Poll, CoreError> {
    let url = format!("{AUTH_HOST}/token");
    let response = net::post_form(
        &url,
        &[
            ("client_id", client_id),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", &code.device_code),
        ],
    )?;

    if response.is_ok() {
        return Ok(Poll::Ready(MsaToken {
            access_token: string_field(&response.body, "access_token"),
            refresh_token: response.body["refresh_token"].as_str().map(str::to_owned),
        }));
    }

    match response.body["error"].as_str().unwrap_or("") {
        "authorization_pending" => Ok(Poll::Pending),
        "slow_down" => Ok(Poll::SlowDown),
        other => Err(CoreError::Network(format!(
            "device code poll failed: {other} ({})",
            error_message(&response.body)
        ))),
    }
}

/// Refreshes an MSA token.
pub fn refresh(client_id: &str, refresh_token: &str) -> Result<MsaToken, CoreError> {
    let url = format!("{AUTH_HOST}/token");
    let response = net::post_form(
        &url,
        &[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", SCOPE),
        ],
    )?;

    if !response.is_ok() {
        return Err(CoreError::Network(error_message(&response.body)));
    }

    Ok(MsaToken {
        access_token: string_field(&response.body, "access_token"),
        refresh_token: response.body["refresh_token"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| Some(refresh_token.to_owned())),
    })
}

/// Exchanges an MSA token for an Xbox Live token and user hash.
pub fn xbox_live(msa_access_token: &str) -> Result<(String, String), CoreError> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={msa_access_token}")
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let response = net::post_json("https://user.auth.xboxlive.com/user/authenticate", &body)?;
    if !response.is_ok() {
        return Err(CoreError::Network(error_message(&response.body)));
    }

    Ok((
        string_field(&response.body, "Token"),
        user_hash(&response.body),
    ))
}

/// Exchanges an Xbox Live token for an XSTS token.
pub fn xsts(xbl_token: &str) -> Result<(String, String), CoreError> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let response = net::post_json("https://xsts.auth.xboxlive.com/xsts/authorize", &body)?;
    if !response.is_ok() {
        return Err(CoreError::Network(xsts_error(&response.body)));
    }

    Ok((
        string_field(&response.body, "Token"),
        user_hash(&response.body),
    ))
}

/// Logs into Minecraft services with an XSTS token.
pub fn login_with_xbox(user_hash: &str, xsts_token: &str) -> Result<MinecraftSession, CoreError> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
    });

    let response = net::post_json(
        "https://api.minecraftservices.com/authentication/login_with_xbox",
        &body,
    )?;

    if response.status == 403 {
        return Err(CoreError::Network(
            "Minecraft 拒绝了该应用（403）：Azure 应用尚未通过 Minecraft API 审核".to_owned(),
        ));
    }
    if !response.is_ok() {
        return Err(CoreError::Network(error_message(&response.body)));
    }

    Ok(MinecraftSession {
        access_token: string_field(&response.body, "access_token"),
        expires_in: response.body["expires_in"].as_u64().unwrap_or(86_400),
    })
}

/// Reads the player profile.
pub fn profile(minecraft_token: &str) -> Result<Profile, CoreError> {
    let response = net::get_json(
        "https://api.minecraftservices.com/minecraft/profile",
        Some(minecraft_token),
    )?;

    if response.status == 404 {
        return Err(CoreError::Network(
            "该账号没有 Minecraft 档案（可能未购买或未激活 Xbox Game Pass）".to_owned(),
        ));
    }
    if !response.is_ok() {
        return Err(CoreError::Network(error_message(&response.body)));
    }

    Ok(Profile {
        id: string_field(&response.body, "id"),
        name: string_field(&response.body, "name"),
    })
}

/// Runs the whole chain from an MSA token.
pub fn complete_login(msa: &MsaToken) -> Result<(Profile, MinecraftSession), CoreError> {
    let (xbl, _) = xbox_live(&msa.access_token)?;
    let (xsts_token, uhs) = xsts(&xbl)?;
    let session = login_with_xbox(&uhs, &xsts_token)?;
    let profile = profile(&session.access_token)?;
    Ok((profile, session))
}

fn user_hash(body: &serde_json::Value) -> String {
    body["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn string_field(body: &serde_json::Value, key: &str) -> String {
    body[key].as_str().unwrap_or_default().to_owned()
}

fn error_message(body: &serde_json::Value) -> String {
    let code = body["error"].as_str().unwrap_or("unknown");
    let description = body["error_description"]
        .as_str()
        .or_else(|| body["errorMessage"].as_str())
        .unwrap_or("");
    if description.is_empty() {
        format!("认证失败：{code}")
    } else {
        format!("认证失败：{code} · {description}")
    }
}

fn xsts_error(body: &serde_json::Value) -> String {
    let code = body["XErr"].as_u64().unwrap_or(0);
    let reason = match code {
        2_148_916_227 => "该账号被 Xbox 封禁",
        2_148_916_233 => "该账号还没有 Xbox 档案，请先登录 minecraft.net 创建",
        2_148_916_235 => "所在地区不支持 Xbox Live",
        2_148_916_236 | 2_148_916_237 => "需要在 Xbox 页面完成成人验证",
        2_148_916_238 => "儿童账号需要家长加入家庭组后才能登录",
        _ => "Xbox 认证失败",
    };
    format!("{reason}（XErr {code}）")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_user_hash() {
        let body = serde_json::json!({
            "DisplayClaims": { "xui": [ { "uhs": "abc123" } ] },
            "Token": "T"
        });
        assert_eq!(user_hash(&body), "abc123");
    }

    #[test]
    fn explains_xsts_errors() {
        let body = serde_json::json!({ "XErr": 2_148_916_238u64 });
        let message = xsts_error(&body);
        assert!(message.contains("儿童账号"));
    }

    #[test]
    fn extracts_oauth_error() {
        let body = serde_json::json!({
            "error": "invalid_client",
            "error_description": "Client not found"
        });
        let message = error_message(&body);
        assert!(message.contains("invalid_client"));
        assert!(message.contains("Client not found"));
    }
}
