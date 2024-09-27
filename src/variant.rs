use crate::property::Table;

use super::property::Ident;
use chrono::{DateTime, NaiveDate, Utc};
use derive_more::derive::{Display, From, TryInto};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;

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
    Table(Rc<Table>),

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
            (a, Nothing) => Hold(a),
            (Nothing, b) => Promote(b),
            (a, Invalid(_)) => Hold(a),
            (Invalid(_), b) => Promote(b),
            (a @ Conflict(_, _), _) => Hold(a),
            (_, b @ Conflict(_, _)) => Promote(b),
            (Set(x), Set(y)) => x.join(y),
            (Table(x), Table(y)) => x.join(y),
            (String(a), String(b)) if a == b => Hold(String(a)),
            (Date(a), Date(b)) if a == b => Hold(Date(a)),
            (Instant(a), Instant(b)) if a == b => Hold(Instant(a)),
            (Float(a), Float(b)) if a == b => Hold(Float(a)),
            (Int(a), Int(b)) if a == b => Hold(Int(a)),
            (a, b) => Promote(Conflict(Box::new(a), Box::new(b))),
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

    pub fn join(self, other: Set) -> JoinResult {
        use JoinResult::*;
        use Variant::Set;

        if self.is_superset(&other) {
            Hold(Set(self))
        } else {
            Promote(Set(self.union(other)))
        }
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

impl Table {
    pub fn join(self: Rc<Self>, other: Rc<Self>) -> JoinResult {
        use JoinResult::*;

        let mut table = Table::new();
        let mut promoted = false;

        for (name, rhs) in Rc::unwrap_or_clone(other).into_iter() {
            let lhs = self.get(&name);
            match lhs.join(rhs) {
                Hold(_) => {}
                Promote(value) => {
                    if !promoted {
                        table = self.as_ref().clone();
                    }
                    table.insert(name, value);
                    promoted = true;
                }
            }
        }

        if promoted {
            Promote(Variant::Table(Rc::new(table)))
        } else {
            Hold(Variant::Table(self))
        }
    }
}

#[derive(Debug, Clone)]
pub enum JoinResult {
    Hold(Variant),
    Promote(Variant),
}

#[derive(Debug, Clone, Display, From, Serialize, Deserialize)]
pub enum Error {
    Detail(String),
}

impl std::error::Error for Error {}
