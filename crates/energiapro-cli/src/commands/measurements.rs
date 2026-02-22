use std::time::Duration;

use clap::{Args, ValueEnum};
use energiapro::{ClientOptions, EnergiaPro, Measurement};
use polars::prelude::*;

use crate::DynError;
use crate::helpers::output::{OutputFormat, export_dataframe, write_stdout};
use crate::helpers::table::render_table;

#[derive(Args, Debug)]
pub(crate) struct MeasurementsArgs {
    #[arg(
        long,
        short = 'u',
        env = "ENERGIAPRO_USERNAME",
        help = "EnergiaPro API username"
    )]
    username: Option<String>,
    #[arg(
        long,
        short = 'k',
        env = "ENERGIAPRO_SECRET_KEY",
        help = "EnergiaPro API secret key"
    )]
    secret_key: Option<String>,
    #[arg(
        long,
        short = 's',
        value_enum,
        default_value_t = Scope::LpnJson,
        help = "Scope to query"
    )]
    scope: Scope,
    #[arg(value_name = "CLIENT_ID", help = "EnergiaPro client identifier")]
    client_id: String,
    #[arg(
        value_name = "INSTALLATION_ID",
        help = "Installation identifier (num_inst)"
    )]
    installation_id: String,
    #[arg(long, help = "Start date filter in YYYY-MM-DD")]
    from: Option<String>,
    #[arg(long, help = "End date filter in YYYY-MM-DD")]
    to: Option<String>,
    #[arg(
        long,
        short = 'f',
        value_enum,
        default_value_t = OutputFormat::Text,
        help = "Output format"
    )]
    format: OutputFormat,
    #[arg(long, default_value_t = 30, help = "HTTP timeout in seconds")]
    timeout_secs: u64,
    #[arg(
        long,
        env = "ENERGIAPRO_BASE_URL",
        help = "HTTPS base API URL (or ENERGIAPRO_BASE_URL)"
    )]
    base_url: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Scope {
    #[value(name = "lpn-json")]
    LpnJson,
    #[value(name = "gc-plus-json")]
    GcPlusJson,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Self::LpnJson => "lpn-json",
            Self::GcPlusJson => "gc-plus-json",
        }
    }
}

pub(super) async fn run(args: MeasurementsArgs) -> Result<(), DynError> {
    let scope = args.scope.as_str();
    let username = args.username.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing username: pass --username or set ENERGIAPRO_USERNAME",
        )
    })?;
    let secret_key = args.secret_key.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing secret key: pass --secret-key or set ENERGIAPRO_SECRET_KEY",
        )
    })?;

    let mut options = ClientOptions::default().with_timeout(Duration::from_secs(args.timeout_secs));
    if let Some(base_url) = args.base_url {
        options = options.with_base_url(base_url);
    }

    let client = EnergiaPro::with_options(username, secret_key, options)?;

    let measurements: Vec<Measurement> = client
        .measurements
        .get(
            args.client_id,
            args.installation_id,
            scope,
            args.from,
            args.to,
        )
        .await?;
    let bytes = match args.format {
        OutputFormat::Text => render_measurements_text(&measurements).into_bytes(),
        format => {
            let mut dataframe = measurements_to_dataframe(&measurements)?;
            export_dataframe(format, &mut dataframe)?
        }
    };

    write_stdout(&bytes)?;
    Ok(())
}

fn render_measurements_text(measurements: &[Measurement]) -> String {
    let rows = measurements
        .iter()
        .map(|measurement| {
            vec![
                measurement.client_id.to_string(),
                measurement.installation_id.clone(),
                measurement.timestamp.clone(),
                measurement.index_m3.to_string(),
                measurement.consumption_m3.to_string(),
                measurement.consumption_kwh.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    render_table(
        &[
            "client_id",
            "installation_id",
            "timestamp",
            "index_m3",
            "consumption_m3",
            "consumption_kwh",
        ],
        &rows,
    )
}

fn measurements_to_dataframe(measurements: &[Measurement]) -> Result<DataFrame, DynError> {
    let client_ids = measurements.iter().map(|m| m.client_id).collect::<Vec<_>>();
    let installation_ids = measurements
        .iter()
        .map(|m| m.installation_id.clone())
        .collect::<Vec<_>>();
    let timestamps = measurements
        .iter()
        .map(|m| m.timestamp.clone())
        .collect::<Vec<_>>();

    let index_m3 = measurements.iter().map(|m| m.index_m3).collect::<Vec<_>>();
    let consumption_m3 = measurements
        .iter()
        .map(|m| m.consumption_m3)
        .collect::<Vec<_>>();
    let consumption_kwh = measurements
        .iter()
        .map(|m| m.consumption_kwh)
        .collect::<Vec<_>>();

    Ok(DataFrame::new_infer_height(vec![
        Series::new("client_id".into(), client_ids).into(),
        Series::new("installation_id".into(), installation_ids).into(),
        Series::new("timestamp".into(), timestamps).into(),
        Series::new("index_m3".into(), index_m3).into(),
        Series::new("consumption_m3".into(), consumption_m3).into(),
        Series::new("consumption_kwh".into(), consumption_kwh).into(),
    ])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn sample_measurements() -> Vec<Measurement> {
        vec![
            Measurement {
                client_id: 1,
                installation_id: "5806.000".to_owned(),
                timestamp: "2024-04-01 15:00:00".to_owned(),
                index_m3: 145506.00,
                consumption_m3: 77.10,
                consumption_kwh: 798.45,
            },
            Measurement {
                client_id: 2,
                installation_id: "5807.000".to_owned(),
                timestamp: "2024-04-02 15:00:00".to_owned(),
                index_m3: 145595.00,
                consumption_m3: 89.30,
                consumption_kwh: 924.80,
            },
        ]
    }

    #[test]
    fn renders_text_table_from_measurements() {
        let measurements = sample_measurements();
        let text = render_measurements_text(&measurements);

        assert!(text.contains("client_id"));
        assert!(text.contains("installation_id"));
        assert!(text.contains("5806.000"));
        assert!(text.contains("798.45"));
    }

    #[test]
    fn exports_csv_from_dataframe() {
        let measurements = sample_measurements();
        let mut dataframe = measurements_to_dataframe(&measurements).unwrap();
        let csv = export_dataframe(OutputFormat::Csv, &mut dataframe).unwrap();
        let csv = String::from_utf8(csv).unwrap();

        assert!(csv.starts_with(
            "client_id,installation_id,timestamp,index_m3,consumption_m3,consumption_kwh\n"
        ));
        assert!(csv.contains("1,5806.000,2024-04-01 15:00:00,145506.0,77.1,798.45"));
        assert!(csv.contains("2,5807.000,2024-04-02 15:00:00,145595.0,89.3,924.8"));
    }

    #[test]
    fn exports_json_with_numeric_measurement_fields() {
        let measurements = sample_measurements();
        let mut dataframe = measurements_to_dataframe(&measurements).unwrap();
        let json_bytes = export_dataframe(OutputFormat::Json, &mut dataframe).unwrap();
        let json = String::from_utf8(json_bytes).unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let row = value.as_array().unwrap().first().unwrap();

        assert!(row.get("index_m3").unwrap().is_number());
        assert!(row.get("consumption_m3").unwrap().is_number());
        assert!(row.get("consumption_kwh").unwrap().is_number());
        assert!(row.get("client_id").unwrap().is_number());
        assert!(row.get("installation_id").unwrap().is_string());
        assert!(row.get("timestamp").unwrap().is_string());
    }

    #[test]
    fn exports_parquet_from_dataframe() {
        let measurements = sample_measurements();
        let mut dataframe = measurements_to_dataframe(&measurements).unwrap();
        let parquet = export_dataframe(OutputFormat::Parquet, &mut dataframe).unwrap();
        assert!(!parquet.is_empty());
    }

    #[test]
    fn exports_jsonl_from_dataframe() {
        let measurements = sample_measurements();
        let mut dataframe = measurements_to_dataframe(&measurements).unwrap();
        let jsonl = export_dataframe(OutputFormat::Jsonl, &mut dataframe).unwrap();
        let jsonl = String::from_utf8(jsonl).unwrap();

        assert!(jsonl.contains("\n"));
        assert!(jsonl.lines().all(|line| line.trim_start().starts_with('{')));
        assert!(!jsonl.trim_start().starts_with('['));
    }
}
