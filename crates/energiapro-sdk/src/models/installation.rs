use serde::{Deserialize, Serialize};

/// EnergiaPro Installation
///
/// This struct represents an EnergiaPro installation, which includes details
/// about the location and address of the installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installation {
    #[serde(alias = "insID")]
    pub id: String,
    #[serde(alias = "adrNomRueC")]
    pub street_name: String,
    #[serde(alias = "adrRueC")]
    pub street_address: String,
    #[serde(alias = "adrNumImm")]
    pub building_number: i64,
    #[serde(alias = "adrCPC")]
    pub postal_code: String,
    #[serde(alias = "adrLocaliteC")]
    pub city: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_french_installation_sample() {
        let payload = r#"
        [
            {
                "insID": "5806.000",
                "adrNomRueC": "Crets",
                "adrRueC": "Rue des Crets 3",
                "adrNumImm": 3,
                "adrCPC": "1037",
                "adrLocaliteC": "Etagnieres"
            }
        ]
        "#;

        let data: Vec<Installation> = serde_json::from_str(payload).unwrap();
        assert_eq!(data[0].id, "5806.000");
        assert_eq!(data[0].postal_code, "1037");
    }
}
