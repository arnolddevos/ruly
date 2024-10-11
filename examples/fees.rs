use ruly::{
    property::{prop, Property},
    quantity::{date::Date, money::AUD, Value},
    rule::{rule2, rule3, Propagator},
};

fn main() {}

static PATIENT: Property<String> = prop("patient");
static ASSIST_51300: Property<Value<AUD>> = prop("assist_51300");
static ASSIST_51303: Property<f64> = prop("assist_51303");
static ITEM: Property<i64> = prop("item");
static SERVICE_DATE: Property<Value<Date>> = prop("service_date");
static SURGEON_MBS_FEE: Property<Value<AUD>> = prop("surgeon_mbs_fee");
static ASSIST_NOGAP_FEE: Property<Value<AUD>> = prop("assist_nogap_fee");

fn fees() -> Vec<Box<dyn Propagator>> {
    Vec::from([
        rule2(
            &ITEM,
            &ASSIST_51300,
            &ASSIST_NOGAP_FEE,
            |input| match input {
                (51300, fee) => Some(fee),
                _ => None,
            },
        ),
        rule3(
            &ITEM,
            &ASSIST_51303,
            &SURGEON_MBS_FEE,
            &ASSIST_NOGAP_FEE,
            |input| match input {
                (51303, r, s) => Some(s.scale(r * 0.2)),
                _ => None,
            },
        ),
    ])
}
