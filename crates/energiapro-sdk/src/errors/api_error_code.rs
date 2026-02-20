use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiErrorCode {
    MethodNotPost,
    SecretKeyAlreadyUsed,
    ScopeNotFound,
    MaxSessionsReached,
    MissingParameters,
    MissingSsl,
    InvalidUsername,
    MissingPassword,
    PortalAccountDisabled,
    ApiAccountDisabled,
    NoLpnData,
    NoInstallations,
    TokenCorrupted,
    TokenInvalid,
    Unknown(String),
}

impl ApiErrorCode {
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

    pub fn is_token_error(&self) -> bool {
        matches!(self, Self::TokenCorrupted | Self::TokenInvalid)
    }

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
