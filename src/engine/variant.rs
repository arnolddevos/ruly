use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
    /// StringSet(HashSet<String>),

    /// A correctable error, below the above
    Invalid(String),

    /// Bottom of the join lattice
    Nothing,
}

impl Variant {
    pub fn join(self, other: Self) -> Self {
        // micro-optimisation: we expect to be called when a!=b and often when a or b is Nothing
        match (self, other) {
            (a, Self::Nothing) => a,
            (Self::Nothing, b) => b,
            (a, Self::Invalid(_)) => a,
            (Self::Invalid(_), b) => b,
            (Self::Conflict(a, b), _) => Self::Conflict(a, b),
            (_, Self::Conflict(a, b)) => Self::Conflict(a, b),
            (a, b) if a == b => a,
            (a, b) => Self::Conflict(a.to_string(), b.to_string()),
        }
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(v) => v.fmt(f),
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
