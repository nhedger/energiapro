use serde_json::Value;
use thiserror::Error;

use super::api_error_code::ApiErrorCode;

#[derive(Debug, Error)]
pub enum EnergiaProError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("failed to generate one-time secret_key: {0}")]
    SecretKeyGeneration(#[from] bcrypt::BcryptError),
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid json payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("authentication succeeded but token is missing")]
    MissingToken,
    #[error("api error {code}: {message}")]
    Api { code: ApiErrorCode, message: String },
}

impl EnergiaProError {
    pub(crate) fn from_api_payload(payload: &Value) -> Option<Self> {
        let object = payload.as_object()?;
        let error_code = object.get("errorCode")?;
        let error_code = match error_code {
            Value::String(code) => code.to_owned(),
            Value::Number(code) => code.to_string(),
            _ => return None,
        };

        if error_code == "0" {
            return None;
        }

        let message = object
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("Not allowed.")
            .to_owned();

        Some(Self::Api {
            code: ApiErrorCode::from_api_code(&error_code),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_api_error_code_from_payload() {
        let payload: Value = serde_json::json!({
            "error": "Not allowed.",
            "errorCode": "220"
        });

        let err = EnergiaProError::from_api_payload(&payload);
        assert!(matches!(
            err,
            Some(EnergiaProError::Api {
                code: ApiErrorCode::TokenInvalid,
                ..
            })
        ));
    }

    #[test]
    fn ignores_success_payload() {
        let payload: Value = serde_json::json!({ "errorCode": "0" });
        assert!(EnergiaProError::from_api_payload(&payload).is_none());
    }
}
