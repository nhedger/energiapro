use crate::errors::EnergiaProError;

pub(crate) trait AuthenticatedApiRequest {
    fn validate_request(&self) -> Result<(), EnergiaProError>;

    fn to_request_builder(
        &self,
        http_client: &reqwest::Client,
        base_url: &str,
        token: &str,
    ) -> reqwest::RequestBuilder;
}
