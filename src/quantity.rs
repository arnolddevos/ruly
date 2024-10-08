use crate::{
    property::{prop, Property},
    variant::{Error, Variant},
};
use regex::Regex;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
    sync::LazyLock,
};

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

impl<Q: Quantity> Value<Q> {
    pub fn from_repr(repr: Q::Repr) -> Self {
        Self(repr)
    }

    pub fn to_repr(self) -> Q::Repr {
        self.0
    }
}

impl<Q> Clone for Value<Q>
where
    Q: Quantity,
    Q::Repr: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Q> Debug for Value<Q>
where
    Q: Quantity,
    Q::Repr: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value").field(&self.0).finish()
    }
}

impl<Q> PartialEq for Value<Q>
where
    Q: Quantity,
    Q::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

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
        let digits = format!("{:03}", value.abs());
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn currency_formating() {
        type C = Value<Currency>;
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
        type C = Value<Currency>;
        assert_eq!(C::from_repr(9), "$0.09".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-9), "-$0.09".parse::<C>().unwrap());
        assert_eq!(C::from_repr(79), "$0.79".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-79), "-$0.79".parse::<C>().unwrap());
        assert_eq!(C::from_repr(101), "$1.01".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-101), "-$1.01".parse::<C>().unwrap());
        assert_eq!(C::from_repr(1234), "$12.34".parse::<C>().unwrap());
        assert_eq!(C::from_repr(-1234), "-$12.34".parse::<C>().unwrap());
    }
}
