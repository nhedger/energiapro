mod authenticate;
mod installations;
mod measurements;

use crate::errors::EnergiaProError;

/// A trait representing a response from the EnergiaPro API.
///
/// This trait defines the necessary methods for transforming and mapping the
/// raw response payload into the appropriate model types used by the SDK. Each
/// specific response type (e.g., `MeasurementsResponse`) will implement this
/// trait to provide the necessary logic for handling the API response and
/// converting it into the desired model format.
pub(crate) trait Response {
    /// The type of the model that this response will be mapped to after processing
    type Model;

    /// Transform the raw response payload if necessary before mapping it to the
    /// final model. This method can be used to perform any necessary adjustments or
    /// transformations on the response data before it is converted into the model type.
    fn transform(self) -> Result<Self, EnergiaProError>
    where
        Self: Sized,
    {
        Ok(self)
    }

    /// Map the transformed response into the final model type. This method should
    /// handle the logic for converting the response data into the appropriate model
    /// format.
    fn map(self) -> Result<Self::Model, EnergiaProError>;

    /// A convenience method that combines the transformation and mapping steps into
    /// a single operation. This method first transforms the response and then maps it
    /// to the final model type, returning the result.
    fn into_model(self) -> Result<Self::Model, EnergiaProError>
    where
        Self: Sized,
    {
        self.transform()?.map()
    }
}

pub(crate) use authenticate::AuthenticateResponse;
pub(crate) use installations::InstallationsResponse;
pub(crate) use measurements::MeasurementsResponse;
