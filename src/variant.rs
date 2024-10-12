use chrono::{DateTime, NaiveDate, Utc};
use derive_more::derive::{Display, From, TryInto};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

/// A general value.  A monomorphic version of the types used in rules.
/// `Variant` implements `Lattice` such that
/// - `Invalid` variants are inferior to all others.
/// - `Set` variants are joined by union.
/// - `Table` variants are joined by joining their values by key.
/// - `Schedule` variants are immutable and are joined if equal.
/// - Scalar variants are joined if equal.
/// - Other pairs result in a `Conflict` which is the top of the join lattice.   
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

    /// Join by joining members with equal keys.
    #[display("Table")]
    Table(Rc<Table>),

    /// A correctable error, below the above
    Invalid(Error),
}

impl Variant {
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Variant::Table(table) => Some(table),
            _ => None,
        }
    }
}

/// Marks a type as a join semi-lattice. See [wikipedia](en.wikipedia.org/wiki/Semilattice).  
/// The join operation combines two values and is closed, associative, commutative and idempotent.
///
/// This means a.join(b).join(b) == a.join(b) which makes `join` useful in rule systems
/// that iterate joins to reach a fixed point.
///   
/// A lattice has a correspending partial ordering such that c == a.join(b) implies c >= a and c > b.  
/// When implementing `Lattice` it may make sense to also implement `PartialOrd`.  
pub trait Lattice {
    /// Compute a join in place. Return `true` iff self is updated.
    /// This is useful for values that are expensive to clone.
    fn join_update(&mut self, other: Self) -> bool;

    /// Compute a join.  By default, defer to `join_update`.
    fn join(mut self, other: Self) -> Self
    where
        Self: Sized,
    {
        self.join_update(other);
        self
    }
}

impl Lattice for Variant {
    /// Combine two variants according to the rules of a join-semilattice.
    fn join_update(&mut self, other: Self) -> bool {
        use Variant::*;
        match (self, other) {
            (Set(a), Set(b)) => a.join_update(b),
            (Table(a), Table(b)) => join_update_tables(a, b),
            (String(a), String(b)) if *a == b => false,
            (Date(a), Date(b)) if *a == b => false,
            (Instant(a), Instant(b)) if *a == b => false,
            (Float(a), Float(b)) if *a == b => false,
            (Int(a), Int(b)) if *a == b => false,
            (Conflict(_, _), _) => false,
            (a, b @ Conflict(_, _)) => {
                *a = b;
                true
            }
            (_, Invalid(_)) => false,
            (a @ Invalid(_), b) => {
                *a = b;
                true
            }
            (a, b) => {
                let a1 = std::mem::replace(a, Int(0));
                *a = Conflict(Box::new(a1), Box::new(b));
                true
            }
        }
    }
}

fn join_update_tables(a: &mut Rc<Table>, b: Rc<Table>) -> bool {
    if Rc::ptr_eq(a, &b) {
        false
    } else {
        Rc::make_mut(a).join_update(Rc::unwrap_or_clone(b))
    }
}

/// A set of `Ident`s.  This implements `Lattice` and `join` is by set union.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Set(HashSet<Ident>);

impl Lattice for Set {
    fn join_update(&mut self, other: Self) -> bool {
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

/// A `Table` is a map of `Ident` to `Variant`.  It is monomorphic but
/// represents typed data. Each `Ident` represents a `Property<A>` for some type `A`
/// and the corresponding `Variant` represents an `A` value.
///
/// `Table` implements `Lattice`.  Joining a table joins values of the same key.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Table(HashMap<Ident, Variant>);

impl Table {
    /// Create an empty Table
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Mutable access to a value
    pub fn get_mut(&mut self, name: &Ident) -> Option<&mut Variant> {
        self.0.get_mut(name)
    }

    /// Borrow a value
    pub fn get(&self, name: &Ident) -> Option<&Variant> {
        self.0.get(name)
    }

    /// Insert an entry into the Table.
    pub fn insert(&mut self, name: Ident, value: Variant) -> Option<Variant> {
        self.0.insert(name, value)
    }
}

impl Lattice for Table {
    fn join_update(&mut self, other: Self) -> bool {
        let mut modified = false;
        for (k, v) in other.0 {
            if let Some(u) = self.0.get_mut(&k) {
                modified |= u.join_update(v)
            } else {
                self.0.insert(k, v);
                modified = true;
            }
        }
        modified
    }
}

/// An `Ident` identifies a property or an element of a set.
#[derive(PartialEq, Eq, Hash, Debug, Display, From, Clone)]
pub enum Ident {
    NonIntern(String),
    Intern(&'static str),
    Anonymous(u64),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExternalIdent {
    NonIntern(String),
    Anonymous(u64),
}

impl Serialize for Ident {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let x = match self {
            Ident::NonIntern(i) => ExternalIdent::NonIntern(i.clone()),
            Ident::Intern(i) => ExternalIdent::NonIntern(i.to_string()),
            Ident::Anonymous(i) => ExternalIdent::Anonymous(*i),
        };
        x.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Ident {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let x: ExternalIdent = ExternalIdent::deserialize(deserializer)?;
        match x {
            ExternalIdent::NonIntern(i) => Ok(Ident::NonIntern(i)),
            ExternalIdent::Anonymous(i) => Ok(Ident::Anonymous(i)),
        }
    }
}

/// A skeleton Error type
#[derive(Debug, Clone, Display, From, Serialize, Deserialize)]
pub enum Error {
    Detail(String),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Detail(value.to_string())
    }
}

impl std::error::Error for Error {}

impl<A> From<Result<A, Error>> for Variant
where
    A: Into<Variant>,
{
    fn from(value: Result<A, Error>) -> Self {
        match value {
            Ok(a) => a.into(),
            Err(e) => e.into(),
        }
    }
}

static CONV_FAIL: &'static str = "numeric conversion failed";

impl From<u32> for Variant {
    fn from(value: u32) -> Self {
        (value as i64).into()
    }
}

impl TryFrom<Variant> for u32 {
    type Error = Error;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        let i: i64 = value.try_into().or(Err(CONV_FAIL))?;
        Ok(i.try_into().or(Err(CONV_FAIL))?)
    }
}

impl From<i32> for Variant {
    fn from(value: i32) -> Self {
        (value as i64).into()
    }
}

impl TryFrom<Variant> for i32 {
    type Error = Error;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        let i: i64 = value.try_into().or(Err(CONV_FAIL))?;
        Ok(i.try_into().or(Err(CONV_FAIL))?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cheap_clones() {
        let v: Variant = Rc::new(Table::new()).into();
        let w = v.clone();
        let t: Rc<Table> = v.try_into().unwrap();
        let u: Rc<Table> = w.try_into().unwrap();
        assert!(Rc::ptr_eq(&t, &u))
    }
}
