use super::Quantity;
use chrono::{Datelike, NaiveDate};

/// A naive date quantity with opinionated formatting and parsing.
pub struct Date;

static FORMATS: &[&str] = &["%d/%m/%Y", "%F", "%v", "%d.%m.%Y"];

impl Quantity for Date {
    type Repr = NaiveDate;

    fn parse(text: &str) -> Result<Self::Repr, crate::variant::Error> {
        for fmt in FORMATS {
            if let Ok(d) = NaiveDate::parse_from_str(text, fmt) {
                let d = if d.year() < 100 {
                    NaiveDate::from_ymd_opt(d.year() + 2000, d.month(), d.day()).unwrap()
                } else {
                    d
                };
                return Ok(d);
            }
        }
        Err("unrecognised date format".into())
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
            C::from_repr(NaiveDate::from_ymd_opt(2021, 5, 23).unwrap()),
            "23/05/21".parse::<C>().unwrap()
        );
        assert_eq!(
            C::from_repr(NaiveDate::from_ymd_opt(2001, 5, 23).unwrap()),
            "2001-05-23".parse::<C>().unwrap()
        );
    }
}
