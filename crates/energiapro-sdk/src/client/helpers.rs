use reqwest::header::AUTHORIZATION;
use serde_json::Value;

use crate::errors::EnergiaProError;

/// Send an HTTP request and parse JSON response.
pub(super) async fn send_json_request(
    request: reqwest::RequestBuilder,
) -> Result<Value, EnergiaProError> {
    let response = request.send().await?;
    let payload = response.text().await?;
    let payload = strip_utf8_bom(&payload);
    Ok(serde_json::from_str(payload)?)
}

/// Send a POST request to the EnergiaPro API.
pub(super) async fn post_form_json(
    http_client: &reqwest::Client,
    url: &str,
    form: Vec<(&str, String)>,
    bearer_token: Option<String>,
) -> Result<Value, EnergiaProError> {
    let mut request = http_client.post(url).form(&form);

    if let Some(token) = bearer_token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    send_json_request(request).await
}

/// Strip the UTF-8 Byte Order Mark (BOM) from the beginning of a string.
pub(super) fn strip_utf8_bom(payload: &str) -> &str {
    payload.trim_start_matches('\u{feff}')
}

/// Construct the full URL for an API endpoint.
pub(super) fn endpoint_url(base_url: &str, endpoint: &str) -> String {
    format!("{base_url}/{endpoint}")
}
