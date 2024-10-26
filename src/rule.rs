use crate::{
    propagator::Propagator,
    property::{Path, Property},
    table::{Ident, IdentPath, Table},
    variant::{Error, Variant},
};

/// A polymophic function implementing `Propagator`.
///
/// A `Rule` consists of a dependent `Property`, a dependency `Path` or a
/// tuple of dependency paths, and a function connecting these.
///
/// A `Rule` is constructed by an expression e.g. `infer(prop).from(path).rule(func)`.
/// The `infer` function constructs the head of the rule with a dependent `Property`.
/// The `from` method adds a dependency `Path` and may be chained to add more paths.
/// A function is passed to the `rule` method and a `Propagator` object is returned.
///
/// The rule function takes a single argument, either the dependency value or a
/// tuple of two or three dependency values.  The function is not invoked unless all dependency
/// values are available.  It returns an optional dependent value.   
#[derive(Debug)]
pub struct Rule<H, T, F> {
    output: H,
    input: T,
    func: F,
}

#[derive(Debug)]
struct FuncOptional<F>(F);

#[derive(Debug)]
struct FuncFallible<F>(F);

/// The head of a `Rule` that produces values of type `A` for the given `Property``.
pub fn infer<A>(prop: &Property<A>) -> Rule<Property<A>, (), ()> {
    Rule {
        output: prop.clone(),
        input: (),
        func: (),
    }
}

impl<A> Rule<Property<A>, (), ()> {
    /// Add the 1st dependency to a rule.  The dependency is a path of type `B`.
    pub fn from<B>(self, path: impl Into<Path<B>>) -> Rule<Property<A>, Path<B>, ()> {
        Rule {
            output: self.output,
            input: path.into(),
            func: (),
        }
    }
}

impl<A, B> Rule<Property<A>, Path<B>, ()> {
    /// Add the 2nd dependency to a rule.  The dependency is a path of type `C`.
    pub fn from<C>(self, path: impl Into<Path<C>>) -> Rule<Property<A>, (Path<B>, Path<C>), ()> {
        Rule {
            output: self.output,
            input: (self.input, path.into()),
            func: (),
        }
    }

    /// Add an optional function to complete a rule of arity 1.  Return a `Propagator` object.
    pub fn rule<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn(B) -> Option<A> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
    {
        Box::new(Rule {
            output: self.output,
            input: self.input,
            func: FuncOptional(func),
        })
    }

    /// Add a fallible function to complete a rule of arity 1.  Return a `Propagator` object.
    pub fn rule_fallible<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn(B) -> Result<Option<A>, Error> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
    {
        Box::new(Rule {
            output: self.output,
            input: self.input,
            func: FuncFallible(func),
        })
    }
}

impl<A, B, C> Rule<Property<A>, (Path<B>, Path<C>), ()> {
    /// Add the 3rd dependency to a rule.  The dependency is a path of type `D`.
    pub fn from<D>(
        self,
        path: impl Into<Path<D>>,
    ) -> Rule<Property<A>, (Path<B>, Path<C>, Path<D>), ()> {
        Rule {
            output: self.output,
            input: (self.input.0, self.input.1, path.into()),
            func: (),
        }
    }

    /// Add an optional function to complete a rule of arity 2.  Return a `Propagator` object.
    pub fn rule<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn((B, C)) -> Option<A> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
        C: TryFrom<Variant> + 'static,
    {
        Box::new(Rule {
            output: self.output,
            input: self.input,
            func: FuncOptional(func),
        })
    }
}

impl<A, B, C, D> Rule<Property<A>, (Path<B>, Path<C>, Path<D>), ()> {
    /// Add an optional function to complete a rule of arity 3.  Return a `Propagator` object.
    pub fn rule<F>(self, func: F) -> Box<dyn Propagator>
    where
        F: Fn((B, C, D)) -> Option<A> + 'static,
        A: Into<Variant> + 'static,
        B: TryFrom<Variant> + 'static,
        C: TryFrom<Variant> + 'static,
        D: TryFrom<Variant> + 'static,
    {
        Box::new(Rule {
            output: self.output,
            input: self.input,
            func: FuncOptional(func),
        })
    }
}

impl<A, B, F> Propagator for Rule<Property<A>, Path<B>, FuncOptional<F>>
where
    F: Fn(B) -> Option<A>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
{
    fn target(&self) -> &Ident {
        &self.output.name
    }

    fn dependencies(&self) -> Vec<&IdentPath> {
        Vec::from([])
    }

    fn fire(&self, state: &Table) -> Option<Variant> {
        Some((self.func.0)(self.input.query(state)?)?.into())
    }
}

impl<A, B, F> Propagator for Rule<Property<A>, Path<B>, FuncFallible<F>>
where
    F: Fn(B) -> Result<Option<A>, Error>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
{
    fn target(&self) -> &Ident {
        &self.output.name
    }

    fn dependencies(&self) -> Vec<&IdentPath> {
        Vec::from([self.input.ident_path()])
    }

    fn fire(&self, state: &Table) -> Option<Variant> {
        match (self.func.0)(self.input.query(state)?) {
            Ok(Some(x)) => Some(x.into()),
            Ok(None) => None,
            Err(e) => Some(Variant::Invalid(e)),
        }
    }
}

impl<A, B, C, F> Propagator for Rule<Property<A>, (Path<B>, Path<C>), FuncOptional<F>>
where
    F: Fn((B, C)) -> Option<A>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
    C: TryFrom<Variant>,
{
    fn target(&self) -> &Ident {
        &self.output.name
    }

    fn dependencies(&self) -> Vec<&IdentPath> {
        Vec::from([self.input.0.ident_path(), self.input.1.ident_path()])
    }

    fn fire(&self, state: &Table) -> Option<Variant> {
        Some((self.func.0)((self.input.0.query(state)?, self.input.1.query(state)?))?.into())
    }
}

impl<A, B, C, D, F> Propagator for Rule<Property<A>, (Path<B>, Path<C>, Path<D>), FuncOptional<F>>
where
    F: Fn((B, C, D)) -> Option<A>,
    A: Into<Variant>,
    B: TryFrom<Variant>,
    C: TryFrom<Variant>,
    D: TryFrom<Variant>,
{
    fn target(&self) -> &Ident {
        &self.output.name
    }

    fn dependencies(&self) -> Vec<&IdentPath> {
        Vec::from([
            self.input.0.ident_path(),
            self.input.1.ident_path(),
            self.input.2.ident_path(),
        ])
    }

    fn fire(&self, state: &Table) -> Option<Variant> {
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
