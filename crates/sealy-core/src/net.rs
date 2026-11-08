//! HTTP helpers shared by the metadata and download paths.

use std::io::Read;
use std::time::Duration;

use crate::error::CoreError;

/// Builds a `ureq` agent with the native TLS backend and sane timeouts.
pub fn agent() -> Result<ureq::Agent, CoreError> {
    let connector = ureq::native_tls::TlsConnector::new()
        .map_err(|err| CoreError::Network(format!("tls backend: {err}")))?;

    Ok(ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(connector))
        .timeout_connect(Duration::from_secs(8))
        .timeout(Duration::from_secs(60))
        .build())
}

/// Fetches a URL as text.
pub fn get_text(url: &str) -> Result<String, CoreError> {
    let agent = agent()?;
    agent
        .get(url)
        .call()
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))?
        .into_string()
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))
}

/// Fetches a URL and streams it into `sink`, returning the byte count.
pub fn download_to<W: std::io::Write>(
    agent: &ureq::Agent,
    url: &str,
    sink: &mut W,
) -> Result<u64, CoreError> {
    let response = agent
        .get(url)
        .call()
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))?;

    let mut reader = response.into_reader();
    let mut buffer = [0_u8; 32 * 1024];
    let mut total = 0_u64;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| CoreError::Network(format!("{url}: {err}")))?;
        if read == 0 {
            break;
        }
        sink.write_all(&buffer[..read])?;
        total += read as u64;
    }

    Ok(total)
}

/// Fetches a URL into memory.
pub fn get_bytes(url: &str) -> Result<Vec<u8>, CoreError> {
    let agent = agent()?;
    let response = agent
        .get(url)
        .call()
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))?;
    Ok(bytes)
}

/// Response of a JSON call that keeps the status code around.
#[derive(Debug, Clone)]
pub struct JsonResponse {
    /// HTTP status.
    pub status: u16,
    /// Parsed body, when it was JSON.
    pub body: serde_json::Value,
}

impl JsonResponse {
    /// Whether the request succeeded.
    pub fn is_ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Posts a form body and returns JSON.
pub fn post_form(url: &str, form: &[(&str, &str)]) -> Result<JsonResponse, CoreError> {
    let agent = agent()?;
    let response = agent.post(url).send_form(form);

    read_json(url, response)
}

/// Posts a JSON body and returns JSON.
pub fn post_json(url: &str, body: &serde_json::Value) -> Result<JsonResponse, CoreError> {
    let agent = agent()?;
    let response = agent
        .post(url)
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .send_json(body.clone());

    read_json(url, response)
}

/// Gets JSON, optionally with a bearer token.
pub fn get_json(url: &str, bearer: Option<&str>) -> Result<JsonResponse, CoreError> {
    let agent = agent()?;
    let mut request = agent.get(url).set("Accept", "application/json");
    if let Some(token) = bearer {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }

    read_json(url, request.call())
}

fn read_json(
    url: &str,
    response: Result<ureq::Response, ureq::Error>,
) -> Result<JsonResponse, CoreError> {
    let (status, text) = match response {
        Ok(response) => {
            let status = response.status();
            let text = response.into_string().unwrap_or_default();
            (status, text)
        }
        Err(ureq::Error::Status(status, response)) => {
            let text = response.into_string().unwrap_or_default();
            (status, text)
        }
        Err(err) => return Err(CoreError::Network(format!("{url}: {err}"))),
    };

    let body = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);

    Ok(JsonResponse { status, body })
}
