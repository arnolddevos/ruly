use super::Quantity;
use chrono::NaiveDate;

/// A naive date quantity with opinionated formatting and parsing.
pub struct Date;

static FORMATS: &[&str] = &["%d/%m/%Y", "%d.%m.%Y", "%Y-%m-%d"];

impl Quantity for Date {
    type Repr = NaiveDate;

    fn parse(text: &str) -> Result<Self::Repr, crate::variant::Error> {
        for fmt in FORMATS {
            if let Ok(d) = NaiveDate::parse_from_str(text, fmt) {
                return Ok(d);
            }
        }
        Err("invalid date format".into())
    }

    fn format(value: &Self::Repr) -> String {
        value.format("%d/%m/%Y").to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::quantity::Value;

    #[test]
    fn date_value_parsing() {
        type C = Value<Date>;
        assert_eq!(
            C::from_repr(NaiveDate::from_ymd_opt(2001, 5, 23).unwrap()),
            "23/5/2001".parse::<C>().unwrap()
        );
        assert_eq!(
            C::from_repr(NaiveDate::from_ymd_opt(2001, 5, 23).unwrap()),
            "23.05.2001".parse::<C>().unwrap()
        );
        assert_eq!(
            C::from_repr(NaiveDate::from_ymd_opt(2001, 5, 23).unwrap()),
            "2001-05-23".parse::<C>().unwrap()
        );
    }
}
