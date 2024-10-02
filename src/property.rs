use crate::variant::{Lattice, Variant};
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Div;
use std::{marker::PhantomData, rc::Rc, str::FromStr};

/// A type used in a rule or property must implement the `Model` marker trait.
/// This implies it implements the string and `Variant` conversion traits mentioned.
pub trait Model: FromStr + ToString + TryFrom<Variant> + Into<Variant> {}

/// An `Ident` identifies a property or (see `Variant`) an element of a set.
#[derive(PartialEq, Eq, Hash, Debug, Display, From, Clone, Serialize, Deserialize)]
pub struct Ident(Rc<str>);

/// A property gives a name (`Ident`) and canonical type of a value.
/// A property is also supposed to confer some meaning to a value,
/// ie its interpretation or what it represents.
/// If two properties have the same name they are equal.
/// Equal properties should have the same type (but this is not enforced).
#[derive(Eq, Hash, Debug)]
pub struct Property<A> {
    pub name: Ident,
    marker: PhantomData<A>,
}

impl<A> Clone for Property<A> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            marker: PhantomData,
        }
    }
}

impl<A, B> PartialEq<Property<B>> for Property<A> {
    fn eq(&self, other: &Property<B>) -> bool {
        self.name == other.name
    }
}

impl<A: Model> Property<A> {
    pub fn new(ident: impl Into<Ident>) -> Self {
        Self {
            name: ident.into(),
            marker: PhantomData,
        }
    }
}

/// A `Table` is a map of `Ident` to `Variant`.  It is monomorphic but
/// represents typed data. Each `Ident` represents a `Property<A>` for some type `A`
/// and the corresponding `Variant` represents an `A` value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Table(HashMap<Ident, Variant>);

impl Table {
    /// Create an empty Table
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Untyped mutable access
    pub fn get_mut(&mut self, name: &Ident) -> Option<&mut Variant> {
        self.0.get_mut(name)
    }

    /// Typed mutable access
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

/// A `Path` designates a property in a nested `Table`.
/// Tables can be nested to any depth because a `Variant` value can be a `Table`.
/// A `Path` is constructed by connecting `Property`s with the `/` operator.
#[derive(Debug, Clone)]
pub struct Path<A> {
    prefix: Vec<Ident>,
    subject: Property<A>,
}

impl<A> Div<&Property<A>> for &Property<Box<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: &Property<A>) -> Self::Output {
        Path::<A> {
            prefix: Vec::from([self.name.clone()]),
            subject: rhs.clone(),
        }
    }
}

impl<A> Div<&Property<A>> for Path<Box<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: &Property<A>) -> Self::Output {
        let mut prefix = self.prefix;
        prefix.push(self.subject.name);
        Path::<A> {
            prefix,
            subject: rhs.clone(),
        }
    }
}

impl<A> Into<Path<A>> for &Property<A> {
    fn into(self) -> Path<A> {
        Path::<A> {
            prefix: Vec::new(),
            subject: self.clone(),
        }
    }
}

/// The ability to query a `Table` implemented for `Property` and `Path`.
pub trait Query {
    type Output;
    fn query(&self, table: &Table) -> Option<Self::Output>;
}

impl<A: Model> Query for Property<A> {
    type Output = A;

    fn query(&self, table: &Table) -> Option<A> {
        table.get(&self.name)?.clone().try_into().ok()
    }
}

impl<A: Model> Query for Path<A> {
    type Output = A;

    fn query(&self, table: &Table) -> Option<A> {
        let mut step = table;
        for next in self.prefix.iter() {
            step = step.get(next)?.as_table()?;
        }
        step.get(&self.subject.name)?.clone().try_into().ok()
    }
}
