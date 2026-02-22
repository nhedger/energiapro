use std::sync::Arc;

use crate::client::Client;
use crate::errors::EnergiaProError;
use crate::models::Measurement;
use crate::requests::MeasurementsRequest;
use crate::responses::Response;
use crate::types::{DateInput, MeasurementScope};

/// Resource for measurement-related API operations.
#[derive(Clone)]
pub struct MeasurementsResource {
    client: Arc<Client>,
}

impl MeasurementsResource {
    pub(crate) fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// Retrieve measurements for a given installation and optional date range.
    pub async fn get(
        &self,
        client_id: impl AsRef<str>,
        installation_id: impl AsRef<str>,
        scope: impl Into<MeasurementScope>,
        from: Option<impl DateInput>,
        to: Option<impl DateInput>,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let request =
            MeasurementsRequest::new(client_id.as_ref(), installation_id.as_ref()).scope(scope);

        let request = if let Some(from) = from {
            request.from(from)
        } else {
            request
        };

        let request = if let Some(to) = to {
            request.to(to)
        } else {
            request
        };

        self.client.send(&request).await?.into_model()
    }

    /// Retrieve all measurements for a given installation.
    ///
    /// # Notes
    ///
    /// This method retrieves all measurements for the specified installation,
    /// which may result in a large amount of data being returned. Consider
    /// using the other methods in this resource to filter measurements by date
    /// or date range if you do not need the entire dataset.
    pub async fn all(
        &self,
        client_id: impl AsRef<str>,
        installation_id: impl AsRef<str>,
        scope: impl Into<MeasurementScope>,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let request =
            MeasurementsRequest::new(client_id.as_ref(), installation_id.as_ref()).scope(scope);

        self.client.send(&request).await?.into_model()
    }

    /// Retrieve measurements for a given installation and date.
    pub async fn for_date(
        &self,
        client_id: impl AsRef<str>,
        installation_id: impl AsRef<str>,
        scope: impl Into<MeasurementScope>,
        date: impl DateInput,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let date = date.into_date_string();

        let request = MeasurementsRequest::new(client_id.as_ref(), installation_id.as_ref())
            .scope(scope)
            .from(date.clone())
            .to(date);

        self.client.send(&request).await?.into_model()
    }

    /// Retrieve measurements for an installation and optional date range.
    pub async fn for_date_range(
        &self,
        client_id: impl AsRef<str>,
        installation_id: impl AsRef<str>,
        scope: impl Into<MeasurementScope>,
        from: impl DateInput,
        to: impl DateInput,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let request = MeasurementsRequest::new(client_id.as_ref(), installation_id.as_ref())
            .scope(scope)
            .from(from)
            .to(to);

        self.client.send(&request).await?.into_model()
    }

    /// Retrieve measurements for an installation since a given date.
    pub async fn since(
        &self,
        client_id: impl AsRef<str>,
        installation_id: impl AsRef<str>,
        scope: impl Into<MeasurementScope>,
        date: impl DateInput,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let request = MeasurementsRequest::new(client_id.as_ref(), installation_id.as_ref())
            .scope(scope)
            .from(date);

        self.client.send(&request).await?.into_model()
    }

    /// Retrieve measurements for an installation up to a given date.
    pub async fn up_to(
        &self,
        client_id: impl AsRef<str>,
        installation_id: impl AsRef<str>,
        scope: impl Into<MeasurementScope>,
        date: impl DateInput,
    ) -> Result<Vec<Measurement>, EnergiaProError> {
        let request = MeasurementsRequest::new(client_id.as_ref(), installation_id.as_ref())
            .scope(scope)
            .to(date);

        self.client.send(&request).await?.into_model()
    }
}
