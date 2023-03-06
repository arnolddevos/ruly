use super::variant::{Keyword, Variant};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

pub trait Property {
    type Value;
    fn name(&self) -> &str;
}

pub struct Table(HashMap<String, Variant>);

impl Table {
    pub fn insert_pv<P, V>(&mut self, prop: &P, value: V)
    where
        P: Property<Value = V>,
        V: Into<Variant>,
    {
        self.insert(prop.name(), value.into());
    }

    pub fn insert(&mut self, name: &str, value: Variant) -> Option<Variant> {
        self.0.insert(name.to_string(), value)
    }

    pub fn get(&self, name: &str) -> Option<Variant> {
        self.0.get(name).cloned()
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn get1<P>(&self, p: P) -> Option<P::Value>
    where
        P: Property,
        P::Value: TryFrom<Variant>,
    {
        self.0
            .get(p.name())
            .and_then(|value| value.clone().try_into().ok())
    }

    pub fn get2<P, Q>(&self, p: P, q: Q) -> Option<(P::Value, Q::Value)>
    where
        P: Property,
        Q: Property,
        P::Value: TryFrom<Variant>,
        Q::Value: TryFrom<Variant>,
    {
        Some((self.get1(p)?, self.get1(q)?))
    }

    pub fn get3<P, Q, R>(&self, p: P, q: Q, r: R) -> Option<(P::Value, Q::Value, R::Value)>
    where
        P: Property,
        Q: Property,
        R: Property,
        P::Value: TryFrom<Variant>,
        Q::Value: TryFrom<Variant>,
        R::Value: TryFrom<Variant>,
    {
        Some((self.get1(p)?, self.get1(q)?, self.get1(r)?))
    }
}

pub struct StringProp(&'static str);

impl Property for StringProp {
    type Value = String;
    fn name(&self) -> &str {
        self.0
    }
}

pub struct IntProp(&'static str);

impl Property for IntProp {
    type Value = i64;
    fn name(&self) -> &str {
        self.0
    }
}

pub struct FloatProp(&'static str);

impl Property for FloatProp {
    type Value = f64;
    fn name(&self) -> &str {
        self.0
    }
}

pub struct DateProp(&'static str);

impl Property for DateProp {
    type Value = NaiveDate;
    fn name(&self) -> &str {
        self.0
    }
}

pub struct InstantProp(&'static str);

impl Property for InstantProp {
    type Value = DateTime<Utc>;
    fn name(&self) -> &str {
        self.0
    }
}

pub struct KeyWordProp<T>(&'static str, T);

impl<T> Property for KeyWordProp<T> {
    type Value = Keyword<T>;
    fn name(&self) -> &str {
        self.0
    }
}
