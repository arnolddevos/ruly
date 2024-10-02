use crate::variant::{Ident, Table, Variant};
use derive_more::From;
use std::ops::Div;
use std::{marker::PhantomData, str::FromStr};

/// A type used in a rule or property must implement the `Model` marker trait.
/// This implies it implements the string and `Variant` conversion traits mentioned.
pub trait Model: FromStr + ToString + TryFrom<Variant> + Into<Variant> {}

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

impl Model for String {}

// pub static FRED: Property<String> = Property::new("fred".to_string());

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
