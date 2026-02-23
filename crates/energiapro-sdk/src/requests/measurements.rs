use chrono::NaiveDate;
use reqwest::header::AUTHORIZATION;
use serde_json::Value;

use crate::errors::EnergiaProError;
use crate::responses::MeasurementsResponse;
use crate::types::{DateInput, MeasurementScope};

use super::Request;

const MEASUREMENTS_ENDPOINT: &str = "index.php";

/// A request for fetching measurements
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MeasurementsRequest {
    /// Scope of the measurements to fetch
    scope: MeasurementScope,

    /// Client ID associated with the measurements
    client_id: String,

    /// Installation ID for which to fetch measurements
    installation_id: String,

    /// Optional start date for filtering measurements (format: YYYY-MM-DD)
    from: Option<String>,

    /// Optional end date for filtering measurements (format: YYYY-MM-DD)
    to: Option<String>,
}

impl MeasurementsRequest {
    /// Create request for fetching measurements
    pub(crate) fn new(client_id: impl Into<String>, installation_id: impl Into<String>) -> Self {
        Self {
            scope: MeasurementScope::default(),
            client_id: client_id.into(),
            installation_id: installation_id.into(),
            from: None,
            to: None,
        }
    }

    pub(crate) fn scope(mut self, scope: impl Into<MeasurementScope>) -> Self {
        self.scope = scope.into();
        self
    }

    /// Set the start date filter. Accepts `NaiveDate`, `String`, or `&str`.
    pub(crate) fn from(mut self, from: impl DateInput) -> Self {
        self.from = Some(from.into_date_string());
        self
    }

    /// Set the end date filter. Accepts `NaiveDate`, `String`, or `&str`.
    pub(crate) fn to(mut self, to: impl DateInput) -> Self {
        self.to = Some(to.into_date_string());
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

        let from = self
            .from
            .as_deref()
            .map(|from| validate_date_argument("from", from))
            .transpose()?;

        let to = self
            .to
            .as_deref()
            .map(|to| validate_date_argument("to", to))
            .transpose()?;

        if let (Some(from), Some(to)) = (from, to)
            && from > to
        {
            return Err(EnergiaProError::InvalidArgument(
                "from must be less than or equal to to".to_owned(),
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

fn validate_date_argument(field: &str, value: &str) -> Result<NaiveDate, EnergiaProError> {
    let parsed = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        EnergiaProError::InvalidArgument(format!("{field} must be in YYYY-MM-DD format"))
    })?;

    if parsed.format("%Y-%m-%d").to_string() != value {
        return Err(EnergiaProError::InvalidArgument(format!(
            "{field} must be in YYYY-MM-DD format"
        )));
    }

    Ok(parsed)
}

impl Request for MeasurementsRequest {
    type Response = MeasurementsResponse;

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

    fn parse_response(&self, payload: Value) -> Result<Self::Response, EnergiaProError> {
        Ok(MeasurementsResponse::new(payload, self.installation_id()))
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
    }

    #[test]
    fn builds_measurements_form_data_with_naive_dates() {
        let request = MeasurementsRequest::new("507167", "5806.000")
            .from(NaiveDate::parse_from_str("2024-04-01", "%Y-%m-%d").unwrap())
            .to(NaiveDate::parse_from_str("2024-04-02", "%Y-%m-%d").unwrap());

        assert_eq!(
            request.form_data(),
            vec![
                ("scope", "lpn-json".to_owned()),
                ("client_id", "507167".to_owned()),
                ("num_inst", "5806.000".to_owned()),
                ("date_debut", "2024-04-01".to_owned()),
                ("date_fin", "2024-04-02".to_owned()),
            ]
        );
    }

    #[test]
    fn builds_measurements_form_data_with_string_dates() {
        let request = MeasurementsRequest::new("507167", "5806.000")
            .from("2024-04-01")
            .to("2024-04-30".to_owned());

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
    fn rejects_invalid_date_format_strings() {
        let err = MeasurementsRequest::new("507167", "5806.000")
            .from("2024/04/01")
            .validate();
        assert!(matches!(
            err,
            Err(EnergiaProError::InvalidArgument(message)) if message == "from must be in YYYY-MM-DD format"
        ));

        let err = MeasurementsRequest::new("507167", "5806.000")
            .to("2024-4-1")
            .validate();
        assert!(matches!(
            err,
            Err(EnergiaProError::InvalidArgument(message)) if message == "to must be in YYYY-MM-DD format"
        ));
    }

    #[test]
    fn rejects_inverted_date_range() {
        let err = MeasurementsRequest::new("507167", "5806.000")
            .from("2024-04-30")
            .to("2024-04-01")
            .validate();
        assert!(matches!(
            err,
            Err(EnergiaProError::InvalidArgument(message))
                if message == "from must be less than or equal to to"
        ));
    }

    #[test]
    fn accepts_equal_date_range_bounds() {
        let err = MeasurementsRequest::new("507167", "5806.000")
            .from("2024-04-01")
            .to("2024-04-01")
            .validate();
        assert!(err.is_ok());
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
