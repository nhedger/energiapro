use serde_json::Value;
use thiserror::Error;

use super::api_error_code::ApiErrorCode;

/// Error type returned by fallible operations in this SDK.
#[derive(Debug, Error)]
pub enum EnergiaProError {
    /// A provided argument failed client-side validation.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// Failed to generate the one-time secret key hash.
    #[error("failed to generate one-time secret_key: {0}")]
    SecretKeyGeneration(#[from] bcrypt::BcryptError),
    /// Failed to join a blocking task.
    #[error("failed to join blocking task: {0}")]
    BlockingTaskJoin(#[from] tokio::task::JoinError),
    /// HTTP transport-level failure.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// Non-success HTTP status not mapped to a typed API error.
    #[error("http status {status} from {endpoint}: {body_snippet}")]
    HttpStatus {
        /// HTTP status code returned by the server.
        status: reqwest::StatusCode,
        /// Endpoint URL that produced the response.
        endpoint: String,
        /// Trimmed excerpt of the response body for diagnostics.
        body_snippet: String,
    },
    /// Failed to parse or serialize JSON payloads.
    #[error("invalid json payload: {0}")]
    Json(#[from] serde_json::Error),
    /// Authentication response did not contain a token.
    #[error("authentication succeeded but token is missing")]
    MissingToken,
    /// Structured API error payload.
    #[error("api error {code}: {message}")]
    Api {
        /// Parsed API error code.
        code: ApiErrorCode,
        /// API-provided message.
        message: String,
    },
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

    #[tokio::test]
    async fn maps_blocking_task_join_error() {
        let join_error = tokio::task::spawn_blocking(|| {
            panic!("simulated panic in blocking task");
        })
        .await
        .unwrap_err();

        let err: EnergiaProError = join_error.into();
        assert!(matches!(err, EnergiaProError::BlockingTaskJoin(_)));
    }
}
