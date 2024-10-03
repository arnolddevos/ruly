use crate::{
    property::{Model, Path, Property, Query},
    variant::{Error, Ident, Lattice, Table, Variant},
};

/// The monomorphic view of a rule used in the evaluators.
pub trait Propagator {
    fn property_name(&self) -> &Ident;
    fn fire(&self, state: &Table) -> Option<Variant>;
}

/// A typed rule for a given property.
pub struct Rule<A, F> {
    prop: Property<A>,
    func: F,
}

/// Create a typed rule and return it as a propagator.
pub fn rule<A, F>(prop: &Property<A>, func: F) -> Box<dyn Propagator>
where
    A: Model + 'static,
    F: Fn(&Table) -> Option<A> + 'static,
{
    Box::new(Rule {
        prop: prop.clone(),
        func,
    })
}

/// A typed rule with one explicit dependency.
pub fn rule1<A, B, F>(prop1: impl Into<Path<A>>, prop: &Property<B>, func: F) -> Box<dyn Propagator>
where
    A: Model + 'static,
    B: Model + 'static,
    F: Fn(A) -> Option<B> + 'static,
{
    let prop1 = prop1.into();
    rule(prop, move |state| func(prop1.query(state)?))
}

/// A typed rule with two explicit dependencies.
pub fn rule2<A, B, C, F>(
    prop1: impl Into<Path<A>>,
    prop2: impl Into<Path<B>>,
    prop: &Property<C>,
    func: F,
) -> Box<dyn Propagator>
where
    A: Model + 'static,
    B: Model + 'static,
    C: Model + 'static,
    F: Fn((A, B)) -> Option<C> + 'static,
{
    let prop1 = prop1.into();
    let prop2 = prop2.into();
    rule(prop, move |state| {
        func((prop1.query(state)?, prop2.query(state)?))
    })
}

/// A typed rule with three explicit dependencies.
pub fn rule3<A, B, C, D, F>(
    prop1: impl Into<Path<A>>,
    prop2: impl Into<Path<B>>,
    prop3: impl Into<Path<C>>,
    prop: &Property<D>,
    func: F,
) -> Box<dyn Propagator>
where
    A: Model + 'static,
    B: Model + 'static,
    C: Model + 'static,
    D: Model + 'static,
    F: Fn((A, B, C)) -> Option<D> + 'static,
{
    let prop1 = prop1.into();
    let prop2 = prop2.into();
    let prop3 = prop3.into();
    rule(prop, move |state| {
        func((
            prop1.query(state)?,
            prop2.query(state)?,
            prop3.query(state)?,
        ))
    })
}

impl<A, F> Propagator for Rule<A, F>
where
    A: Model,
    F: Fn(&Table) -> Option<A>,
{
    fn property_name(&self) -> &Ident {
        &self.prop.name
    }
    fn fire(&self, state: &Table) -> Option<Variant> {
        (self.func)(state).map(|x| x.into())
    }
}

/// A fallible, typed rule for a given property.
pub struct RuleFallible<A, F> {
    prop: Property<A>,
    func: F,
}

/// Create a typed rule and return it as a propagator.
pub fn rule_fallible<A, F>(prop: &Property<A>, func: F) -> Box<dyn Propagator>
where
    A: Model + 'static,
    F: Fn(&Table) -> Result<Option<A>, Error> + 'static,
{
    Box::new(RuleFallible {
        prop: prop.clone(),
        func,
    })
}

impl<A, F> Propagator for RuleFallible<A, F>
where
    A: Model,
    F: Fn(&Table) -> Result<Option<A>, Error>,
{
    fn property_name(&self) -> &Ident {
        &self.prop.name
    }
    fn fire(&self, state: &Table) -> Option<Variant> {
        match (self.func)(state) {
            Ok(Some(x)) => Some(x.into()),
            Ok(None) => None,
            Err(e) => Some(Variant::Invalid(e)),
        }
    }
}

pub type Rules = Vec<Box<dyn Propagator>>;

/// Evaluate rules in priority order. The first result for a given
/// property stands.  Each rule is evaluated at most once.
/// Variant::Nothing indicate no result and no joins are performed.  
pub fn evaluate_priority_once(table: &mut Table, rules: &Rules) -> usize {
    let mut changes = 0;
    for rule in rules {
        if table.get(rule.property_name()).is_none() {
            if let Some(b) = rule.fire(&table) {
                table.insert(rule.property_name().clone(), b);
                changes += 1;
            }
        }
    }
    changes
}

/// This recursively joins results until a fixed point is reached.  
/// Rule order is unimportant.
/// The strategy is called naive evaluation in the lit.  
/// Naive is the best we can do because the rules are opaque.
/// Rules or combinations of rules that diverge are caught by an iteration limit.
pub fn evaluate_naive(table: &mut Table, rules: &Rules, limit: usize) -> Result<usize, Error> {
    let mut iteration = 0;
    loop {
        iteration += 1;
        if iteration > limit {
            break Err(Error::Detail(format!("exhausted {limit} iterations ")));
        }

        let mut changes = 0;

        for rule in rules {
            if let Some(value) = rule.fire(&table) {
                if let Some(extant) = table.get_mut(rule.property_name()) {
                    if extant.join_update(value) {
                        changes = 1;
                    }
                } else {
                    table.insert(rule.property_name().clone(), value);
                    changes += 1;
                }
            }
        }

        if changes == 0 {
            break Ok(iteration);
        }
    }
}
