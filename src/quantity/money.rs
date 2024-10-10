use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0, one_of},
    combinator::{all_consuming, opt},
    sequence::{preceded, terminated, tuple},
};

use crate::variant::{Error, Variant};
use std::{fmt::Display, marker::PhantomData, str::FromStr};

/// Marks a type (usually a unit type) as representing a currency such as
/// US dollars or Euros.  Members of this trait give the fixed characteristics
/// of a currency.  So far, that is just the symbol.
pub trait Currency {
    const SYMBOL: &'static str;
    const PRECISION: u32;
    const FACTOR: i64 = 10i64.pow(Self::PRECISION);
}

/// Australian currency.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AUD;

impl Currency for AUD {
    const SYMBOL: &'static str = "$";
    const PRECISION: u32 = 2;
}

/// US currency.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct USD;

impl Currency for USD {
    const SYMBOL: &'static str = "$";
    const PRECISION: u32 = 2;
}

/// Represents an amount of money in a given currency.
/// The currency is a type parameter and money of different
/// currencies are not comparable.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money<C> {
    cents: i64,
    currency: PhantomData<C>,
}

impl<C> Money<C> {
    pub fn from_repr(cents: i64) -> Self {
        Self {
            cents,
            currency: PhantomData,
        }
    }
}

impl<C> FromStr for Money<C>
where
    C: Currency,
{
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let error = "error in money value";
        let spaces = multispace0::<&str, nom::error::Error<&str>>;
        let digits = digit1::<&str, nom::error::Error<&str>>;

        let (_, (sign1, _, sign2, whole, frac)) = all_consuming(terminated(
            tuple((
                opt(terminated(one_of("+-"), spaces)),
                opt(terminated(tag(C::SYMBOL), spaces)),
                opt(terminated(one_of("+-"), spaces)),
                opt(digits),
                opt(preceded(tag("."), opt(digits))),
            )),
            spaces,
        ))(input)
        .or(Err(error))?;

        let mut value: i64 = 0;

        if let Some(x) = whole {
            value += x.parse::<i64>().or(Err(error))? * C::FACTOR;
        }

        if let Some(Some(x)) = frac {
            let mut frac = x.parse::<i64>().or(Err(error))?;
            let prec = x.len() as u32;
            if prec < C::PRECISION {
                frac *= 10i64.pow(C::PRECISION - prec);
            } else if prec > C::PRECISION {
                frac /= 10i64.pow(prec - C::PRECISION);
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

        Ok(Self {
            cents: value,
            currency: PhantomData,
        })
    }
}

impl<C> Display for Money<C>
where
    C: Currency,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = if self.cents < 0 { "-" } else { "" };
        let magnitude = format!("{:01$}", self.cents.abs(), (C::PRECISION + 1) as usize);
        let split = magnitude.len() - C::PRECISION as usize;
        f.write_fmt(format_args!(
            "{}{}{}.{}",
            sign,
            C::SYMBOL,
            &magnitude[..split],
            &magnitude[split..]
        ))
    }
}

impl<C> From<Money<C>> for Variant {
    fn from(value: Money<C>) -> Self {
        value.cents.into()
    }
}

impl<C> TryFrom<Variant> for Money<C> {
    type Error = Error;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        Ok(Self {
            cents: value.try_into().or(Err("incorrect variant for money"))?,
            currency: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn currency_formating() {
        type C = Money<AUD>;
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
    fn currency_parsing() {
        type C = Money<AUD>;
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
}
