use super::{
    property::{Ident, Model, Property, Table, View},
    variant::{Error, JoinResult, Variant},
};

/// The monomorphic view of a rule used in the evaluators.
pub trait Propagator {
    fn property_name(&self) -> &Ident;
    fn fire(&self, state: View) -> Variant;
}

/// A typed rule for a given property.
pub struct Rule<A, F> {
    prop: Property<A>,
    func: F,
}

/// Create a typed rule and return it as a propagator.
pub fn rule<A, F>(prop: Property<A>, func: F) -> Box<dyn Propagator>
where
    A: Model + 'static,
    F: Fn(View) -> Option<A> + 'static,
{
    Box::new(Rule { prop, func })
}

impl<A, F> Propagator for Rule<A, F>
where
    A: Model,
    F: Fn(View) -> Option<A>,
{
    fn property_name(&self) -> &Ident {
        &self.prop.name
    }
    fn fire(&self, state: View) -> Variant {
        if let Some(x) = (self.func)(state) {
            x.into()
        } else {
            Variant::Nothing
        }
    }
}

/// A fallible, typed rule for a given property.
pub struct RuleFallible<A, F> {
    prop: Property<A>,
    func: F,
}

/// Create a typed rule and return it as a propagator.
pub fn rule_fallible<A, F>(prop: Property<A>, func: F) -> Box<dyn Propagator>
where
    A: Model + 'static,
    F: Fn(View) -> Result<Option<A>, Error> + 'static,
{
    Box::new(RuleFallible { prop, func })
}

impl<A, F> Propagator for RuleFallible<A, F>
where
    A: Model,
    F: Fn(View) -> Result<Option<A>, Error>,
{
    fn property_name(&self) -> &Ident {
        &self.prop.name
    }
    fn fire(&self, state: View) -> Variant {
        match (self.func)(state) {
            Ok(Some(x)) => x.into(),
            Ok(None) => Variant::Nothing,
            Err(e) => Variant::Invalid(e),
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
        let a = table.get(rule.property_name());
        if a.is_nothing() {
            let b = rule.fire(table.view());
            if !b.is_nothing() {
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
pub fn evaluate_naive(table: &mut Table, rules: &Rules) {
    loop {
        let mut changes = 0;
        for rule in rules {
            let value = rule.fire(table.view());
            if !value.is_nothing() {
                let prev = table.get(rule.property_name());
                match prev.join(value) {
                    JoinResult::Left(_) => (),
                    JoinResult::Right(value) | JoinResult::Greater(value) => {
                        table.insert(rule.property_name().clone(), value);
                        changes += 1;
                    }
                }
            }
        }
        if changes == 0 {
            break;
        }
    }
}
