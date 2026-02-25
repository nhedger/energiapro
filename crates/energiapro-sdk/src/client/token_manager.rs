use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::Client;
use crate::errors::EnergiaProError;
use crate::requests::AuthenticateRequest;
use crate::responses::Response;

const BCRYPT_COST: u32 = 11;

/// Manages authentication tokens
///
/// The TokenManager is responsible for obtaining, caching, and refreshing API
/// tokens. It ensures that only one token refresh operation occurs at a time
/// and that cached tokens are used when valid to minimize unnecessary API
/// calls.
///
/// Tokens typically expire after 60 minutes, but the TokenManager refreshes
/// them after 55 minutes to avoid edge cases where a token might expire during
/// an API request.
///
/// To obtain a token, call the `obtain` method with a reference to the API
/// client. This method will return a valid token, either from the cache or by
/// authenticating with the API if necessary.
///
/// To clear the cached token (e.g., if you know it has been revoked), call the
/// `clear` method.
pub(super) struct TokenManager {
    /// The EnergiaPro username
    username: String,
    /// The EnergiaPro secret key
    secret_key: String,
    /// Cached token and its expiration time
    cached_token: Mutex<Option<(String, Instant)>>,
    /// Mutex to ensure that only one task is refreshing the token at a time
    refresh_lock: Mutex<()>,
}

impl TokenManager {
    pub(super) fn new(username: String, secret_key: String) -> Self {
        Self {
            username,
            secret_key,
            cached_token: Mutex::new(None),
            refresh_lock: Mutex::new(()),
        }
    }

    /// Obtain an EnergiaPro API token
    ///
    /// This method will first attempt to obtain a valid token from the cache.
    /// If no valid token is found, it will exchange the credentials for a new
    /// token using the API.
    ///
    /// Tokens expire after 60 minutes, but we refresh them after 55 minutes to
    /// avoid edge cases where a token expires during a request.
    pub(super) async fn obtain(&self, client: &Client) -> Result<String, EnergiaProError> {
        if let Some(token) = self.obtain_from_cache().await {
            Ok(token)
        } else {
            self.obtain_from_api(client).await
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

        if let Some((value, expires_at)) = cached_token.as_ref()
            && Instant::now() < *expires_at
        {
            return Some(value.clone());
        }

        None
    }

    /// Obtain a new token from the API
    ///
    /// This method exchanges the credentials for a fresh API token and updates
    /// the cache with the new token and its expiration time.
    async fn obtain_from_api(&self, client: &Client) -> Result<String, EnergiaProError> {
        // Wait for the refresh lock to ensure that only one task is refreshing
        // the token at a time.
        let _refresh_guard = self.refresh_lock.lock().await;

        // Check the cache again in case another task refreshed the token while we
        // were waiting for the lock.
        if let Some(token) = self.obtain_from_cache().await {
            return Ok(token);
        }

        // No valid token in the cache, so we need to authenticate with the API to
        // get a new token.
        let new_token = self.authenticate(client).await?;

        // Wait for the cache lock to update the cached token with the new value and
        // expiration time.
        let mut cached_token = self.cached_token.lock().await;

        // Update the cache with the new token and its expiration time.
        const TOKEN_TTL: Duration = Duration::from_secs(55 * 60);
        *cached_token = Some((new_token.clone(), Instant::now() + TOKEN_TTL));

        Ok(new_token)
    }

    /// Exchange the credentials for a new token
    async fn authenticate(&self, client: &Client) -> Result<String, EnergiaProError> {
        let secret_key = self.secret_key.clone();

        let one_time_secret_key =
            tokio::task::spawn_blocking(move || bcrypt::hash(&secret_key, BCRYPT_COST)).await??;

        let request = AuthenticateRequest::new(self.username.to_owned(), one_time_secret_key);

        client.execute_request(&request, "").await?.into_model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generates_bcrypt_one_time_secret_key() {
        let secret_key = "super-secret".to_owned();

        let hash = tokio::task::spawn_blocking(move || bcrypt::hash(&secret_key, BCRYPT_COST))
            .await
            .unwrap()
            .unwrap();

        assert_ne!(hash, "super-secret");
        assert!(bcrypt::verify("super-secret", &hash).unwrap());
    }

    #[tokio::test]
    async fn returns_cached_token_when_not_expired() {
        let token = TokenManager::new("username".to_owned(), "super-secret".to_owned());

        {
            let mut cached_token = token.cached_token.lock().await;
            *cached_token = Some((
                "cached-value".to_owned(),
                Instant::now() + Duration::from_secs(60),
            ));
        }

        assert_eq!(
            token.obtain_from_cache().await,
            Some("cached-value".to_owned())
        );
    }

    #[tokio::test]
    async fn ignores_expired_cached_token() {
        let token = TokenManager::new("username".to_owned(), "super-secret".to_owned());

        {
            let mut cached_token = token.cached_token.lock().await;
            *cached_token = Some((
                "expired-value".to_owned(),
                Instant::now() - Duration::from_secs(1),
            ));
        }

        assert_eq!(token.obtain_from_cache().await, None);
    }
}
