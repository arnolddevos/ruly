use super::{
    property::{Model, Property},
    table::{Table, View},
    variant::{JoinResult, Variant},
};

/// A typed rule for a given property.
pub struct Rule<M, F> {
    prop: Property<M>,
    func: F,
}

/// Create a typed rule and convert return it as a propagator.
pub fn rule<M, F>(prop: Property<M>, func: F) -> Box<dyn Propagator>
where
    M: Model + 'static,
    F: Fn(View) -> Option<M::Repr> + 'static,
    Variant: From<M::Repr>,
{
    Box::new(Rule { prop, func })
}

/// The monomorphic view of a rule used in the evaluators.
pub trait Propagator {
    fn property_name(&self) -> &str;
    fn fire(&self, state: View) -> Variant;
}

impl<M, F> Propagator for Rule<M, F>
where
    M: Model,
    F: Fn(View) -> Option<M::Repr>,
    Variant: From<M::Repr>,
{
    fn property_name(&self) -> &str {
        &self.prop.name
    }
    fn fire(&self, state: View) -> Variant {
        let x = (self.func)(state);
        x.into()
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
        if a == Variant::Nothing {
            let b = rule.fire(table.view());
            if b != Variant::Nothing {
                table.insert(rule.property_name(), b);
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
            if value != Variant::Nothing {
                let prev = table.get(rule.property_name());
                match prev.join(value) {
                    JoinResult::Left(_) => (),
                    JoinResult::Right(value) | JoinResult::Greater(value) => {
                        table.insert(rule.property_name(), value);
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
