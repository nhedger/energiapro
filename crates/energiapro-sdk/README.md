# EnergiaPro SDK

[![Crates.io](https://img.shields.io/crates/v/energiapro.svg)](https://crates.io/crates/energiapro)
[![Documentation](https://docs.rs/energiapro/badge.svg)](https://docs.rs/energiapro)

The `energiapro` crate provides a Rust interface to the EnergiaPro API, 
allowing developers to interact with the EnergiaPro platform programmatically. 
This SDK simplifies the process of making API calls, handling authentication, 
and managing responses.

## Installation

Run the following command to add `energiapro` to your Rust project:

```bash
cargo add energiapro
```

## Usage

Here's a simple example of how to use the `energiapro` crate to fetch 
measurements for a specific installation.

```rust
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
    ).await?;

// Print out some details about the retrieved measurements
println!(
    "retrieved {} measurements for January 15, 2024",
    measurements.len()
);
```

See the [`examples`](./examples) directory for more usage examples and patterns.

## License

This project is licensed under both the MIT License and the Apache License
(Version 2.0). See the [LICENSE-MIT] and [LICENSE-APACHE] files for details.

[LICENSE-MIT]: ../../LICENSE-MIT
[LICENSE-APACHE]: ../../LICENSE-APACHE