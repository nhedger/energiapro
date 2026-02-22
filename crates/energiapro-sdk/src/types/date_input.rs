use chrono::NaiveDate;

/// Input accepted by public SDK methods where a date is required.
///
/// String-based inputs must be provided using the `YYYY-MM-DD` format.
pub trait DateInput: private::Sealed {
    fn into_date_string(self) -> String;
}

impl DateInput for NaiveDate {
    fn into_date_string(self) -> String {
        self.format("%Y-%m-%d").to_string()
    }
}

impl DateInput for String {
    fn into_date_string(self) -> String {
        self
    }
}

impl DateInput for &str {
    fn into_date_string(self) -> String {
        self.to_owned()
    }
}

mod private {
    use chrono::NaiveDate;

    pub trait Sealed {}

    impl Sealed for NaiveDate {}
    impl Sealed for String {}
    impl Sealed for &str {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_naive_date() {
        let date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        assert_eq!(date.into_date_string(), "2024-04-01");
    }

    #[test]
    fn keeps_string_input() {
        assert_eq!("2024-04-01".into_date_string(), "2024-04-01");
        assert_eq!("2024-04-01".to_owned().into_date_string(), "2024-04-01");
    }
}
