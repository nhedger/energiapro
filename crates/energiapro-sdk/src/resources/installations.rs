use std::sync::Arc;

use crate::client::Client;
use crate::errors::EnergiaProError;
use crate::models::Installation;
use crate::requests::InstallationsRequest;
use crate::responses::Response;

/// Resource for installation-related API operations.
#[derive(Clone)]
pub struct InstallationsResource {
    client: Arc<Client>,
}

impl InstallationsResource {
    pub(crate) fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// Retrieve installations for a given client identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `client_id` is empty or contains only whitespace.
    /// - authentication fails or a token cannot be obtained/refreshed.
    /// - the HTTP request fails.
    /// - the API returns a non-success status or error payload.
    /// - the response payload cannot be parsed into installations.
    pub async fn list(
        &self,
        client_id: impl AsRef<str>,
    ) -> Result<Vec<Installation>, EnergiaProError> {
        let request = InstallationsRequest::new(client_id.as_ref());
        self.client.send(&request).await?.into_model()
    }
}
