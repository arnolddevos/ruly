use super::{
    property::{Model, Property},
    table::{Table, View},
    variant::Variant,
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
    let mut usize = 0;
    for rule in rules {
        let a = table.get(rule.property_name());
        if matches!(a, Variant::Nothing | Variant::Invalid(_)) {
            let b = rule.fire(table.view());
            if !matches!(b, Variant::Nothing) {
                table.insert(rule.property_name(), b);
                usize += 1;
            }
        }
    }
    usize
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
        let mut usize = 0;
        for rule in rules {
            let value = rule.fire(table.view());
            let prev = table.get(rule.property_name());
            let join = prev.join(value);
            let prev = table.insert(rule.property_name(), join.clone());
            if prev != join {
                usize += 1
            };
        }
        if usize == 0 {
            break;
        }
    }
}
