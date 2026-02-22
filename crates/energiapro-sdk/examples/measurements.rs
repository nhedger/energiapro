use energiapro::{EnergiaPro, MeasurementScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fetch_all_measurements().await?;
    fetch_measurements_for_date().await?;
    fetch_measurements_with_date_range().await?;
    Ok(())
}

/// Example function demonstrating how to fetch all measurements for a given client and installation.
async fn fetch_all_measurements() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new EnergiaPro client
    let energiapro = EnergiaPro::new("<USERNAME>", "<SECRET_KEY>")?;

    // Fetch measurements from the API
    let measurements = energiapro
        .measurements
        .all("client-id", "installation-id", MeasurementScope::LpnJson)
        .await?;

    // Print out some details about the retrieved measurements
    println!("retrieved {} measurements", measurements.len());

    Ok(())
}

/// Example function demonstrating how to fetch measurements for a specific date.
async fn fetch_measurements_for_date() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new EnergiaPro client
    let energiapro = EnergiaPro::new("<USERNAME>", "<SECRET_KEY>")?;

    // Fetch measurements from the API for a specific date
    let measurements = energiapro
        .measurements
        .for_date(
            "client-id",
            "installation-id",
            MeasurementScope::LpnJson,
            "2024-01-15",
        )
        .await?;

    // Print out some details about the retrieved measurements
    println!(
        "retrieved {} measurements for January 15, 2024",
        measurements.len()
    );

    Ok(())
}

/// Example function demonstrating how to fetch measurements for a specific date range.
async fn fetch_measurements_with_date_range() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new EnergiaPro client
    let energiapro = EnergiaPro::new("<USERNAME>", "<SECRET_KEY>")?;

    // Fetch measurements from the API with a date range
    let measurements = energiapro
        .measurements
        .for_date_range(
            "client-id",
            "installation-id",
            MeasurementScope::LpnJson,
            "2024-01-01",
            "2024-01-31",
        )
        .await?;

    // Print out some details about the retrieved measurements
    println!(
        "retrieved {} measurements for January 2024",
        measurements.len()
    );

    Ok(())
}
