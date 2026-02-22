use reqwest::header::AUTHORIZATION;
use serde_json::Value;

use crate::errors::EnergiaProError;
use crate::responses::InstallationsResponse;

use super::Request;

const INSTALLATIONS_ENDPOINT: &str = "index.php";
const INSTALLATIONS_SCOPE: &str = "installation-lpn-list";
const INSTALLATIONS_NUM_INST_PLACEHOLDER: &str = "0";

/// A request for fetching available installations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationsRequest {
    /// Client ID associated with the installations
    client_id: String,
}

impl InstallationsRequest {
    /// Create request for fetching installations.
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), EnergiaProError> {
        if self.client_id.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "client_id cannot be empty".to_owned(),
            ));
        }

        Ok(())
    }

    pub(crate) fn form_data(&self) -> Vec<(&'static str, String)> {
        vec![
            ("scope", INSTALLATIONS_SCOPE.to_owned()),
            ("client_id", self.client_id.clone()),
            ("num_inst", INSTALLATIONS_NUM_INST_PLACEHOLDER.to_owned()),
        ]
    }
}

impl Request for InstallationsRequest {
    type Response = InstallationsResponse;

    fn validate_request(&self) -> Result<(), EnergiaProError> {
        self.validate()
    }

    fn to_request_builder(
        &self,
        http_client: &reqwest::Client,
        base_url: &str,
        token: &str,
    ) -> reqwest::RequestBuilder {
        let form = self.form_data();
        let url = format!("{base_url}/{INSTALLATIONS_ENDPOINT}");

        http_client
            .post(url)
            .form(&form)
            .header(AUTHORIZATION, format!("Bearer {token}"))
    }

    fn parse_response(&self, payload: Value) -> Result<Self::Response, EnergiaProError> {
        Ok(InstallationsResponse::new(payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_required_installations_arguments() {
        let err = InstallationsRequest::new("").validate();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));
    }

    #[test]
    fn builds_installations_form_data() {
        let request = InstallationsRequest::new("507167");

        assert_eq!(
            request.form_data(),
            vec![
                ("scope", "installation-lpn-list".to_owned()),
                ("client_id", "507167".to_owned()),
                ("num_inst", "0".to_owned()),
            ]
        );
    }
}
