use std::sync::Arc;

use crate::client::{Client, ClientOptions};
use crate::errors::EnergiaProError;
use crate::resources::{InstallationsResource, MeasurementsResource};

/// High-level asynchronous client for the EnergiaPro API.
///
/// Use [`EnergiaPro::new`] to create a client with default options, or
/// [`EnergiaPro::with_options`] to customize networking behavior. API
/// operations are available through the [`EnergiaPro::installations`] and
/// [`EnergiaPro::measurements`] resource properties.
pub struct EnergiaPro {
    /// Installation-related API operations.
    pub installations: InstallationsResource,
    /// Measurement-related API operations.
    pub measurements: MeasurementsResource,
}

impl EnergiaPro {
    /// Create a new EnergiaPro SDK client with default [`ClientOptions`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `username` is empty or contains only whitespace.
    /// - `secret_key` is empty or contains only whitespace.
    /// - the underlying HTTP client cannot be initialized.
    pub fn new(
        username: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Result<Self, EnergiaProError> {
        let client = Arc::new(Client::new(username, secret_key)?);

        Ok(Self {
            installations: InstallationsResource::new(Arc::clone(&client)),
            measurements: MeasurementsResource::new(client),
        })
    }

    /// Create a new EnergiaPro SDK client with custom [`ClientOptions`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `username` is empty or contains only whitespace.
    /// - `secret_key` is empty or contains only whitespace.
    /// - `options.base_url` is empty, invalid, or does not use `https`.
    /// - the underlying HTTP client cannot be initialized.
    pub fn with_options(
        username: impl Into<String>,
        secret_key: impl Into<String>,
        options: ClientOptions,
    ) -> Result<Self, EnergiaProError> {
        let client = Arc::new(Client::with_options(username, secret_key, options)?);

        Ok(Self {
            installations: InstallationsResource::new(Arc::clone(&client)),
            measurements: MeasurementsResource::new(client),
        })
    }
}
