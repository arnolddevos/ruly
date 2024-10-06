use crate::{
    property::{prop, Property},
    variant::{Error, Variant},
};
use regex::Regex;
use std::{fmt::Display, str::FromStr, sync::LazyLock};

/// `Quantity` and `Value` help to define new types for `Property`s.
///
/// For example, `Value<Currency>` can be used to define a property of type `Property<Value<Currency>>`
/// for money values.   `Currency` is just a unit type, the necessary definitions are
/// in its implementation of trait `Quantity`.   This includes a represention type, `Repr` which
/// is `i64` in this case.
///
/// The type `Value<Q>` has blanket implementations of `TryFrom<Variant>` and `Into<Variant>`, which are needed
/// by the rule system, and `FromStr` and `Display`. The type `Q` names the the quantity being represented and
/// its implmenation of trait `Quantity` provides the definitions needed.
///
/// A in implementation of `Quantity` defines a `Repr` which should be chosen to be
/// convertible to and from `Variant`.  Methods `parse()` and `format()` should provide a string representation
/// specific to the quantity.  These should round-trip so that `v == parse(format(v)).unwrap()`.
pub trait Quantity {
    type Repr;

    fn parse(text: &str) -> Result<Self::Repr, Error>;
    fn format(value: &Self::Repr) -> String;
}

/// A wrapper for `Quantity` representations.  Essentially a _newtype_ for `Quantity::Repr`.
/// Blank implimentations are defined for `TryFrom<Variant>`, `Into<Variant>`, `FromStr` and `Display`.
pub struct Value<Q: Quantity>(Q::Repr);

impl<Q> From<Value<Q>> for Variant
where
    Q: Quantity,
    Q::Repr: Into<Variant>,
{
    fn from(value: Value<Q>) -> Self {
        value.0.into()
    }
}

impl<Q> TryFrom<Variant> for Value<Q>
where
    Q: Quantity,
    Q::Repr: Into<Variant>,
{
    type Error = Error;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

impl<Q> FromStr for Value<Q>
where
    Q: Quantity,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Q::parse(s)?))
    }
}

impl<Q> Display for Value<Q>
where
    Q: Quantity,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&Q::format(&self.0))
    }
}

/// Construct a Property in a const context e.g.
/// `pub static FRED: Property<Value<Currency>> = quant("fred");`
pub const fn quant<Q: Quantity>(name: &'static str) -> Property<Value<Q>> {
    prop(name)
}

pub struct Currency;

type Lex = LazyLock<Regex>;

impl Quantity for Currency {
    type Repr = i64;

    fn format(value: &Self::Repr) -> String {
        let value = *value;
        let sign = if value < 0 { "-" } else { "" };
        let digits = format!("{:}", value.abs());
        let split = digits.len() - 2;
        let whole = &digits[..split];
        let cents = &digits[split..];
        format!("{sign}${whole}.{cents}")
    }

    fn parse(text: &str) -> Result<Self::Repr, Error> {
        static RE: Lex = Lex::new(|| {
            Regex::new(
                r"^
                (?x)  # ignore ws in this pattern

                (?:   # a leading sign
                    (?<sign1>[+-])\s*[$]?\s*
                )|
                (?:   # a leading symbol
                    [$]\s*(?<sign2>[+-])?\s*
                )|

                # neither sign nor symbol
                (?:)
                
                # the number of whole dollars (no commas)
                (?<whole>\d+)\s*

                (?: # fraction part
                    [.]\s*
                
                    # in cents
                    (?<cents>\d\d)|
                    
                    # in deci-dollars I guess
                    (?<decis>\d)|
                    
                    # not given
                    (?:)
                )|

                # no decimal point
                (?:)
            $",
            )
            .unwrap()
        });

        let capts = RE.captures(text).ok_or("invalid currency syntax")?;
        let mut amount: i64 = capts
            .name("whole")
            .unwrap()
            .as_str()
            .parse::<i64>()
            .unwrap()
            * 100;
        if let Some(decis) = capts.name("decis") {
            amount += decis.as_str().parse::<i64>().unwrap() * 10;
        }
        if let Some(cents) = capts.name("cents") {
            amount += cents.as_str().parse::<i64>().unwrap();
        }
        if let Some(sign) = capts.name("sign1").or(capts.name("sign2")) {
            if sign.as_str() == "-" {
                amount = -amount;
            }
        }
        Ok(amount)
    }
}
