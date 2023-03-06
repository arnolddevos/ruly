use super::{
    table::{Property, Table},
    variant::Variant,
};

pub struct Rule<P, F>(P, F);

pub fn rule<P, F>(prop: P, func: F) -> Box<dyn Propagator>
where
    P: Property + 'static,
    F: Fn(&Table) -> Option<P::Value> + 'static,
    P::Value: Into<Variant>,
{
    Box::new(Rule(prop, func))
}

pub trait Propagator {
    fn property_name(&self) -> &str;
    fn fire(&self, state: &Table) -> Option<Variant>;
}

impl<P, F> Propagator for Rule<P, F>
where
    P: Property,
    F: Fn(&Table) -> Option<P::Value>,
    P::Value: Into<Variant>,
{
    fn property_name(&self) -> &str {
        &self.0.name()
    }
    fn fire(&self, state: &Table) -> Option<Variant> {
        (self.1)(state).map(|v| v.into())
    }
}

pub type Rules = Vec<Box<dyn Propagator>>;

pub fn one_shot_stable(table: &mut Table, rules: &Rules) -> usize {
    let mut usize = 0;
    for rule in rules {
        if !table.contains_key(rule.property_name()) {
            if let Some(value) = rule.fire(table) {
                table.insert(rule.property_name(), value);
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
            if let Some(value) = rule.fire(table) {
                if let Some(prev) = table.get(rule.property_name()) {
                    let value = prev.join(&value);
                    if prev != value {
                        table.insert(rule.property_name(), value);
                        usize += 1;
                    }
                } else {
                    table.insert(rule.property_name(), value);
                    usize += 1;
                }
            }
        }
        if usize == 0 {
            break;
        }
    }
}
