use std::time::Duration;

use clap::Args;
use energiapro_sdk::{ClientOptions, EnergiaPro, Installation};
use polars::prelude::*;

use crate::DynError;
use crate::helpers::output::{OutputFormat, export_dataframe, write_stdout};
use crate::helpers::table::render_table;

#[derive(Args, Debug)]
pub(crate) struct InstallationsArgs {
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
    #[arg(value_name = "CLIENT_ID", help = "EnergiaPro client identifier")]
    client_id: String,
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

pub(super) async fn run(args: InstallationsArgs) -> Result<(), DynError> {
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
    let installations: Vec<Installation> = client.installations.list(args.client_id).await?;
    let bytes = match args.format {
        OutputFormat::Text => render_installations_text(&installations).into_bytes(),
        format => {
            let mut dataframe = installations_to_dataframe(&installations)?;
            export_dataframe(format, &mut dataframe)?
        }
    };

    write_stdout(&bytes)?;
    Ok(())
}

fn render_installations_text(installations: &[Installation]) -> String {
    let rows = installations
        .iter()
        .map(|installation| {
            vec![
                installation.id.clone(),
                installation.street_name.clone(),
                installation.street_address.clone(),
                installation.building_number.to_string(),
                installation.postal_code.clone(),
                installation.city.clone(),
            ]
        })
        .collect::<Vec<_>>();

    render_table(
        &[
            "id",
            "street_name",
            "street_address",
            "building_number",
            "postal_code",
            "city",
        ],
        &rows,
    )
}

fn installations_to_dataframe(installations: &[Installation]) -> Result<DataFrame, DynError> {
    let ids = installations
        .iter()
        .map(|i| i.id.clone())
        .collect::<Vec<_>>();
    let street_names = installations
        .iter()
        .map(|i| i.street_name.clone())
        .collect::<Vec<_>>();
    let street_addresses = installations
        .iter()
        .map(|i| i.street_address.clone())
        .collect::<Vec<_>>();
    let building_numbers = installations
        .iter()
        .map(|i| i.building_number)
        .collect::<Vec<_>>();
    let postal_codes = installations
        .iter()
        .map(|i| i.postal_code.clone())
        .collect::<Vec<_>>();
    let cities = installations
        .iter()
        .map(|i| i.city.clone())
        .collect::<Vec<_>>();

    Ok(DataFrame::new_infer_height(vec![
        Series::new("id".into(), ids).into(),
        Series::new("street_name".into(), street_names).into(),
        Series::new("street_address".into(), street_addresses).into(),
        Series::new("building_number".into(), building_numbers).into(),
        Series::new("postal_code".into(), postal_codes).into(),
        Series::new("city".into(), cities).into(),
    ])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn sample_installations() -> Vec<Installation> {
        vec![
            Installation {
                id: "5806.000".to_owned(),
                street_name: "Crets".to_owned(),
                street_address: "Rue des Crets 3".to_owned(),
                building_number: 3,
                postal_code: "1037".to_owned(),
                city: "Etagnieres".to_owned(),
            },
            Installation {
                id: "5807.000".to_owned(),
                street_name: "Moulin".to_owned(),
                street_address: "Rue du Moulin 10".to_owned(),
                building_number: 10,
                postal_code: "1000".to_owned(),
                city: "Lausanne".to_owned(),
            },
        ]
    }

    #[test]
    fn renders_text_table_from_installations() {
        let installations = sample_installations();
        let text = render_installations_text(&installations);

        assert!(text.contains("id"));
        assert!(text.contains("street_name"));
        assert!(text.contains("5806.000"));
        assert!(text.contains("Etagnieres"));
    }

    #[test]
    fn exports_csv_from_dataframe() {
        let installations = sample_installations();
        let mut dataframe = installations_to_dataframe(&installations).unwrap();
        let csv = export_dataframe(OutputFormat::Csv, &mut dataframe).unwrap();
        let csv = String::from_utf8(csv).unwrap();

        assert!(
            csv.starts_with("id,street_name,street_address,building_number,postal_code,city\n")
        );
        assert!(csv.contains("5806.000,Crets,Rue des Crets 3,3,1037,Etagnieres"));
        assert!(csv.contains("5807.000,Moulin,Rue du Moulin 10,10,1000,Lausanne"));
    }

    #[test]
    fn exports_json_with_expected_installation_fields() {
        let installations = sample_installations();
        let mut dataframe = installations_to_dataframe(&installations).unwrap();
        let json_bytes = export_dataframe(OutputFormat::Json, &mut dataframe).unwrap();
        let json = String::from_utf8(json_bytes).unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let row = value.as_array().unwrap().first().unwrap();

        assert!(row.get("id").unwrap().is_string());
        assert!(row.get("street_name").unwrap().is_string());
        assert!(row.get("street_address").unwrap().is_string());
        assert!(row.get("building_number").unwrap().is_number());
        assert!(row.get("postal_code").unwrap().is_string());
        assert!(row.get("city").unwrap().is_string());
    }

    #[test]
    fn exports_parquet_from_dataframe() {
        let installations = sample_installations();
        let mut dataframe = installations_to_dataframe(&installations).unwrap();
        let parquet = export_dataframe(OutputFormat::Parquet, &mut dataframe).unwrap();
        assert!(!parquet.is_empty());
    }

    #[test]
    fn exports_jsonl_from_dataframe() {
        let installations = sample_installations();
        let mut dataframe = installations_to_dataframe(&installations).unwrap();
        let jsonl = export_dataframe(OutputFormat::Jsonl, &mut dataframe).unwrap();
        let jsonl = String::from_utf8(jsonl).unwrap();

        assert!(jsonl.contains("\n"));
        assert!(jsonl.lines().all(|line| line.trim_start().starts_with('{')));
        assert!(!jsonl.trim_start().starts_with('['));
    }
}
