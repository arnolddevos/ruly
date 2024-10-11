#![cfg(feature = "quantity")]

pub mod date;
pub mod money;

use crate::{
    property::{prop, Property},
    variant::{Error, Variant},
};

use std::{
    fmt::{Debug, Display},
    ops::{Add, Mul, Neg},
    str::FromStr,
};

/// `Quantity` and `Value` help to define new types for `Property`s.
///
/// A quantity is a unit type that implements trait `Quantity`.  
/// This defines a representation ,`Repr`, that should be convertable to and from `Variant`.
/// And it defines `parse` and `format` functions that convert
/// this representation to and from string.
///
/// The type `Value<Q>`, where Q is a quantity, wraps the representation and provides  
/// blanket implementations of `TryFrom<Variant>` and `Into<Variant>`, which are needed
/// by the rule system, and `FromStr` and `Display`.
///
/// In the rule system, a property for a quantity has type `Property<Value<Q>>` and
/// can be conveniently defined by function `quant`.
///
/// For example, a `Value<AUD>` is an amount in Australian dollars.
/// A property named `balance` is defined by `quant::<AUD>('balance')` and has type `Property<Value<AUD>>`.
/// The `Quantity` implementation for `AUD` defines the representation as `i64`.   
///
/// The advantages of using `Value<AUD>` instead of simply `i64` are (1) type safety
/// and (2) currency-specific conversions to and from string.
pub trait Quantity {
    type Repr;

    fn parse(text: &str) -> Result<Self::Repr, Error>;
    fn format(value: &Self::Repr) -> String;
}

/// A wrapper for `Quantity` representations.  Essentially a _newtype_ for `Quantity::Repr`.
/// Blanket implimentations are defined for `TryFrom<Variant>`, `Into<Variant>`, `FromStr` and `Display`.
pub struct Value<Q: Quantity>(Q::Repr);

/// Construct a Property in a const context e.g.
/// `pub static BALANCE: Property<Value<AUD>> = quant("balance");`
pub const fn quant<Q: Quantity>(name: &'static str) -> Property<Value<Q>> {
    prop(name)
}

impl<Q: Quantity> Value<Q> {
    /// Construct a `Value` from its representation.
    pub fn from_repr(repr: Q::Repr) -> Self {
        Self(repr)
    }

    /// Reduce a `Value` to its representation
    pub fn to_repr(self) -> Q::Repr {
        self.0
    }
}

impl<Q> Value<Q>
where
    Q: Quantity<Repr = i64>,
{
    /// Scale a `Value` that has an i64 representation
    pub fn scale(self, factor: f64) -> Self {
        Self(((self.0 as f64) * factor) as i64)
    }
}

// Manual implementation of this trait to provide correct
// requirements on `Quantity` parameter.
impl<Q> Clone for Value<Q>
where
    Q: Quantity,
    Q::Repr: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Manual implementation of this trait to provide correct
// requirements on `Quantity` parameter.
impl<Q> Debug for Value<Q>
where
    Q: Quantity,
    Q::Repr: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value").field(&self.0).finish()
    }
}

// Manual implementation of this trait to provide correct
// requirements on `Quantity` parameter.
impl<Q> PartialEq for Value<Q>
where
    Q: Quantity,
    Q::Repr: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// Manual implementation of this trait to provide correct
// requirements on `Quantity` parameter.
impl<Q> Eq for Value<Q>
where
    Q: Quantity,
    Q::Repr: Eq,
{
}

// Manual implementation of this trait to provide correct
// requirements on `Quantity` parameter.
impl<Q> PartialOrd for Value<Q>
where
    Q: Quantity,
    Q::Repr: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// Manual implementation of this trait to provide correct
// requirements on `Quantity` parameter.
impl<Q> Ord for Value<Q>
where
    Q: Quantity,
    Q::Repr: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

// A Quantity can be scaled if the representation can be scaled
impl<Q, S> Mul<S> for Value<Q>
where
    Q: Quantity,
    Q::Repr: Mul<S, Output = Q::Repr>,
{
    type Output = Self;

    fn mul(self, rhs: S) -> Self::Output {
        Self(self.0 * rhs)
    }
}

// A Quantity can be added with the same species if representations can be added
impl<Q> Add<Self> for Value<Q>
where
    Q: Quantity,
    Q::Repr: Add<Q::Repr, Output = Q::Repr>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

// A Quantity can be negated if its representation can be negated
impl<Q> Neg for Value<Q>
where
    Q: Quantity,
    Q::Repr: Neg<Output = Q::Repr>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
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
    Q::Repr: TryFrom<Variant>,
{
    type Error = Error;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .try_into()
                .or(Err("incorrect type stored in variant"))?,
        ))
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
