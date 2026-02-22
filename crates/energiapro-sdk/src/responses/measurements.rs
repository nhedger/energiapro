use serde::Deserialize;
use serde::de;
use serde_json::Value;

use crate::errors::EnergiaProError;
use crate::models::Measurement;
use crate::responses::Response;

#[derive(Debug, Clone)]
pub(crate) struct MeasurementsResponse {
    payload: Value,
    installation_id: String,
}

impl MeasurementsResponse {
    pub(crate) fn new(payload: Value, installation_id: impl Into<String>) -> Self {
        Self {
            payload,
            installation_id: installation_id.into(),
        }
    }
}

impl Response for MeasurementsResponse {
    type Model = Vec<Measurement>;

    fn transform(mut self) -> Result<Self, EnergiaProError> {
        fn ensure_installation_id_on_rows(payload: Value, installation_id: &str) -> Value {
            match payload {
                Value::Array(rows) => Value::Array(
                    rows.into_iter()
                        .map(|row| ensure_installation_id_on_row(row, installation_id))
                        .collect(),
                ),
                other => other,
            }
        }

        fn ensure_installation_id_on_row(row: Value, installation_id: &str) -> Value {
            match row {
                Value::Object(mut object) => {
                    if !object.contains_key("installation_id") && !object.contains_key("num_inst") {
                        object.insert(
                            "num_inst".to_owned(),
                            Value::String(installation_id.to_owned()),
                        );
                    }

                    Value::Object(object)
                }
                other => other,
            }
        }

        self.payload = ensure_installation_id_on_rows(self.payload, &self.installation_id);
        Ok(self)
    }

    fn map(self) -> Result<Self::Model, EnergiaProError> {
        let measurements: Vec<ApiMeasurement> = serde_json::from_value(self.payload)?;
        Ok(measurements.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Deserialize)]
struct ApiMeasurement {
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    client_id: u64,
    #[serde(alias = "num_inst", alias = "installation_id")]
    installation_id: String,
    #[serde(alias = "date")]
    timestamp: String,
    #[serde(
        alias = "index_m3",
        deserialize_with = "deserialize_f64_from_string_or_number"
    )]
    index_m3: f64,
    #[serde(
        alias = "quantite_m3",
        deserialize_with = "deserialize_f64_from_string_or_number"
    )]
    consumption_m3: f64,
    #[serde(
        alias = "consommation_kw_h",
        deserialize_with = "deserialize_f64_from_string_or_number"
    )]
    consumption_kwh: f64,
}

impl From<ApiMeasurement> for Measurement {
    fn from(value: ApiMeasurement) -> Self {
        Self {
            client_id: value.client_id,
            installation_id: value.installation_id,
            timestamp: value.timestamp,
            index_m3: value.index_m3,
            consumption_m3: value.consumption_m3,
            consumption_kwh: value.consumption_kwh,
        }
    }
}

fn deserialize_u64_from_string_or_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64OrString {
        U64(u64),
        String(String),
    }

    match U64OrString::deserialize(deserializer)? {
        U64OrString::U64(value) => Ok(value),
        U64OrString::String(value) => value
            .parse::<u64>()
            .map_err(|_| de::Error::custom("expected unsigned integer as number or string")),
    }
}

fn deserialize_f64_from_string_or_number<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum F64OrString {
        F64(f64),
        String(String),
    }

    match F64OrString::deserialize(deserializer)? {
        F64OrString::F64(value) => Ok(value),
        F64OrString::String(value) => value
            .parse::<f64>()
            .map_err(|_| de::Error::custom("expected decimal number as number or string")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_payload_and_injects_installation_id_when_missing() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "date": "2024-04-01 15:00:00",
                "quantite_m3": "77.10",
                "index_m3": "145506.00",
                "consommation_kw_h": "798.45"
            }
        ]);

        let response = MeasurementsResponse::new(payload, "5806.000");
        let measurements = response.into_model().unwrap();

        assert_eq!(measurements[0].client_id, 507167);
        assert_eq!(measurements[0].installation_id, "5806.000");
        assert_eq!(measurements[0].consumption_m3, 77.10);
    }

    #[test]
    fn injects_num_inst_when_missing_from_response() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "date": "2024-04-01 15:00:00",
                "quantite_m3": 77.10
            }
        ]);

        let transformed = MeasurementsResponse::new(payload, "5806.000")
            .transform()
            .unwrap();
        let first = transformed.payload.as_array().unwrap().first().unwrap();

        assert_eq!(first.get("num_inst").unwrap(), "5806.000");
    }

    #[test]
    fn keeps_num_inst_from_response_when_present() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "num_inst": "9999.000",
                "quantite_m3": 77.10
            }
        ]);

        let transformed = MeasurementsResponse::new(payload, "5806.000")
            .transform()
            .unwrap();
        let first = transformed.payload.as_array().unwrap().first().unwrap();

        assert_eq!(first.get("num_inst").unwrap(), "9999.000");
    }

    #[test]
    fn keeps_installation_id_from_response_when_present() {
        let payload = serde_json::json!([
            {
                "client_id": 507167,
                "installation_id": "9999.000",
                "quantite_m3": 77.10
            }
        ]);

        let transformed = MeasurementsResponse::new(payload, "5806.000")
            .transform()
            .unwrap();
        let first = transformed.payload.as_array().unwrap().first().unwrap();

        assert_eq!(first.get("installation_id").unwrap(), "9999.000");
        assert!(first.get("num_inst").is_none());
    }

    #[test]
    fn maps_mixed_string_and_numeric_field_formats() {
        let payload = serde_json::json!([
            {
                "client_id": "507167",
                "num_inst": "5806.000",
                "date": "2024-04-01 15:00:00",
                "quantite_m3": 77.10,
                "index_m3": "145506.00",
                "consommation_kw_h": 798.45
            }
        ]);

        let measurements = MeasurementsResponse::new(payload, "5806.000")
            .into_model()
            .unwrap();

        assert_eq!(measurements[0].client_id, 507167);
        assert_eq!(measurements[0].index_m3, 145506.00);
        assert_eq!(measurements[0].consumption_m3, 77.10);
        assert_eq!(measurements[0].consumption_kwh, 798.45);
    }
}
