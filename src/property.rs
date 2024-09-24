use super::variant::Variant;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{marker::PhantomData, rc::Rc, str::FromStr};

#[derive(PartialEq, Eq, Hash, Debug, Display, From, Clone, Serialize, Deserialize)]
pub struct Ident(Rc<str>);

#[derive(PartialEq, Eq, Hash, Debug)]
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

pub struct View<'a>(&'a Table);

impl<'a> View<'a> {
    pub fn get1<M1>(&self, prop1: &Property<M1>) -> Option<M1>
    where
        M1: Model,
    {
        self.0.get(&prop1.name).try_into().ok()
    }

    pub fn get2<M1, M2>(&self, prop1: &Property<M1>, prop2: &Property<M2>) -> Option<(M1, M2)>
    where
        M1: Model,
        M2: Model,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?))
    }

    pub fn get3<M1, M2, M3>(
        &self,
        prop1: &Property<M1>,
        prop2: &Property<M2>,
        prop3: &Property<M3>,
    ) -> Option<(M1, M2, M3)>
    where
        M1: Model,
        M2: Model,
        M3: Model,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?, self.get1(prop3)?))
    }
}
