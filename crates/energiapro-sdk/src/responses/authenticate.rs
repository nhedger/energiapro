use serde_json::Value;

use crate::errors::EnergiaProError;

use super::Response;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthenticateResponse {
    payload: Value,
}

impl AuthenticateResponse {
    pub(crate) fn new(payload: Value) -> Self {
        Self { payload }
    }
}

impl Response for AuthenticateResponse {
    type Model = String;

    fn map(self) -> Result<Self::Model, EnergiaProError> {
        self.payload
            .get("token")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(EnergiaProError::MissingToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_payload_token() {
        let response = AuthenticateResponse::new(serde_json::json!({ "token": "abc123" }));
        let token = response.into_model().unwrap();

        assert_eq!(token, "abc123");
    }

    #[test]
    fn errors_when_token_is_missing() {
        let response = AuthenticateResponse::new(serde_json::json!({ "errorCode": "0" }));
        let err = response.into_model();

        assert!(matches!(err, Err(EnergiaProError::MissingToken)));
    }
}
