/// Scope value used by the measurements endpoint.
///
/// [`MeasurementScope::LpnJson`] is the default scope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MeasurementScope {
    /// Standard LPN JSON payload (`lpn-json`).
    #[default]
    LpnJson,
    /// Extended GC+ JSON payload (`gc-plus-json`).
    GcPlusJson,
    /// Custom raw scope string for unsupported or future API scopes.
    Custom(String),
}

impl MeasurementScope {
    /// Return the raw API scope value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::LpnJson => "lpn-json",
            Self::GcPlusJson => "gc-plus-json",
            Self::Custom(value) => value.as_str(),
        }
    }
}

impl From<&str> for MeasurementScope {
    fn from(value: &str) -> Self {
        match value {
            "lpn-json" => Self::LpnJson,
            "gc-plus-json" => Self::GcPlusJson,
            other => Self::Custom(other.to_owned()),
        }
    }
}

impl From<String> for MeasurementScope {
    fn from(value: String) -> Self {
        match value.as_str() {
            "lpn-json" => Self::LpnJson,
            "gc-plus-json" => Self::GcPlusJson,
            _ => Self::Custom(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_scopes() {
        assert_eq!(
            MeasurementScope::from("lpn-json"),
            MeasurementScope::LpnJson
        );
        assert_eq!(
            MeasurementScope::from("gc-plus-json"),
            MeasurementScope::GcPlusJson
        );
    }

    #[test]
    fn keeps_custom_scope() {
        assert_eq!(
            MeasurementScope::from("custom-scope"),
            MeasurementScope::Custom("custom-scope".to_owned())
        );
    }

    #[test]
    fn default_scope_is_lpn_json() {
        assert_eq!(MeasurementScope::default(), MeasurementScope::LpnJson);
        assert_eq!(MeasurementScope::default().as_str(), "lpn-json");
    }
}
