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
                "insID": "INSTALLATION_ID_1",
                "adrNomRueC": "STREET_NAME_1",
                "adrRueC": "STREET_ADDRESS_1",
                "adrNumImm": 3,
                "adrCPC": "POSTAL_CODE_1",
                "adrLocaliteC": "CITY_1"
            }
        ]);

        let response = InstallationsResponse::new(payload);
        let installations = response.into_model().unwrap();

        assert_eq!(installations[0].id, "INSTALLATION_ID_1");
        assert_eq!(installations[0].postal_code, "POSTAL_CODE_1");
    }
}
