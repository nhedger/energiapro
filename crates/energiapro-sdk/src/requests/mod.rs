mod authenticate;
mod installations;
mod measurements;

use crate::errors::EnergiaProError;
use crate::responses::Response as ApiResponse;
use serde_json::Value;

/// A trait representing a request to the EnergiaPro API.
///
/// This trait defines the necessary methods for validating the request,
/// converting it into an HTTP request builder, and parsing the response from
/// the API. Each specific request type (e.g., `MeasurementsRequest`) will
/// implement this trait to provide the necessary logic for interacting with
/// the corresponding API endpoint.
pub(crate) trait Request {
    /// The type of the response that this request expects to receive from the API.
    type Response: ApiResponse;

    /// Validate the request parameters before sending it to the API
    fn validate_request(&self) -> Result<(), EnergiaProError>;

    /// Convert the request into a `reqwest::RequestBuilder` that can be sent
    /// to the API. This method should set up the URL, HTTP method, headers,
    /// and body as needed for the specific request type.
    fn to_request_builder(
        &self,
        http_client: &reqwest::Client,
        base_url: &str,
        token: &str,
    ) -> reqwest::RequestBuilder;

    /// Parse the response payload from the API into the appropriate response type.
    fn parse_response(&self, payload: Value) -> Result<Self::Response, EnergiaProError>;
}

pub(crate) use authenticate::AuthenticateRequest;

// Re-exports
pub use installations::InstallationsRequest;
pub use measurements::MeasurementsRequest;
