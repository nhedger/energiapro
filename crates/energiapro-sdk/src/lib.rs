//! Async Rust SDK for the EnergiaPro API.
//!
//! This crate exposes [`EnergiaPro`] as a high-level client for retrieving
//! installations and measurements from EnergiaPro.
//!
//! Authentication is handled automatically: the SDK exchanges your username and
//! secret key for an API token, caches it, and refreshes it as needed.
//!
//! # Examples
//!
//! ```no_run
//! use energiapro::{EnergiaPro, MeasurementScope};
//!
//! # async fn demo() -> Result<(), energiapro::EnergiaProError> {
//! let sdk = EnergiaPro::new("username", "secret_key")?;
//! let measurements = sdk
//!     .measurements
//!     .all("507167", "5806.000", MeasurementScope::LpnJson)
//!     .await?;
//!
//! println!("retrieved {}", measurements.len());
//! # Ok(())
//! # }
//! ```

mod client;
mod energiapro;
mod errors;
mod models;
mod requests;
mod resources;
mod responses;
mod types;

pub use client::ClientOptions;
pub use energiapro::EnergiaPro;
pub use errors::{ApiErrorCode, EnergiaProError};
pub use models::{Installation, Measurement};
pub use types::{DateInput, MeasurementScope};
