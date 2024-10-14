use crate::{
    table::{Ident, IdentPath, Table},
    variant::{Error, Lattice, Variant},
};

/// The monomorphic view of a rule used in the evaluators.
pub trait Propagator {
    fn target(&self) -> &Ident;
    fn dependencies(&self) -> Vec<&IdentPath>;
    fn fire(&self, state: &Table) -> Option<Variant>;
}

/// A corpus of rules
pub type Propagators = Vec<Box<dyn Propagator>>;

/// A general, untyped `Propagator` of any arity implemented by a function.
pub struct PropagatorFunc<F> {
    target: Ident,
    dependencies: Vec<IdentPath>,
    func: F,
}

impl<F> PropagatorFunc<F>
where
    F: Fn(&[Option<&Variant>]) -> Option<Variant> + 'static,
{
    /// Create a general `Propagator`
    pub fn new(
        target: Ident,
        deps: impl IntoIterator<Item = IdentPath>,
        func: F,
    ) -> Box<dyn Propagator> {
        let dependencies = deps.into_iter().collect();
        Box::new(PropagatorFunc {
            target,
            dependencies,
            func,
        })
    }
}

impl<F> Propagator for PropagatorFunc<F>
where
    F: Fn(&[Option<&Variant>]) -> Option<Variant> + 'static,
{
    fn target(&self) -> &Ident {
        &self.target
    }

    fn dependencies(&self) -> Vec<&IdentPath> {
        self.dependencies.iter().collect()
    }

    fn fire(&self, state: &Table) -> Option<Variant> {
        let input: Vec<Option<&Variant>> = self
            .dependencies
            .iter()
            .map(|p| state.get_path(p))
            .collect();
        (self.func)(&input)
    }
}

/// Evaluate rules in priority order. The first result for a given property stands.  
/// Each rule is evaluated at most once and no joins are performed.  
pub fn evaluate_priority_once(table: &mut Table, rules: &Propagators) -> usize {
    let mut changes = 0;
    for rule in rules {
        if table.get(rule.target()).is_none() {
            if let Some(b) = rule.fire(&table) {
                table.insert(rule.target().clone(), b);
                changes += 1;
            }
        }
    }
    changes
}

/// This recursively joins results until a fixed point is reached.  
/// Rule order is unimportant.
/// The strategy is called naive evaluation in the lit.  
/// Naive is the best we can do without using the rule dependency information.
/// Rules or combinations of rules that diverge are caught by an iteration limit.
pub fn evaluate_naive(
    table: &mut Table,
    rules: &Propagators,
    limit: usize,
) -> Result<usize, Error> {
    let mut iteration = 0;
    loop {
        iteration += 1;
        if iteration > limit {
            break Err(Error::Detail(format!("exhausted {limit} iterations ")));
        }

        let mut changes = 0;

        for rule in rules {
            if let Some(value) = rule.fire(&table) {
                if let Some(extant) = table.get_mut(rule.target()) {
                    if extant.join_update(value) {
                        changes = 1;
                    }
                } else {
                    table.insert(rule.target().clone(), value);
                    changes += 1;
                }
            }
        }

        if changes == 0 {
            break Ok(iteration);
        }
    }
}
