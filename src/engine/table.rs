use super::{
    property::{Model, Property},
    variant::Variant,
};
use std::collections::HashMap;

pub struct Table(HashMap<String, Variant>);

impl Table {
    pub fn get(&self, name: &str) -> Variant {
        self.0.get(name).cloned().unwrap_or(Variant::Nothing)
    }

    pub fn insert(&mut self, name: &str, value: Variant) -> Variant {
        self.0
            .insert(name.to_string(), value)
            .unwrap_or(Variant::Nothing)
    }

    pub fn view<'a>(&'a self) -> View<'a> {
        View(&self)
    }
}

pub struct View<'a>(&'a Table);

impl<'a> View<'a> {
    pub fn get1<M1>(&self, prop1: Property<M1>) -> Option<M1::Repr>
    where
        M1: Model,
        M1::Repr: TryFrom<Variant>,
    {
        self.0.get(prop1.name).try_into().ok()
    }

    pub fn get2<M1, M2>(
        &self,
        prop1: Property<M1>,
        prop2: Property<M2>,
    ) -> Option<(M1::Repr, M2::Repr)>
    where
        M1: Model,
        M1::Repr: TryFrom<Variant>,
        M2: Model,
        M2::Repr: TryFrom<Variant>,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?))
    }

    pub fn get3<M1, M2, M3>(
        &self,
        prop1: Property<M1>,
        prop2: Property<M2>,
        prop3: Property<M3>,
    ) -> Option<(M1::Repr, M2::Repr, M3::Repr)>
    where
        M1: Model,
        M1::Repr: TryFrom<Variant>,
        M2: Model,
        M2::Repr: TryFrom<Variant>,
        M3: Model,
        M3::Repr: TryFrom<Variant>,
    {
        Some((self.get1(prop1)?, self.get1(prop2)?, self.get1(prop3)?))
    }
}
