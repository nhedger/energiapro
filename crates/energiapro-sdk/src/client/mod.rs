mod options;
mod token_manager;

use self::token_manager::TokenManager;
use crate::errors::EnergiaProError;
use crate::requests::Request;
use serde_json::Value;

pub use options::ClientOptions;

const ERROR_BODY_SNIPPET_LIMIT: usize = 512;

pub(crate) struct Client {
    /// The underlying HTTP client used for making requests to the EnergiaPro API.
    http_client: reqwest::Client,

    /// The base URL for the EnergiaPro API
    base_url: String,

    /// The token lifecycle owner responsible for obtaining and refreshing authentication tokens.
    token: TokenManager,
}

impl Client {
    /// Create a new HTTP client
    pub fn new(
        username: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Result<Self, EnergiaProError> {
        Self::with_options(username, secret_key, ClientOptions::default())
    }

    /// Create a new HTTP client with custom options
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
        let token = TokenManager::new(username, secret_key);

        Ok(Self {
            http_client,
            base_url,
            token,
        })
    }

    /// Send an authenticated request to the EnergiaPro API and parse the response.
    ///
    /// This method handles token management, including refreshing the token if
    /// it detects an authentication error in the API response. It will retry
    /// the request once with a fresh token if necessary.
    pub(crate) async fn send<R>(&self, request: &R) -> Result<R::Response, EnergiaProError>
    where
        R: Request,
    {
        let mut has_retried_with_fresh_token = false;

        loop {
            let token = self.token.obtain(self).await?;
            match self.execute_request(request, &token).await {
                Err(EnergiaProError::Api { code, .. })
                    if code.is_token_error() && !has_retried_with_fresh_token =>
                {
                    self.token.clear().await;
                    has_retried_with_fresh_token = true;
                    continue;
                }
                result => return result,
            }
        }
    }

    pub(super) async fn execute_request<R>(
        &self,
        request: &R,
        token: &str,
    ) -> Result<R::Response, EnergiaProError>
    where
        R: Request,
    {
        request.validate_request()?;

        let response = request
            .to_request_builder(&self.http_client, &self.base_url, token)
            .send()
            .await?;

        let status = response.status();
        let endpoint = response.url().to_string();
        let payload = response.text().await?;
        let payload = payload.trim_start_matches('\u{feff}');

        if !status.is_success() {
            return Err(Self::map_non_success_response(status, endpoint, payload));
        }

        let payload: Value = serde_json::from_str(payload)?;

        if let Some(error) = EnergiaProError::from_api_payload(&payload) {
            return Err(error);
        }

        request.parse_response(payload)
    }

    /// Normalize the base URL.
    fn normalize_base_url(base_url: String) -> Result<String, EnergiaProError> {
        let normalized = base_url.trim().trim_end_matches('/').to_owned();
        if normalized.is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "base_url cannot be empty".to_owned(),
            ));
        }

        let parsed = reqwest::Url::parse(&normalized).map_err(|_| {
            EnergiaProError::InvalidArgument("base_url must be a valid absolute URL".to_owned())
        })?;
        if parsed.scheme() != "https" {
            return Err(EnergiaProError::InvalidArgument(
                "base_url must use https".to_owned(),
            ));
        }

        Ok(normalized)
    }

    fn map_non_success_response(
        status: reqwest::StatusCode,
        endpoint: String,
        payload: &str,
    ) -> EnergiaProError {
        if let Ok(payload_json) = serde_json::from_str::<Value>(payload)
            && let Some(error) = EnergiaProError::from_api_payload(&payload_json)
        {
            return error;
        }

        EnergiaProError::HttpStatus {
            status,
            endpoint,
            body_snippet: Self::error_body_snippet(payload),
        }
    }

    fn error_body_snippet(payload: &str) -> String {
        let trimmed = payload.trim();
        if trimmed.is_empty() {
            return "<empty response body>".to_owned();
        }

        let mut chars = trimmed.chars();
        let snippet: String = chars.by_ref().take(ERROR_BODY_SNIPPET_LIMIT).collect();
        if chars.next().is_some() {
            format!("{snippet}...")
        } else {
            snippet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ApiErrorCode;

    #[test]
    fn normalizes_base_url_and_builds_endpoints() {
        let normalized = Client::normalize_base_url("https://example.com/api/".to_owned()).unwrap();
        assert_eq!(normalized, "https://example.com/api");
        assert_eq!(
            format!("{normalized}/authenticate.php"),
            "https://example.com/api/authenticate.php"
        );
        assert_eq!(
            format!("{normalized}/index.php"),
            "https://example.com/api/index.php"
        );
    }

    #[test]
    fn rejects_non_https_base_url() {
        let err = Client::normalize_base_url("http://example.com/api".to_owned());
        assert!(matches!(
            err,
            Err(EnergiaProError::InvalidArgument(message)) if message == "base_url must use https"
        ));
    }

    #[test]
    fn rejects_non_absolute_base_url() {
        let err = Client::normalize_base_url("example.com/api".to_owned());
        assert!(matches!(
            err,
            Err(EnergiaProError::InvalidArgument(message)) if message == "base_url must be a valid absolute URL"
        ));
    }

    #[test]
    fn rejects_empty_base_url() {
        let err = Client::normalize_base_url("   ".to_owned());
        assert!(matches!(
            err,
            Err(EnergiaProError::InvalidArgument(message)) if message == "base_url cannot be empty"
        ));
    }

    #[test]
    fn maps_non_success_json_api_error_payload() {
        let err = Client::map_non_success_response(
            reqwest::StatusCode::UNAUTHORIZED,
            "https://example.com/api/index.php".to_owned(),
            r#"{"error":"Not allowed.","errorCode":"220"}"#,
        );

        assert!(matches!(
            err,
            EnergiaProError::Api {
                code: ApiErrorCode::TokenInvalid,
                message
            } if message == "Not allowed."
        ));
    }

    #[test]
    fn maps_non_success_non_json_payload_to_http_status_error() {
        let endpoint = "https://example.com/api/index.php";
        let err = Client::map_non_success_response(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            endpoint.to_owned(),
            "<html>oops</html>",
        );

        assert!(matches!(
            err,
            EnergiaProError::HttpStatus {
                status,
                endpoint: actual_endpoint,
                body_snippet
            } if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR
                && actual_endpoint == endpoint
                && body_snippet == "<html>oops</html>"
        ));
    }

    #[test]
    fn truncates_non_success_body_snippet() {
        let err = Client::map_non_success_response(
            reqwest::StatusCode::BAD_GATEWAY,
            "https://example.com/api/index.php".to_owned(),
            &"x".repeat(ERROR_BODY_SNIPPET_LIMIT + 10),
        );

        assert!(matches!(
            err,
            EnergiaProError::HttpStatus { body_snippet, .. }
                if body_snippet.len() == ERROR_BODY_SNIPPET_LIMIT + 3 && body_snippet.ends_with("...")
        ));
    }
}
