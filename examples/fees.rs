use ruly::{
    propagator::Rules,
    property::{prop, Property},
    quantity::{date::Date, money::AUD, Value},
    rule::infer,
};

fn main() {
    fees();
}

static _PATIENT: Property<String> = prop("patient");
static _SERVICE_DATE: Property<Value<Date>> = prop("service_date");
static ASSIST_51300: Property<Value<AUD>> = prop("assist_51300");
static ASSIST_51303: Property<f64> = prop("assist_51303");
static ITEM: Property<i64> = prop("item");
static SURGEON_MBS_FEE: Property<Value<AUD>> = prop("surgeon_mbs_fee");
static ASSIST_NOGAP_FEE: Property<Value<AUD>> = prop("assist_nogap_fee");

fn fees() -> Rules {
    Vec::from([
        infer(&ASSIST_NOGAP_FEE)
            .from(&ITEM)
            .from(&ASSIST_51300)
            .rule(|input| match input {
                (51300, fee) => Some(fee),
                _ => None,
            }),
        infer(&ASSIST_NOGAP_FEE)
            .from(&ITEM)
            .from(&ASSIST_51303)
            .from(&SURGEON_MBS_FEE)
            .rule(|input| match input {
                (51303, r, s) => Some(s.scale(r * 0.2)),
                _ => None,
            }),
    ])
}
