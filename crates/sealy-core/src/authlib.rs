//! authlib-injector: Yggdrasil authentication and the `-javaagent` hook.
//!
//! Third-party servers expose a Yggdrasil API root (often ending in
//! `/api/yggdrasil`). The launcher authenticates against it and passes the
//! account server to authlib-injector, which rewrites the game's auth calls.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::error::CoreError;
use crate::net;

/// Result of a Yggdrasil login.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Token the game authenticates with.
    pub access_token: String,
    /// Stable client token for refreshes.
    pub client_token: String,
    /// Dashed profile UUID.
    pub uuid: String,
    /// Player name.
    pub name: String,
}

/// Trims trailing slashes so `{api}/authserver/...` stays well formed.
pub fn normalize_api(api: &str) -> String {
    api.trim().trim_end_matches('/').to_owned()
}

/// Logs in with a username and password.
pub fn login(api: &str, username: &str, password: &str) -> Result<Session, CoreError> {
    let body = json!({
        "agent": { "name": "Minecraft", "version": 1 },
        "username": username,
        "password": password,
        "clientToken": format!("sealy-{}", uuid_like()),
        "requestUser": false
    });

    let response = net::post_json(
        &format!("{}/authserver/authenticate", normalize_api(api)),
        &body,
    )?;
    read_session(&response.body, response.status)
}

/// Refreshes an existing token.
pub fn refresh(api: &str, access_token: &str, client_token: &str) -> Result<Session, CoreError> {
    let body = json!({
        "accessToken": access_token,
        "clientToken": client_token,
        "requestUser": false
    });

    let response = net::post_json(&format!("{}/authserver/refresh", normalize_api(api)), &body)?;
    read_session(&response.body, response.status)
}

fn read_session(body: &serde_json::Value, status: u16) -> Result<Session, CoreError> {
    if !(200..300).contains(&status) {
        let message = body["errorMessage"]
            .as_str()
            .or_else(|| body["error"].as_str())
            .unwrap_or("外置登录失败");
        return Err(CoreError::Network(format!("{message}（HTTP {status}）")));
    }

    let access_token = body["accessToken"]
        .as_str()
        .ok_or_else(|| CoreError::Network("服务器没有返回 accessToken".to_owned()))?;
    let profile = body
        .get("selectedProfile")
        .ok_or_else(|| CoreError::Network("该账号没有绑定角色".to_owned()))?;

    Ok(Session {
        access_token: access_token.to_owned(),
        client_token: body["clientToken"].as_str().unwrap_or_default().to_owned(),
        uuid: dash_uuid(profile["id"].as_str().unwrap_or_default()),
        name: profile["name"].as_str().unwrap_or_default().to_owned(),
    })
}

/// Downloads the newest authlib-injector jar into `SealyData/authlib-injector`.
pub fn ensure_injector(root: &Path) -> Result<PathBuf, CoreError> {
    let directory = root.join("SealyData").join("authlib-injector");
    let latest = net::get_json(
        "https://authlib-injector.yushi.moe/artifact/latest.json",
        None,
    )?;
    if !latest.is_ok() {
        return Err(CoreError::Network(format!(
            "无法获取 authlib-injector 版本（HTTP {}）",
            latest.status
        )));
    }

    let version = latest.body["version"].as_str().unwrap_or("latest");
    let url = latest.body["download_url"]
        .as_str()
        .ok_or_else(|| CoreError::Network("authlib-injector 响应缺少下载地址".to_owned()))?;
    let target = directory.join(format!("authlib-injector-{version}.jar"));

    if target.is_file()
        && fs::metadata(&target)
            .map(|meta| meta.len() > 0)
            .unwrap_or(false)
    {
        return Ok(target);
    }

    fs::create_dir_all(&directory)?;
    let agent = net::agent()?;
    let mut file = fs::File::create(&target)?;
    net::download_to(&agent, url, &mut file, &mut |_| {})?;
    Ok(target)
}

/// `-javaagent:<jar>=<api>` for authlib-injector.
pub fn javaagent(jar: &Path, api: &str) -> String {
    format!("-javaagent:{}={}", jar.display(), normalize_api(api))
}

fn dash_uuid(raw: &str) -> String {
    if raw.len() == 32 {
        format!(
            "{}-{}-{}-{}-{}",
            &raw[0..8],
            &raw[8..12],
            &raw[12..16],
            &raw[16..20],
            &raw[20..32]
        )
    } else {
        raw.to_owned()
    }
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|delta| delta.as_nanos())
        .unwrap_or(0);
    format!("{nanos:032x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_api_roots() {
        assert_eq!(
            normalize_api(" https://littleskin.cn/api/yggdrasil/ "),
            "https://littleskin.cn/api/yggdrasil"
        );
    }

    #[test]
    fn builds_javaagent_argument() {
        let arg = javaagent(Path::new("C:/x/authlib.jar"), "https://a.cn/api/yggdrasil/");
        assert_eq!(
            arg,
            "-javaagent:C:/x/authlib.jar=https://a.cn/api/yggdrasil"
        );
    }

    #[test]
    fn dashes_profile_ids() {
        assert_eq!(
            dash_uuid("b50ad385829d3141a2167e7d7539ba7f"),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
    }

    #[test]
    fn reads_authlib_error_message() {
        let body = serde_json::json!({ "error": "ForbiddenOperationException", "errorMessage": "密码错误" });
        let err = read_session(&body, 403).unwrap_err();
        assert!(err.to_string().contains("密码错误"));
    }

    /// Network test: needs internet, run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn live_login_reports_bad_credentials() {
        let err = login(
            "https://littleskin.cn/api/yggdrasil",
            "sealy-nonexistent",
            "bad",
        )
        .unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    /// Network test: downloads the injector jar.
    #[test]
    #[ignore]
    fn live_downloads_injector() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        let jar = ensure_injector(&root).unwrap();
        assert!(jar.is_file());
        assert!(
            std::fs::metadata(&jar)
                .map(|meta| meta.len() > 100_000)
                .unwrap_or(false),
            "jar too small: {}",
            jar.display()
        );
    }
}
