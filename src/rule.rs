use crate::variant::{Error, Ident, Lattice, Table, Variant};

/// The monomorphic view of a rule used in the evaluators.
pub trait Propagator {
    fn property_name(&self) -> &Ident;
    fn fire(&self, state: &Table) -> Option<Variant>;
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
