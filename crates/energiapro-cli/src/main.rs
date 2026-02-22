use clap::Parser;

mod commands;
mod helpers;

type DynError = Box<dyn std::error::Error>;

#[derive(Parser, Debug)]
#[command(name = "energiapro", version, about = "CLI for EnergiaPro API")]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
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
    cli.command.run().await
}
