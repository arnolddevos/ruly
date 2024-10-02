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

    /// Typed access to the value of a property or path.
    pub fn get1<A>(&self, prop1: &impl PropOrPath<A>) -> Option<A>
    where
        A: Model,
    {
        prop1.extract(&self)
    }

    /// Typed access to the joint values of a pair of properties or paths.
    pub fn get2<A, B>(
        &self,
        prop1: &impl PropOrPath<A>,
        prop2: &impl PropOrPath<B>,
    ) -> Option<(A, B)>
    where
        A: Model,
        B: Model,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?))
    }

    /// Typed access to the joint values of a triplet of properties or paths.
    pub fn get3<A, B, C>(
        &self,
        prop1: &impl PropOrPath<A>,
        prop2: &impl PropOrPath<B>,
        prop3: &impl PropOrPath<C>,
    ) -> Option<(A, B, C)>
    where
        A: Model,
        B: Model,
        C: Model,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?, self.get1(prop3)?))
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

impl<A> Div<&Property<A>> for &Property<Rc<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: &Property<A>) -> Self::Output {
        Path::<A> {
            prefix: Vec::from([self.name.clone()]),
            subject: rhs.clone(),
        }
    }
}

impl<A> Div<&Property<A>> for Path<Rc<Table>> {
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

/// A rule can refer to data in a `Table` uniformly by `Property` or `Path`.
pub trait PropOrPath<A> {
    fn extract(&self, table: &Table) -> Option<A>;
}

impl<A: Model> PropOrPath<A> for Property<A> {
    fn extract(&self, table: &Table) -> Option<A> {
        table.get(&self.name)?.clone().try_into().ok()
    }
}

impl<A: Model> PropOrPath<A> for Path<A> {
    fn extract(&self, table: &Table) -> Option<A> {
        let mut prefix = self.prefix.iter();
        let v = if let Some(next) = prefix.next() {
            let mut step = table.get(next)?.as_table()?;
            for next in prefix {
                step = step.get(next)?.as_table()?;
            }
            step.get(&self.subject.name)?
        } else {
            table.get(&self.subject.name)?
        };
        v.clone().try_into().ok()
    }
}
