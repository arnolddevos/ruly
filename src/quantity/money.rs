use crate::quantity::Quantity;
use crate::variant::Error;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0, one_of},
    combinator::{all_consuming, opt},
    sequence::{preceded, terminated, tuple},
};

/// Australian currency.
pub struct AUD;

impl Quantity for AUD {
    type Repr = i64;

    fn parse(text: &str) -> Result<Self::Repr, Error> {
        money_from_str::<2>(text, "$")
    }

    fn format(value: &Self::Repr) -> String {
        money_to_str::<2>(*value, "$")
    }
}

/// US currency.
pub struct USD;

impl Quantity for USD {
    type Repr = i64;

    fn parse(text: &str) -> Result<Self::Repr, Error> {
        money_from_str::<2>(text, "$")
    }

    fn format(value: &Self::Repr) -> String {
        money_to_str::<2>(*value, "$")
    }
}

fn money_from_str<const PRECISION: u32>(input: &str, symbol: &str) -> Result<i64, Error> {
    let error = "error in money value";
    let spaces = multispace0::<&str, nom::error::Error<&str>>;
    let digits = digit1::<&str, nom::error::Error<&str>>;

    let (_, (sign1, _, sign2, whole, frac)) = all_consuming(terminated(
        tuple((
            opt(terminated(one_of("+-"), spaces)),
            opt(terminated(tag(symbol), spaces)),
            opt(terminated(one_of("+-"), spaces)),
            opt(digits),
            opt(preceded(tag("."), opt(digits))),
        )),
        spaces,
    ))(input)
    .or(Err(error))?;

    let mut value: i64 = 0;

    if let Some(x) = whole {
        value += x.parse::<i64>().or(Err(error))? * 10i64.pow(PRECISION);
    }

    if let Some(Some(x)) = frac {
        let mut frac = x.parse::<i64>().or(Err(error))?;
        let prec = x.len() as u32;
        if prec < PRECISION {
            frac *= 10i64.pow(PRECISION - prec);
        } else if prec > PRECISION {
            frac /= 10i64.pow(prec - PRECISION);
        }
        value += frac;
    }

    match (sign1, sign2) {
        (Some('-'), None) | (None, Some('-')) => {
            value = -value;
        }
        (None, None) => {}
        _ => Err(error)?,
    }

    Ok(value)
}

fn money_to_str<const PRECISION: u32>(value: i64, symbol: &str) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let magnitude = format!("{:01$}", value.abs(), (PRECISION + 1) as usize);
    let split = magnitude.len() - PRECISION as usize;
    format!(
        "{}{}{}.{}",
        sign,
        symbol,
        &magnitude[..split],
        &magnitude[split..]
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::quantity::Value;

    #[test]
    fn currency_value_formatting() {
        type C = Value<AUD>;
        assert_eq!(C::from_repr(9).to_string(), "$0.09");
        assert_eq!(C::from_repr(-9).to_string(), "-$0.09");
        assert_eq!(C::from_repr(79).to_string(), "$0.79");
        assert_eq!(C::from_repr(-79).to_string(), "-$0.79");
        assert_eq!(C::from_repr(101).to_string(), "$1.01");
        assert_eq!(C::from_repr(-101).to_string(), "-$1.01");
        assert_eq!(C::from_repr(1234).to_string(), "$12.34");
        assert_eq!(C::from_repr(-1234).to_string(), "-$12.34");
    }

    #[test]
    fn currency_value_parsing() {
        type C = Value<AUD>;
        assert_eq!(C::from_repr(9), "$0.09".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-9), "-$0.09".parse::<C>().unwrap());
        assert_eq!(C::from_repr(79), "$0.79".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-79), "-$0.79".parse::<C>().unwrap());
        assert_eq!(C::from_repr(101), "$1.01".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-101), "-$1.01".parse::<C>().unwrap());
        assert_eq!(C::from_repr(1234), "$12.34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-1234), "-$12.34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-1234), "$ -12.34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-34), "$ - .34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(34), ".34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(3400), "34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(3400), "34.".parse::<C>().unwrap());
        assert_eq!(C::from_repr(3410), "34.1".parse::<C>().unwrap());
    }

    #[test]
    fn currency_value_comparisons() {
        type C = Value<AUD>;
        assert!(C::from_repr(145) < C::from_repr(155));
        assert!(C::from_repr(155) == C::from_repr(155));
        assert!("$12.34".parse::<C>().unwrap() > "1".parse::<C>().unwrap())
    }
}
