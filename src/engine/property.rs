use std::{error, fmt::Display, marker::PhantomData, str::FromStr};

pub struct Property<M> {
    pub name: &'static str,
    pub model: M,
}

#[derive(Debug)]
pub struct Error(String);

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("error parsing: ")?;
        f.write_str(&self.0)
    }
}

pub fn property<M: Model>(name: &'static str) -> Property<M> {
    Property {
        name,
        model: M::UNIT,
    }
}

pub trait Model {
    type Repr;
    const UNIT: Self;
    fn parse(raw: &str) -> Result<Self::Repr, Error>;
    fn format(repr: &Self::Repr) -> String;
}

pub struct Standard<T>(PhantomData<T>);

impl<T> Model for Standard<T>
where
    T: FromStr + ToString,
    T::Err: Display,
{
    type Repr = T;
    const UNIT: Self = Self(PhantomData);

    fn format(repr: &Self::Repr) -> String {
        repr.to_string()
    }

    fn parse(raw: &str) -> Result<Self::Repr, Error> {
        raw.parse()
            .or_else(|err| Err(Error(format!("{err}: {raw}"))))
    }
}
