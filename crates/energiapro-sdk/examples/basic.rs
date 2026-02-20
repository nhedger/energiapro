use energiapro_sdk::{EnergiaProClient, MeasurementScope, MeasurementsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new EnergiaPro client
    let energiapro = EnergiaProClient::new("<USERNAME>", "<SECRET_KEY>")?;

    // Prepare a request for fetching measurements
    let request = MeasurementsRequest::new("<CLIENT_ID>", "<INSTALLATION_ID>")
        .scope(MeasurementScope::LpnJson)
        .from("2024-01-01")
        .to("2024-01-31");

    // Fetch measurements from the API
    let measurements = energiapro.measurements(request).await?;

    // Print out some details about the retrieved measurements
    println!("retrieved {} measurements", measurements.len());
    if let Some(first) = measurements.first() {
        println!("first timestamp: {}", first.timestamp);
        println!("first consumption_kwh: {}", first.consumption_kwh);
    }

    Ok(())
}
