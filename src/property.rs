use super::variant::Variant;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::empty;
use std::ops::Div;
use std::{marker::PhantomData, rc::Rc, str::FromStr};

#[derive(PartialEq, Eq, Hash, Debug, Display, From, Clone, Serialize, Deserialize)]
pub struct Ident(Rc<str>);

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Property<A> {
    pub name: Ident,
    marker: PhantomData<A>,
}

impl<A: Model> Property<A> {
    pub fn new(ident: impl Into<Ident>) -> Self {
        Self {
            name: ident.into(),
            marker: PhantomData,
        }
    }
}

pub trait Model: FromStr + ToString + TryFrom<Variant> + Into<Variant> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table(HashMap<Ident, Variant>);

impl Table {
    pub fn get(&self, name: &Ident) -> Variant {
        self.0.get(name).cloned().unwrap_or(Variant::Nothing)
    }

    pub fn insert(&mut self, name: Ident, value: Variant) -> Variant {
        self.0.insert(name, value).unwrap_or(Variant::Nothing)
    }

    pub fn view<'a>(&'a self) -> View<'a> {
        View(&self)
    }
}

#[derive(Debug, Clone)]
pub struct Path<A> {
    prefix: Vec<Ident>,
    subject: Property<A>,
}

impl<A> Div<Property<A>> for Property<Rc<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: Property<A>) -> Self::Output {
        Path::<A> {
            prefix: Vec::from([self.name]),
            subject: rhs,
        }
    }
}

impl<A> Div<Property<A>> for Path<Rc<Table>> {
    type Output = Path<A>;

    fn div(self, rhs: Property<A>) -> Self::Output {
        let mut prefix = self.prefix;
        prefix.push(self.subject.name);
        Path::<A> {
            prefix,
            subject: rhs,
        }
    }
}

pub trait PropOrPath<A> {
    fn prefix(&self) -> impl Iterator<Item = &Ident>;
    fn subject(&self) -> &Ident;
}

impl<A> PropOrPath<A> for Property<A> {
    fn prefix(&self) -> impl Iterator<Item = &Ident> {
        empty()
    }
    fn subject(&self) -> &Ident {
        &self.name
    }
}
impl<A> PropOrPath<A> for Path<A> {
    fn prefix(&self) -> impl Iterator<Item = &Ident> {
        self.prefix.iter()
    }
    fn subject(&self) -> &Ident {
        &self.subject.name
    }
}

pub struct View<'a>(&'a Table);

impl<'a> View<'a> {
    pub fn get1<M1>(&self, prop1: &impl PropOrPath<M1>) -> Option<M1>
    where
        M1: Model,
    {
        let root = self.0;
        let mut prefix = prop1.prefix();
        let v = if let Some(next) = prefix.next() {
            let mut step: Rc<Table> = root.get(next).try_into().ok()?;
            for next in prefix {
                step = step.get(next).try_into().ok()?;
            }
            step.get(prop1.subject())
        } else {
            root.get(prop1.subject())
        };
        v.try_into().ok()
    }

    pub fn get2<M1, M2>(
        &self,
        prop1: &impl PropOrPath<M1>,
        prop2: &impl PropOrPath<M2>,
    ) -> Option<(M1, M2)>
    where
        M1: Model,
        M2: Model,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?))
    }

    pub fn get3<M1, M2, M3>(
        &self,
        prop1: &impl PropOrPath<M1>,
        prop2: &impl PropOrPath<M2>,
        prop3: &impl PropOrPath<M3>,
    ) -> Option<(M1, M2, M3)>
    where
        M1: Model,
        M2: Model,
        M3: Model,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?, self.get1(prop3)?))
    }
}
