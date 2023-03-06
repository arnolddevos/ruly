use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, iter::once};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Variant {
    String(String),
    Date(NaiveDate),
    Instant(DateTime<Utc>),
    Float(f64),
    Int(i64),
    Error(HashSet<String>),
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::String(v) => v.fmt(f),
            Variant::Date(v) => v.fmt(f),
            Variant::Instant(v) => v.fmt(f),
            Variant::Float(v) => v.fmt(f),
            Variant::Int(v) => v.fmt(f),
            Variant::Error(vs) => fmt_error(vs, f),
        }
    }
}

fn fmt_error(vs: &HashSet<String>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if vs.len() == 1 {
        f.write_str("Error:")?
    } else {
        f.write_str("Conflict:")?
    }
    for v in vs.iter() {
        f.write_str(" ")?;
        f.write_str(v)?
    }
    Ok(())
}

impl Variant {
    pub fn join(&self, other: &Variant) -> Variant {
        match (self, other) {
            (Variant::Error(vs0), Variant::Error(vs1)) => {
                Variant::Error(vs0.iter().cloned().chain(vs1.iter().cloned()).collect())
            }
            (Variant::Error(vs0), v1) => {
                Variant::Error(vs0.iter().cloned().chain(once(v1.to_string())).collect())
            }
            (v0, Variant::Error(vs1)) => {
                Variant::Error(once(v0.to_string()).chain(vs1.iter().cloned()).collect())
            }
            (v0, v1) => {
                if v0 == v1 {
                    v0.clone()
                } else {
                    Variant::Error([v0.to_string(), v1.to_string()].into())
                }
            }
        }
    }
}

pub trait KeywordType
where
    Self: Sized,
{
    fn normalise(value: &str) -> Option<Keyword<Self>>;
}

#[derive(Clone, Debug)]
pub struct Keyword<T>(String, T);

impl TryFrom<Variant> for String {
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Error(_) => Err(()),
            value => Ok(value.to_string()),
        }
    }
}

impl From<String> for Variant {
    fn from(value: String) -> Self {
        Variant::String(value)
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
        Variant::Int(value)
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
        Variant::Float(value)
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
        Variant::Date(value)
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
        Variant::Instant(value)
    }
}

impl<T> TryFrom<Variant> for Keyword<T>
where
    T: KeywordType,
{
    type Error = ();
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::String(x) => {
                if let Some(kw) = T::normalise(&x) {
                    Ok(kw)
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

impl<T> From<Keyword<T>> for Variant {
    fn from(value: Keyword<T>) -> Self {
        Variant::String(value.0)
    }
}
