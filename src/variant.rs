use super::property::Ident;
use chrono::{DateTime, NaiveDate, Utc};
use derive_more::derive::{Display, From, TryInto};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Display;

/// A general value.  A monomorphic version of Model type.
#[derive(Serialize, Deserialize, Clone, Debug, From, TryInto, Display)]
pub enum Variant {
    /// Top of the join lattice
    #[display("conflict {} {}", _0, _1)]
    Conflict(Box<Variant>, Box<Variant>),

    /// Join by equality
    String(String),
    Date(NaiveDate),
    Instant(DateTime<Utc>),
    Float(f64),
    Int(i64),

    /// Join by union
    StringSet(Set),

    /// A correctable error, below the above
    Invalid(Error),

    /// Bottom of the join lattice
    Nothing,
}

impl Variant {
    /// Combine two variants according to the rules of a join-semilattice.
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
                    Greater(StringSet(x.union(y)))
                }
            }
            (String(a), String(b)) if a == b => Left(String(a)),
            (Date(a), Date(b)) if a == b => Left(Date(a)),
            (Instant(a), Instant(b)) if a == b => Left(Instant(a)),
            (Float(a), Float(b)) if a == b => Left(Float(a)),
            (Int(a), Int(b)) if a == b => Left(Int(a)),
            (a, b) => Greater(Conflict(Box::new(a), Box::new(b))),
        }
    }

    pub fn is_nothing(&self) -> bool {
        match self {
            Variant::Nothing => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Set(HashSet<Ident>);

impl Set {
    /// The HashSet::union method doesn't give me this signature.
    /// TODO: investigate.
    pub fn union(self, other: Set) -> Set {
        let (Set(mut x), Set(y)) = if self.0.len() >= other.0.len() {
            (self, other)
        } else {
            (other, self)
        };
        x.extend(y);
        Set(x)
    }

    pub fn is_superset(&self, other: &Set) -> bool {
        self.0.is_superset(&other.0)
    }
}

impl Display for Set {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        for i in self.0.iter() {
            i.fmt(f)?;
            f.write_str(",")?;
        }
        f.write_str("]")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum JoinResult {
    Left(Variant),
    Right(Variant),
    Greater(Variant),
}

#[derive(Debug, Clone, Display, From, Serialize, Deserialize)]
pub enum Error {
    Detail(String),
}

impl std::error::Error for Error {}
