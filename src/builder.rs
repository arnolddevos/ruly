use crate::{
    property::{Path, Property, Query},
    rule::Propagator,
    variant::{Error, Variant},
};

#[derive(Debug)]
pub struct RuleInfo<H, T, F> {
    output: H,
    input: T,
    func: F,
}

#[derive(Debug)]
pub struct FuncOptional<F>(F);

#[derive(Debug)]
pub struct FuncFallible<F>(F);

pub fn infer<A>(prop: &Property<A>) -> RuleInfo<Property<A>, (), ()> {
    RuleInfo {
        output: prop.clone(),
        input: (),
        func: (),
    }
}

impl<A> RuleInfo<Property<A>, (), ()> {
    pub fn from<B>(self, path: impl Into<Path<B>>) -> RuleInfo<Property<A>, Path<B>, ()> {
        RuleInfo {
            output: self.output,
            input: path.into(),
            func: (),
        }
    }
}

impl<A, B> RuleInfo<Property<A>, Path<B>, ()> {
    pub fn from<C>(
        self,
        path: impl Into<Path<C>>,
    ) -> RuleInfo<Property<A>, (Path<B>, Path<C>), ()> {
        RuleInfo {
            output: self.output,
            input: (self.input, path.into()),
            func: (),
        }
    }

    pub fn rule<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn(B) -> Option<A> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
    {
        Box::new(RuleInfo {
            output: self.output,
            input: self.input,
            func: FuncOptional(func),
        })
    }

    pub fn rule_fallible<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn(B) -> Result<Option<A>, Error> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
    {
        Box::new(RuleInfo {
            output: self.output,
            input: self.input,
            func: FuncFallible(func),
        })
    }
}

impl<A, B, C> RuleInfo<Property<A>, (Path<B>, Path<C>), ()> {
    pub fn from<D>(
        self,
        path: impl Into<Path<D>>,
    ) -> RuleInfo<Property<A>, (Path<B>, Path<C>, Path<D>), ()> {
        RuleInfo {
            output: self.output,
            input: (self.input.0, self.input.1, path.into()),
            func: (),
        }
    }

    pub fn rule<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn((B, C)) -> Option<A> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
        C: TryFrom<Variant> + 'static,
    {
        Box::new(RuleInfo {
            output: self.output,
            input: self.input,
            func: FuncOptional(func),
        })
    }
}

impl<A, B, C, D> RuleInfo<Property<A>, (Path<B>, Path<C>, Path<D>), ()> {
    pub fn rule<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn((B, C, D)) -> Option<A> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
        C: TryFrom<Variant> + 'static,
        D: TryFrom<Variant> + 'static,
    {
        Box::new(RuleInfo {
            output: self.output,
            input: self.input,
            func: FuncOptional(func),
        })
    }
}

impl<A, B, F> Propagator for RuleInfo<Property<A>, Path<B>, FuncOptional<F>>
where
    F: Fn(B) -> Option<A>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
{
    fn property_name(&self) -> &crate::variant::Ident {
        &self.output.name
    }

    fn fire(&self, state: &crate::variant::Table) -> Option<crate::variant::Variant> {
        Some((self.func.0)(self.input.query(state)?)?.into())
    }
}

impl<A, B, F> Propagator for RuleInfo<Property<A>, Path<B>, FuncFallible<F>>
where
    F: Fn(B) -> Result<Option<A>, Error>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
{
    fn property_name(&self) -> &crate::variant::Ident {
        &self.output.name
    }

    fn fire(&self, state: &crate::variant::Table) -> Option<crate::variant::Variant> {
        match (self.func.0)(self.input.query(state)?) {
            Ok(Some(x)) => Some(x.into()),
            Ok(None) => None,
            Err(e) => Some(Variant::Invalid(e)),
        }
    }
}

impl<A, B, C, F> Propagator for RuleInfo<Property<A>, (Path<B>, Path<C>), FuncOptional<F>>
where
    F: Fn((B, C)) -> Option<A>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
    C: TryFrom<Variant>,
{
    fn property_name(&self) -> &crate::variant::Ident {
        &self.output.name
    }

    fn fire(&self, state: &crate::variant::Table) -> Option<crate::variant::Variant> {
        Some((self.func.0)((self.input.0.query(state)?, self.input.1.query(state)?))?.into())
    }
}

impl<A, B, C, D, F> Propagator
    for RuleInfo<Property<A>, (Path<B>, Path<C>, Path<D>), FuncOptional<F>>
where
    F: Fn((B, C, D)) -> Option<A>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
    C: TryFrom<Variant>,
    D: TryFrom<Variant>,
{
    fn property_name(&self) -> &crate::variant::Ident {
        &self.output.name
    }

    fn fire(&self, state: &crate::variant::Table) -> Option<crate::variant::Variant> {
        Some(
            (self.func.0)((
                self.input.0.query(state)?,
                self.input.1.query(state)?,
                self.input.2.query(state)?,
            ))?
            .into(),
        )
    }
}
