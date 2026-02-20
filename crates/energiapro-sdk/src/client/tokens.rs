use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::Mutex;

use super::helpers::{endpoint_url, post_form_json};
use crate::errors::EnergiaProError;

const AUTH_ENDPOINT: &str = "authenticate.php";
const BCRYPT_COST: u32 = 11;

/// EnergiaPro API token
#[derive(Debug, Clone)]
struct Token {
    /// The token value obtained from the API
    value: String,

    /// The time at which the token expires
    expires_at: Instant,
}

impl Token {
    /// Create a new token
    fn new(value: String) -> Self {
        Self {
            value,
            expires_at: Instant::now() + Duration::from_secs(55 * 60),
        }
    }

    /// Check if the token is still valid
    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }

    /// Return the token value if it is valid, otherwise return None
    fn valid_value(&self) -> Option<String> {
        self.is_valid().then(|| self.value.clone())
    }
}

/// EnergiaPro API token manager
#[derive(Debug)]
pub(super) struct TokenManager {
    http_client: reqwest::Client,
    username: String,
    secret_key: String,
    base_url: String,
    cached_token: Mutex<Option<Token>>,
    refresh_lock: Mutex<()>,
}

impl TokenManager {
    /// Create a new TokenManager
    pub(super) fn new(
        http_client: reqwest::Client,
        username: String,
        secret_key: String,
        base_url: String,
    ) -> Self {
        Self {
            http_client,
            username,
            secret_key,
            base_url,
            cached_token: Mutex::new(None),
            refresh_lock: Mutex::new(()),
        }
    }

    /// Obtain an EnergiaPro API token
    pub(super) async fn obtain(&self) -> Result<String, EnergiaProError> {
        if let Some(token) = self.obtain_from_cache().await {
            Ok(token)
        } else {
            self.obtain_from_api().await
        }
    }

    /// Clear the cached token
    pub(super) async fn clear(&self) {
        let mut cached_token = self.cached_token.lock().await;
        *cached_token = None;
    }

    /// Obtain a valid token from the cache
    async fn obtain_from_cache(&self) -> Option<String> {
        let cached_token = self.cached_token.lock().await;
        cached_token.as_ref().and_then(Token::valid_value)
    }

    /// Obtain a new token from the API
    async fn obtain_from_api(&self) -> Result<String, EnergiaProError> {
        let _refresh_guard = self.refresh_lock.lock().await;

        if let Some(token) = self.obtain_from_cache().await {
            return Ok(token);
        }

        let new_token = self.authenticate().await?;

        let mut cached_token = self.cached_token.lock().await;
        *cached_token = Some(Token::new(new_token.clone()));
        Ok(new_token)
    }

    /// Exchange the credentials for a new token
    async fn authenticate(&self) -> Result<String, EnergiaProError> {
        let one_time_secret_key = bcrypt::hash(&self.secret_key, BCRYPT_COST)?;

        let form = vec![
            ("username", self.username.to_owned()),
            ("secret_key", one_time_secret_key),
        ];

        let auth_url = endpoint_url(&self.base_url, AUTH_ENDPOINT);

        let payload = post_form_json(&self.http_client, &auth_url, form, None).await?;

        if let Some(error) = EnergiaProError::from_api_payload(&payload) {
            return Err(error);
        }

        payload
            .get("token")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(EnergiaProError::MissingToken)
    }
}
