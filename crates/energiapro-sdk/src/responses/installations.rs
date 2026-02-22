use serde_json::Value;

use crate::errors::EnergiaProError;
use crate::models::Installation;

use super::Response;

#[derive(Debug, Clone)]
pub(crate) struct InstallationsResponse {
    payload: Value,
}

impl InstallationsResponse {
    pub(crate) fn new(payload: Value) -> Self {
        Self { payload }
    }
}

impl Response for InstallationsResponse {
    type Model = Vec<Installation>;

    fn map(self) -> Result<Self::Model, EnergiaProError> {
        Ok(serde_json::from_value(self.payload)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_installations_payload() {
        let payload = serde_json::json!([
            {
                "insID": "5806.000",
                "adrNomRueC": "Crets",
                "adrRueC": "Rue des Crets 3",
                "adrNumImm": 3,
                "adrCPC": "1037",
                "adrLocaliteC": "Etagnieres"
            }
        ]);

        let response = InstallationsResponse::new(payload);
        let installations = response.into_model().unwrap();

        assert_eq!(installations[0].id, "5806.000");
        assert_eq!(installations[0].postal_code, "1037");
    }
}
