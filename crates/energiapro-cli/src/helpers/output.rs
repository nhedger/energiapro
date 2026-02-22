use std::io::Write;

use clap::ValueEnum;
use polars::prelude::*;

use crate::DynError;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
    Jsonl,
    Csv,
    Parquet,
}

pub(crate) fn export_dataframe(
    format: OutputFormat,
    dataframe: &mut DataFrame,
) -> Result<Vec<u8>, DynError> {
    let mut bytes = Vec::new();

    match format {
        OutputFormat::Text => unreachable!("text output does not use dataframe export"),
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

pub(crate) fn write_stdout(bytes: &[u8]) -> Result<(), DynError> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(bytes)?;
    Ok(())
}
