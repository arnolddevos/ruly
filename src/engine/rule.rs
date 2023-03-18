use super::{
    property::{Model, Property},
    table::{Table, View},
    variant::{JoinResult, Variant},
};

pub struct Rule<M, F> {
    prop: Property<M>,
    func: F,
}

pub fn rule<M, F>(prop: Property<M>, func: F) -> Box<dyn Propagator>
where
    M: Model + 'static,
    F: Fn(View) -> Option<M::Repr> + 'static,
    Variant: From<M::Repr>,
{
    Box::new(Rule { prop, func })
}

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

pub fn one_shot_stable(table: &mut Table, rules: &Rules) -> usize {
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

pub fn recursive_stable(table: &mut Table, rules: &Rules) {
    loop {
        if one_shot_stable(table, rules) == 0 {
            break;
        }
    }
}

pub fn recursive(table: &mut Table, rules: &Rules) {
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
