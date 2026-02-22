use serde::{Deserialize, Serialize};

/// A single measurement row returned by the EnergiaPro API.
///
/// Numeric fields are normalized by the SDK during deserialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    /// Numeric client identifier.
    pub client_id: u64,
    /// Installation identifier (`num_inst` in API payloads).
    #[serde(alias = "num_inst")]
    pub installation_id: String,
    /// Measurement timestamp as returned by the API.
    #[serde(alias = "date")]
    pub timestamp: String,
    /// Meter index in cubic meters.
    #[serde(alias = "index_m3")]
    pub index_m3: f64,
    /// Consumed volume in cubic meters for the interval.
    #[serde(alias = "quantite_m3")]
    pub consumption_m3: f64,
    /// Consumed energy in kilowatt-hours for the interval.
    #[serde(alias = "consommation_kw_h")]
    pub consumption_kwh: f64,
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
                "quantite_m3": 77.10,
                "index_m3": 145506.00,
                "consommation_kw_h": 798.45
            }
        ]
        "#;

        let data: Vec<Measurement> = serde_json::from_str(payload).unwrap();
        assert_eq!(data[0].client_id, 507167);
        assert_eq!(data[0].installation_id, "5806.000");
        assert_eq!(data[0].timestamp, "2024-04-01 15:00:00");
        assert_eq!(data[0].index_m3, 145506.0);
        assert_eq!(data[0].consumption_m3, 77.1);
        assert_eq!(data[0].consumption_kwh, 798.45);
    }
}
