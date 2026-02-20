use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Measurement {
    pub client_id: u64,
    #[serde(alias = "num_inst")]
    pub installation_id: String,
    #[serde(alias = "date")]
    pub timestamp: String,
    #[serde(alias = "index_m3")]
    pub index_m3: String,
    #[serde(alias = "quantite_m3")]
    pub consumption_m3: String,
    #[serde(alias = "consommation_kw_h")]
    pub consumption_kwh: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_french_measurements_sample() {
        let payload = r#"
        [
            {
                "client_id": 507167,
                "num_inst": "5806.000",
                "date": "2024-04-01 15:00:00",
                "quantite_m3": "77.10",
                "index_m3": "145506.00",
                "consommation_kw_h": "798.45"
            }
        ]
        "#;

        let data: Vec<Measurement> = serde_json::from_str(payload).unwrap();
        assert_eq!(data[0].installation_id, "5806.000");
        assert_eq!(data[0].timestamp, "2024-04-01 15:00:00");
        assert_eq!(data[0].index_m3, "145506.00");
        assert_eq!(data[0].consumption_m3, "77.10");
        assert_eq!(data[0].consumption_kwh, "798.45");
    }
}
