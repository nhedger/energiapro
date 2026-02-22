use std::fmt;

/// API-level error code returned by EnergiaPro (`errorCode`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiErrorCode {
    /// Error code `1`: request method is not POST.
    MethodNotPost,
    /// Error code `2`: one-time secret key was already used.
    SecretKeyAlreadyUsed,
    /// Error code `3`: requested scope does not exist.
    ScopeNotFound,
    /// Error code `4`: maximum number of sessions has been reached.
    MaxSessionsReached,
    /// Error code `5`: request is missing required parameters.
    MissingParameters,
    /// Error code `6`: request must use SSL.
    MissingSsl,
    /// Error code `10`: username is invalid.
    InvalidUsername,
    /// Error code `11`: password/secret key is missing.
    MissingPassword,
    /// Error code `12`: portal account is disabled.
    PortalAccountDisabled,
    /// Error code `15`: API account is disabled.
    ApiAccountDisabled,
    /// Error code `100`: no LPN data is available.
    NoLpnData,
    /// Error code `110`: no installations are available.
    NoInstallations,
    /// Error code `210`: token is corrupted.
    TokenCorrupted,
    /// Error code `220`: token is invalid or expired.
    TokenInvalid,
    /// Unknown code preserved as returned by the API.
    Unknown(String),
}

impl ApiErrorCode {
    /// Map a raw API error code string into a typed [`ApiErrorCode`].
    ///
    /// Unrecognized values are preserved in [`ApiErrorCode::Unknown`].
    pub fn from_api_code(code: &str) -> Self {
        match code {
            "1" => Self::MethodNotPost,
            "2" => Self::SecretKeyAlreadyUsed,
            "3" => Self::ScopeNotFound,
            "4" => Self::MaxSessionsReached,
            "5" => Self::MissingParameters,
            "6" => Self::MissingSsl,
            "10" => Self::InvalidUsername,
            "11" => Self::MissingPassword,
            "12" => Self::PortalAccountDisabled,
            "15" => Self::ApiAccountDisabled,
            "100" => Self::NoLpnData,
            "110" => Self::NoInstallations,
            "210" => Self::TokenCorrupted,
            "220" => Self::TokenInvalid,
            other => Self::Unknown(other.to_owned()),
        }
    }

    /// Return `true` if this code indicates token corruption or invalidity.
    pub fn is_token_error(&self) -> bool {
        matches!(self, Self::TokenCorrupted | Self::TokenInvalid)
    }

    /// Return the raw API code string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::MethodNotPost => "1",
            Self::SecretKeyAlreadyUsed => "2",
            Self::ScopeNotFound => "3",
            Self::MaxSessionsReached => "4",
            Self::MissingParameters => "5",
            Self::MissingSsl => "6",
            Self::InvalidUsername => "10",
            Self::MissingPassword => "11",
            Self::PortalAccountDisabled => "12",
            Self::ApiAccountDisabled => "15",
            Self::NoLpnData => "100",
            Self::NoInstallations => "110",
            Self::TokenCorrupted => "210",
            Self::TokenInvalid => "220",
            Self::Unknown(code) => code.as_str(),
        }
    }
}

impl fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_api_error_codes() {
        assert_eq!(
            ApiErrorCode::from_api_code("220"),
            ApiErrorCode::TokenInvalid
        );
        assert_eq!(
            ApiErrorCode::from_api_code("110"),
            ApiErrorCode::NoInstallations
        );
    }

    #[test]
    fn keeps_unknown_codes() {
        assert_eq!(
            ApiErrorCode::from_api_code("999"),
            ApiErrorCode::Unknown("999".to_owned())
        );
    }
}
