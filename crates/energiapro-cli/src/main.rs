use std::io::{Error, ErrorKind, Write};
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use energiapro_sdk::{ClientOptions, EnergiaProClient, Measurement, MeasurementsRequest};
use polars::prelude::*;

type DynError = Box<dyn std::error::Error>;

#[derive(Parser, Debug)]
#[command(name = "energiapro", version, about = "CLI for EnergiaPro API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Fetch and print measurements for one installation",
        long_about = "Fetch measurements for one installation and write formatted output to stdout. The API response must be a top-level JSON array of objects so it can be loaded into a Polars DataFrame."
    )]
    Measurements(MeasurementsArgs),
}

#[derive(Args, Debug)]
struct MeasurementsArgs {
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
        default_value_t = OutputFormat::Json,
        help = "Output format"
    )]
    format: OutputFormat,
    #[arg(long, default_value_t = 30, help = "HTTP timeout in seconds")]
    timeout_secs: u64,
    #[arg(
        long,
        env = "ENERGIAPRO_BASE_URL",
        help = "Base API URL (or ENERGIAPRO_BASE_URL)"
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

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Jsonl,
    Csv,
    Parquet,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Self::LpnJson => "lpn-json",
            Self::GcPlusJson => "gc-plus-json",
        }
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), DynError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Measurements(args) => run_measurements(args).await?,
    }

    Ok(())
}

async fn run_measurements(args: MeasurementsArgs) -> Result<(), DynError> {
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

    let client = EnergiaProClient::with_options(username, secret_key, options)?;

    let mut request = MeasurementsRequest::new(args.client_id, args.installation_id).scope(scope);
    if let Some(from) = args.from {
        request = request.from(from);
    }
    if let Some(to) = args.to {
        request = request.to(to);
    }

    let measurements: Vec<Measurement> = client.measurements(request).await?;
    let mut dataframe = measurements_to_dataframe(&measurements)?;
    let bytes = export_dataframe(args.format, &mut dataframe)?;
    write_stdout(&bytes)?;
    Ok(())
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

    let index_m3 = measurements
        .iter()
        .map(|m| parse_decimal_field("index_m3", &m.index_m3))
        .collect::<Result<Vec<_>, _>>()?;
    let consumption_m3 = measurements
        .iter()
        .map(|m| parse_decimal_field("consumption_m3", &m.consumption_m3))
        .collect::<Result<Vec<_>, _>>()?;
    let consumption_kwh = measurements
        .iter()
        .map(|m| parse_decimal_field("consumption_kwh", &m.consumption_kwh))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DataFrame::new(vec![
        Series::new("client_id".into(), client_ids).into(),
        Series::new("installation_id".into(), installation_ids).into(),
        Series::new("timestamp".into(), timestamps).into(),
        Series::new("index_m3".into(), index_m3).into(),
        Series::new("consumption_m3".into(), consumption_m3).into(),
        Series::new("consumption_kwh".into(), consumption_kwh).into(),
    ])?)
}

fn parse_decimal_field(field: &str, value: &str) -> Result<f64, Error> {
    value.parse::<f64>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("invalid {field} value '{value}', expected decimal number"),
        )
    })
}

fn export_dataframe(format: OutputFormat, dataframe: &mut DataFrame) -> Result<Vec<u8>, DynError> {
    let mut bytes = Vec::new();

    match format {
        OutputFormat::Json => {
            JsonWriter::new(&mut bytes)
                .with_json_format(JsonFormat::Json)
                .finish(dataframe)?;
        }
        OutputFormat::Jsonl => {
            JsonWriter::new(&mut bytes)
                .with_json_format(JsonFormat::JsonLines)
                .finish(dataframe)?;
        }
        OutputFormat::Csv => {
            CsvWriter::new(&mut bytes)
                .include_header(true)
                .finish(dataframe)?;
        }
        OutputFormat::Parquet => {
            ParquetWriter::new(&mut bytes).finish(dataframe)?;
        }
    }

    Ok(bytes)
}

fn write_stdout(bytes: &[u8]) -> Result<(), DynError> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(bytes)?;
    Ok(())
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
                index_m3: "145506.00".to_owned(),
                consumption_m3: "77.10".to_owned(),
                consumption_kwh: "798.45".to_owned(),
            },
            Measurement {
                client_id: 2,
                installation_id: "5807.000".to_owned(),
                timestamp: "2024-04-02 15:00:00".to_owned(),
                index_m3: "145595.00".to_owned(),
                consumption_m3: "89.30".to_owned(),
                consumption_kwh: "924.80".to_owned(),
            },
        ]
    }

    #[test]
    fn rejects_invalid_decimal_values() {
        let mut measurements = sample_measurements();
        measurements[0].index_m3 = "abc".to_owned();

        let err = measurements_to_dataframe(&measurements).unwrap_err();
        assert!(err.to_string().contains("invalid index_m3 value"));
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
