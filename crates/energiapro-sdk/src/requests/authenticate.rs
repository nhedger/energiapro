use serde_json::Value;

use crate::errors::EnergiaProError;
use crate::responses::AuthenticateResponse;

use super::Request;

const AUTH_ENDPOINT: &str = "authenticate.php";

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct AuthenticateRequest {
    username: String,
    one_time_secret_key: String,
}

impl AuthenticateRequest {
    pub(crate) fn new(username: impl Into<String>, one_time_secret_key: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            one_time_secret_key: one_time_secret_key.into(),
        }
    }
}

impl Request for AuthenticateRequest {
    type Response = AuthenticateResponse;

    fn validate_request(&self) -> Result<(), EnergiaProError> {
        if self.username.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "username cannot be empty".to_owned(),
            ));
        }

        if self.one_time_secret_key.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "secret_key cannot be empty".to_owned(),
            ));
        }

        Ok(())
    }

    fn to_request_builder(
        &self,
        http_client: &reqwest::Client,
        base_url: &str,
        _token: &str,
    ) -> reqwest::RequestBuilder {
        let url = format!("{base_url}/{AUTH_ENDPOINT}");
        let form = vec![
            ("username", self.username.clone()),
            ("secret_key", self.one_time_secret_key.clone()),
        ];

        http_client.post(url).form(&form)
    }

    fn parse_response(&self, payload: Value) -> Result<Self::Response, EnergiaProError> {
        Ok(AuthenticateResponse::new(payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_required_authentication_arguments() {
        let err = AuthenticateRequest::new("", "secret").validate_request();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));

        let err = AuthenticateRequest::new("username", "").validate_request();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));
    }
}
