use clap::Subcommand;

use crate::DynError;

mod installations;
mod measurements;

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    #[command(
        about = "Fetch and print installations for one client",
        long_about = "Fetch installations for one client and write formatted output to stdout. The API response must be a top-level JSON array of objects so it can be loaded into a Polars DataFrame."
    )]
    Installations(installations::InstallationsArgs),
    #[command(
        about = "Fetch and print measurements for one installation",
        long_about = "Fetch measurements for one installation and write formatted output to stdout. The API response must be a top-level JSON array of objects so it can be loaded into a Polars DataFrame."
    )]
    Measurements(measurements::MeasurementsArgs),
}

impl Commands {
    pub(crate) async fn run(self) -> Result<(), DynError> {
        match self {
            Self::Installations(args) => installations::run(args).await,
            Self::Measurements(args) => measurements::run(args).await,
        }
    }
}
