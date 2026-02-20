use serde_json::Value;

use super::helpers::send_json_request;
use super::options::ClientOptions;
use super::tokens::TokenManager;
use crate::errors::EnergiaProError;
use crate::models::Measurement;
use crate::requests::{AuthenticatedApiRequest, MeasurementsRequest};

#[cfg(test)]
use super::helpers::{endpoint_url, strip_utf8_bom};

#[derive(Debug)]
pub struct EnergiaProClient {
    /// The underlying HTTP client used for making requests to the EnergiaPro API.
    http_client: reqwest::Client,

    /// The base URL for the EnergiaPro API
    base_url: String,

    /// The token manager responsible for obtaining and refreshing authentication tokens.
    token_manager: TokenManager,
}

impl EnergiaProClient {
    /// Create a new EnergiaPro client
    pub fn new(
        username: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Result<Self, EnergiaProError> {
        Self::with_options(username, secret_key, ClientOptions::default())
    }

    /// Create a new EnergiaPro client with custom options
    pub fn with_options(
        username: impl Into<String>,
        secret_key: impl Into<String>,
        options: ClientOptions,
    ) -> Result<Self, EnergiaProError> {
        let username = username.into();
        if username.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "username cannot be empty".to_owned(),
            ));
        }

        let secret_key = secret_key.into();
        if secret_key.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "secret_key cannot be empty".to_owned(),
            ));
        }

        let http_client = reqwest::Client::builder()
            .timeout(options.timeout)
            .build()?;
        let base_url = Self::normalize_base_url(options.base_url)?;
        let token_manager =
            TokenManager::new(http_client.clone(), username, secret_key, base_url.clone());

        Ok(Self {
            http_client,
            base_url,
            token_manager,
        })
    }

    /// Retrieve measurements for a given installation and time range.
    pub async fn measurements(
        &self,
        request: MeasurementsRequest,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let payload = self.authenticated_json_request(&request).await?;

        let payload = ensure_installation_id_on_rows(payload, request.installation_id());
        Ok(serde_json::from_value(payload)?)
    }

    async fn token(&self) -> Result<String, EnergiaProError> {
        self.token_manager.obtain().await
    }

    async fn authenticated_json_request(
        &self,
        request: &impl AuthenticatedApiRequest,
    ) -> Result<Value, EnergiaProError> {
        request.validate_request()?;
        let mut has_retried_with_fresh_token = false;

        loop {
            let token = self.token().await?;
            let payload = send_json_request(request.to_request_builder(
                &self.http_client,
                &self.base_url,
                &token,
            ))
            .await?;

            if let Some(error) = EnergiaProError::from_api_payload(&payload) {
                if !has_retried_with_fresh_token {
                    if let EnergiaProError::Api { code, .. } = &error {
                        if code.is_token_error() {
                            self.token_manager.clear().await;
                            has_retried_with_fresh_token = true;
                            continue;
                        }
                    }
                }

                return Err(error);
            }

            return Ok(payload);
        }
    }

    /// Normalize the base URL.
    fn normalize_base_url(base_url: String) -> Result<String, EnergiaProError> {
        let normalized = base_url.trim().trim_end_matches('/').to_owned();
        if normalized.is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "base_url cannot be empty".to_owned(),
            ));
        }
        Ok(normalized)
    }
}

fn ensure_installation_id_on_rows(payload: Value, installation_id: &str) -> Value {
    match payload {
        Value::Array(rows) => Value::Array(
            rows.into_iter()
                .map(|row| ensure_installation_id_on_row(row, installation_id))
                .collect(),
        ),
        other => other,
    }
}

fn ensure_installation_id_on_row(row: Value, installation_id: &str) -> Value {
    match row {
        Value::Object(mut object) => {
            if !object.contains_key("installation_id") && !object.contains_key("num_inst") {
                object.insert(
                    "num_inst".to_owned(),
                    Value::String(installation_id.to_owned()),
                );
            }

            Value::Object(object)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base_url_and_builds_endpoints() {
        let normalized =
            EnergiaProClient::normalize_base_url("https://example.com/api/".to_owned()).unwrap();
        assert_eq!(normalized, "https://example.com/api");
        assert_eq!(
            endpoint_url(&normalized, "authenticate.php"),
            "https://example.com/api/authenticate.php"
        );
        assert_eq!(
            endpoint_url(&normalized, "index.php"),
            "https://example.com/api/index.php"
        );
    }

    #[test]
    fn strips_utf8_bom_in_json_payload() {
        let payload = "\u{feff}\u{feff}{\"errorCode\":\"0\"}";
        let value: Value = serde_json::from_str(strip_utf8_bom(payload)).unwrap();
        assert_eq!(value.get("errorCode").unwrap(), "0");
    }

    #[test]
    fn injects_num_inst_when_missing_from_response() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "date": "2024-04-01 15:00:00",
                "quantite_m3": "77.10"
            }
        ]);

        let enriched = ensure_installation_id_on_rows(payload, "5806.000");
        let first = enriched.as_array().unwrap().first().unwrap();

        assert_eq!(first.get("num_inst").unwrap(), "5806.000");
    }

    #[test]
    fn keeps_num_inst_from_response_when_present() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "num_inst": "9999.000",
                "quantite_m3": "77.10"
            }
        ]);

        let enriched = ensure_installation_id_on_rows(payload, "5806.000");
        let first = enriched.as_array().unwrap().first().unwrap();

        assert_eq!(first.get("num_inst").unwrap(), "9999.000");
    }

    #[test]
    fn keeps_installation_id_from_response_when_present() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "installation_id": "9999.000",
                "quantite_m3": "77.10"
            }
        ]);

        let enriched = ensure_installation_id_on_rows(payload, "5806.000");
        let first = enriched.as_array().unwrap().first().unwrap();

        assert_eq!(first.get("installation_id").unwrap(), "9999.000");
        assert!(first.get("num_inst").is_none());
    }
}
