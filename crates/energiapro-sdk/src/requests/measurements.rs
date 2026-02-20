use reqwest::header::AUTHORIZATION;

use crate::errors::EnergiaProError;
use crate::types::MeasurementScope;

use super::AuthenticatedApiRequest;

const MEASUREMENTS_ENDPOINT: &str = "index.php";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementsRequest {
    scope: MeasurementScope,
    client_id: String,
    installation_id: String,
    from: Option<String>,
    to: Option<String>,
}

impl MeasurementsRequest {
    /// Create request for fetching measurements
    pub fn new(client_id: impl Into<String>, installation_id: impl Into<String>) -> Self {
        Self {
            scope: MeasurementScope::default(),
            client_id: client_id.into(),
            installation_id: installation_id.into(),
            from: None,
            to: None,
        }
    }

    pub fn scope(mut self, scope: impl Into<MeasurementScope>) -> Self {
        self.scope = scope.into();
        self
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), EnergiaProError> {
        if self.scope.as_str().trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "scope cannot be empty".to_owned(),
            ));
        }

        if self.installation_id.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "installation_id cannot be empty".to_owned(),
            ));
        }

        if self.client_id.trim().is_empty() {
            return Err(EnergiaProError::InvalidArgument(
                "client_id cannot be empty".to_owned(),
            ));
        }

        if matches!(self.from.as_deref(), Some(date) if date.trim().is_empty()) {
            return Err(EnergiaProError::InvalidArgument(
                "from cannot be an empty string".to_owned(),
            ));
        }

        if matches!(self.to.as_deref(), Some(date) if date.trim().is_empty()) {
            return Err(EnergiaProError::InvalidArgument(
                "to cannot be an empty string".to_owned(),
            ));
        }

        Ok(())
    }

    pub(crate) fn form_data(&self) -> Vec<(&'static str, String)> {
        let mut form = vec![
            ("scope", self.scope.as_str().to_owned()),
            ("client_id", self.client_id.clone()),
            ("num_inst", self.installation_id.clone()),
        ];

        if let Some(from) = self.from.as_deref() {
            form.push(("date_debut", from.to_owned()));
        }

        if let Some(to) = self.to.as_deref() {
            form.push(("date_fin", to.to_owned()));
        }

        form
    }

    pub(crate) fn installation_id(&self) -> &str {
        &self.installation_id
    }
}

impl AuthenticatedApiRequest for MeasurementsRequest {
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
        let url = format!("{base_url}/{MEASUREMENTS_ENDPOINT}");

        http_client
            .post(url)
            .form(&form)
            .header(AUTHORIZATION, format!("Bearer {token}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MeasurementScope;

    #[test]
    fn validates_required_measurements_arguments() {
        let err = MeasurementsRequest::new("507167", "5806.000")
            .scope("")
            .validate();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));

        let err = MeasurementsRequest::new("507167", "").validate();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));

        let err = MeasurementsRequest::new("", "5806.000").validate();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));

        let err = MeasurementsRequest::new("507167", "5806.000")
            .from("   ")
            .validate();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));

        let err = MeasurementsRequest::new("507167", "5806.000")
            .to(" ")
            .validate();
        assert!(matches!(err, Err(EnergiaProError::InvalidArgument(_))));
    }

    #[test]
    fn builds_measurements_form_data() {
        let request = MeasurementsRequest::new("507167", "5806.000")
            .from("2024-04-01")
            .to("2024-04-30");

        assert_eq!(
            request.form_data(),
            vec![
                ("scope", "lpn-json".to_owned()),
                ("client_id", "507167".to_owned()),
                ("num_inst", "5806.000".to_owned()),
                ("date_debut", "2024-04-01".to_owned()),
                ("date_fin", "2024-04-30".to_owned()),
            ]
        );
    }

    #[test]
    fn accepts_known_scope_variant() {
        let request =
            MeasurementsRequest::new("507167", "5806.000").scope(MeasurementScope::GcPlusJson);

        assert_eq!(
            request.form_data(),
            vec![
                ("scope", "gc-plus-json".to_owned()),
                ("client_id", "507167".to_owned()),
                ("num_inst", "5806.000".to_owned()),
            ]
        );
    }

    #[test]
    fn accepts_custom_scope_string() {
        let request = MeasurementsRequest::new("507167", "5806.000").scope("custom-scope");

        assert_eq!(
            request.form_data(),
            vec![
                ("scope", "custom-scope".to_owned()),
                ("client_id", "507167".to_owned()),
                ("num_inst", "5806.000".to_owned()),
            ]
        );
    }
}
