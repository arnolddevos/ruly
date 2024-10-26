use crate::{
    table::{Ident, IdentPath, Table},
    variant::Variant,
};
use std::{marker::PhantomData, ops::Div, rc::Rc};

/// A property confers a meaning to a value, its interpretation or what it represents.
/// A property has a name or `Ident` that identifies it uniquely.
/// Two properties that have the same name represent the same thing and are equal.
/// They should have the same type (but this is not enforced).
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

impl<A> Property<A> {
    pub fn new(ident: impl Into<Ident>) -> Self {
        Self {
            name: ident.into(),
            marker: PhantomData,
        }
    }
}

/// Construct a Property in a const context e.g.
/// `pub static FRED: Property<String> = prop("fred");`
pub const fn prop<A>(name: &'static str) -> Property<A> {
    Property {
        name: Ident::Intern(name),
        marker: PhantomData,
    }
}

/// A `Path` designates a property that may be in a nested `Table`.
/// Tables can be nested to any depth because a `Variant` value can be a `Table`.
/// A `Path` is constructed by connecting `Property`s with the `/` operator.
/// Also, a single `Property` can be lifted into a `Path`.
#[derive(Debug, Clone)]
pub struct Path<A> {
    inner: IdentPath,
    marker: PhantomData<A>,
}

impl<A> Path<A>
where
    A: TryFrom<Variant>,
{
    pub fn query(&self, table: &Table) -> Option<A> {
        table.get_path(&self.inner)?.clone().try_into().ok()
    }

    pub fn ident_path(&self) -> &IdentPath {
        &self.inner
    }
}

impl<A> Div<&Property<A>> for &Property<Rc<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: &Property<A>) -> Self::Output {
        Path::<A> {
            inner: IdentPath::new(self.name.clone()).append(rhs.name.clone()),
            marker: PhantomData,
        }
    }
}

impl<A> Div<&Property<A>> for Path<Rc<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: &Property<A>) -> Self::Output {
        Path::<A> {
            inner: self.inner.append(rhs.name.clone()),
            marker: PhantomData,
        }
    }
}

impl<A> Into<Path<A>> for &Property<A> {
    fn into(self) -> Path<A> {
        Path::<A> {
            inner: IdentPath::new(self.name.clone()),
            marker: PhantomData,
        }
    }
}
