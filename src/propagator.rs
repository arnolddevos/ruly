use crate::{
    table::{Ident, IdentPath, Table},
    variant::{Error, Variant},
};

/// A `Propagator` generates a new value from the existing values in a `Table`.  
/// It declares which entries in the `Table` will influence its output via `dependencies`.  
/// It designates the entry which should be updated with its output value via `target`.
/// This trait is implemented by `PropagatorFunc` (monomorphic) and `Rule` (polymorphic).
pub trait Propagator {
    /// The `Ident` of table entry to update.
    fn target(&self) -> &Ident;
    /// The `IdentPath`s of the table entries that influence this propagator.
    fn dependencies(&self) -> Vec<&IdentPath>;
    /// Evaluate a new value based on the current values in the `Table`.
    fn fire(&self, state: &Table) -> Option<Variant>;
}

/// A corpus of propagators
pub type Propagators = Vec<Box<dyn Propagator>>;

/// A `Propagator` implemented by a function of a variable number of
/// `Variant`s optionally producing a `Variant`.
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
                table.join_entry(rule.target().clone(), b);
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
                if table.join_entry(rule.target().clone(), value) {
                    changes += 1
                }
            }
        }

        if changes == 0 {
            break Ok(iteration);
        }
    }
}
