use crate::variant::{Lattice, Variant};
use derive_more::derive::{Display, From};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

/// A `Table` is a map of `Ident` to `Variant`.  
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

    /// Borrow a value from a nested table
    pub fn get_path(&self, path: &IdentPath) -> Option<&Variant> {
        let mut step = self;
        for next in path.prefix.iter() {
            step = step.get(next)?.as_table()?;
        }
        step.get(&path.subject)
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

/// A set of `Ident`s.  This implements `Lattice` and `join` is by set union.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Set(HashSet<Ident>);

impl Set {
    pub fn new(elems: impl IntoIterator<Item = Ident>) -> Self {
        Self(elems.into_iter().collect())
    }
}

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

/// An `Ident` identifies a table entry or an element of a set.
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

/// An `IdentPath` designates a property that may be in a nested `Table`.
/// Tables can be nested to any depth because a `Variant` value can be a `Table`.
/// An `IdentPath` has at least one `Ident`
#[derive(Debug, Clone)]
pub struct IdentPath {
    prefix: Vec<Ident>, // first elements of the path
    subject: Ident,     // the last element of the path
}

impl IdentPath {
    /// Construct a path of length 1 from an `Ident`
    pub fn new(subject: Ident) -> Self {
        let prefix = Vec::new();
        Self { prefix, subject }
    }

    /// Append an `Ident` to a path.
    pub fn append(self, subject: Ident) -> Self {
        let mut prefix = self.prefix;
        prefix.push(self.subject);
        Self { prefix, subject }
    }
}
