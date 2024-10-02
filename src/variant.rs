use crate::property::Table;

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
    Set(Set),

    #[display("Table")]
    Table(Box<Table>),

    /// A correctable error, below the above
    Invalid(Error),

    /// Bottom of the join lattice
    Nothing,
}

impl Variant {
    pub fn is_nothing(&self) -> bool {
        match self {
            Variant::Nothing => true,
            _ => false,
        }
    }

    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Variant::Table(table) => Some(table),
            _ => None,
        }
    }

    /// Combine two variants according to the rules of a join-semilattice.
    pub fn join_mut(&mut self, other: Self) -> bool {
        use Variant::*;
        match (self, other) {
            (_, Nothing) => false,
            (a @ Nothing, b) => {
                *a = b;
                true
            }
            (_, Invalid(_)) => false,
            (a @ Invalid(_), b) => {
                *a = b;
                true
            }
            (Conflict(_, _), _) => false,
            (a, b @ Conflict(_, _)) => {
                *a = b;
                true
            }
            (Set(a), Set(b)) => a.join_mut(b),
            (Table(a), Table(b)) => a.join_mut(*b),
            (String(a), String(b)) if *a == b => false,
            (Date(a), Date(b)) if *a == b => false,
            (Instant(a), Instant(b)) if *a == b => false,
            (Float(a), Float(b)) if *a == b => false,
            (Int(a), Int(b)) if *a == b => false,
            (a, b) => {
                let a1 = std::mem::replace(a, Nothing);
                *a = Conflict(Box::new(a1), Box::new(b));
                true
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Set(HashSet<Ident>);

impl Set {
    pub fn join_mut(&mut self, other: Self) -> bool {
        let initial = self.0.len();
        self.0.extend(other.0);
        self.0.len() > initial
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

#[derive(Debug, Clone, Display, From, Serialize, Deserialize)]
pub enum Error {
    Detail(String),
}

impl std::error::Error for Error {}
