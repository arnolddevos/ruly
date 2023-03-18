use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, hash::Hash};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Variant {
    /// Top of the join lattice
    Conflict(String, String),

    /// Join by equality
    String(String),
    Date(NaiveDate),
    Instant(DateTime<Utc>),
    Float(f64),
    Int(i64),

    /// Join by union
    StringSet(HashSet<String>),

    /// A correctable error, below the above
    Invalid(String),

    /// Bottom of the join lattice
    Nothing,
}

impl Variant {
    pub fn join(self, other: Self) -> JoinResult {
        use JoinResult::*;
        use Variant::*;
        match (self, other) {
            (a, Nothing) => Left(a),
            (Nothing, b) => Right(b),
            (a, Invalid(_)) => Left(a),
            (Invalid(_), b) => Right(b),
            (a @ Conflict(_, _), _) => Left(a),
            (_, b @ Conflict(_, _)) => Right(b),
            (StringSet(x), StringSet(y)) => {
                if x.is_superset(&y) {
                    Left(StringSet(x))
                } else if y.is_superset(&x) {
                    Right(StringSet(y))
                } else {
                    Greater(StringSet(union(x, y)))
                }
            }
            (a, b) if a == b => Left(a),
            (a, b) => Greater(Conflict(a.to_string(), b.to_string())),
        }
    }
}

/// The HashSet::union method doesn't give me this signature.
/// TODO: investigate.
fn union<A>(mut x: HashSet<A>, mut y: HashSet<A>) -> HashSet<A>
where
    A: Eq + Hash,
{
    if x.len() > y.len() {
        x.extend(y);
        x
    } else {
        y.extend(x);
        y
    }
}

pub enum JoinResult {
    Left(Variant),
    Right(Variant),
    Greater(Variant),
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(v) => v.fmt(f),
            Self::StringSet(v) => write!(f, "{v:?}"),
            Self::Date(v) => v.fmt(f),
            Self::Instant(v) => v.fmt(f),
            Self::Float(v) => v.fmt(f),
            Self::Int(v) => v.fmt(f),
            Self::Conflict(a, b) => write!(f, "Conflict: {a} and {b}"),
            Self::Invalid(a) => write!(f, "Invalid: {a}"),
            Self::Nothing => f.write_str("Nothing"),
        }
    }
}

impl TryFrom<Variant> for String {
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Conflict(_, _) => Err(()),
            value => Ok(value.to_string()),
        }
    }
}

impl From<String> for Variant {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl TryFrom<Variant> for i64 {
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Int(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<i64> for Variant {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl TryFrom<Variant> for f64 {
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Float(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<f64> for Variant {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl TryFrom<Variant> for NaiveDate {
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Date(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<NaiveDate> for Variant {
    fn from(value: NaiveDate) -> Self {
        Self::Date(value)
    }
}

impl TryFrom<Variant> for DateTime<Utc> {
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Instant(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<DateTime<Utc>> for Variant {
    fn from(value: DateTime<Utc>) -> Self {
        Self::Instant(value)
    }
}

impl<T> From<Option<T>> for Variant
where
    Self: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => t.into(),
            None => Self::Nothing,
        }
    }
}

impl<T, E> From<Result<T, E>> for Variant
where
    Self: From<T>,
    E: Display,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(t) => t.into(),
            Err(e) => Self::Invalid(e.to_string()),
        }
    }
}
