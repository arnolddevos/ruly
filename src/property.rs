use crate::variant::Variant;
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
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn kv(name: Ident, value: Variant) -> Self {
        let mut map = HashMap::new();
        map.insert(name, value);
        Self(map)
    }

    pub fn get(&self, name: &Ident) -> Variant {
        self.0.get(name).cloned().unwrap_or(Variant::Nothing)
    }

    pub fn insert(&mut self, name: Ident, value: Variant) -> Variant {
        self.0.insert(name, value).unwrap_or(Variant::Nothing)
    }

    pub fn view<'a>(&'a self) -> View<'a> {
        View(&self)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Ident, &Variant)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Ident, Variant)> {
        self.0.into_iter()
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

/// A rule can refer to data in a `View` uniformly by `Property` or `Path`.
pub trait PropOrPath<A> {
    fn extract(&self, table: &Table) -> Option<A>;
}

impl<A: Model> PropOrPath<A> for Property<A> {
    fn extract(&self, table: &Table) -> Option<A> {
        table.get(&self.name).try_into().ok()
    }
}

impl<A: Model> PropOrPath<A> for Path<A> {
    fn extract(&self, table: &Table) -> Option<A> {
        let mut prefix = self.prefix.iter();
        let v = if let Some(next) = prefix.next() {
            let mut step: Rc<Table> = TryInto::<Rc<Table>>::try_into(table.get(next)).ok()?;
            for next in prefix {
                step = TryInto::<Rc<Table>>::try_into(step.get(next)).ok()?;
            }
            step.get(&self.subject.name)
        } else {
            table.get(&self.subject.name)
        };
        v.try_into().ok()
    }
}

/// A `View` provides typed access to a `Table` via
/// `Property` or `Path` keys.
pub struct View<'a>(&'a Table);

impl<'a> View<'a> {
    pub fn get1<A>(&self, prop1: &impl PropOrPath<A>) -> Option<A>
    where
        A: Model,
    {
        prop1.extract(&self.0)
    }

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
