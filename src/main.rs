use creative_coding_project::app::run_app;
use creative_coding_project::dsp::adsr::*;
use creative_coding_project::dsp::ramp::*;
use creative_coding_project::prelude::*;

fn main() {
    let sample_length = sample_length();
    let mut env = AdsrEnvelope::new();
    env.set_parameters(
        sample_length * 10000.0,
        sample_length * 10000.0,
        0.5,
        sample_length * 10000.0,
    );

    println!("{env:#?}");

    for i in 1..=40 {
        env.next(true);
        println!("{i}: {:?}", env.get_stage());
    }

    for i in 1..=20 {
        env.next(false);
        println!("{}: {:?}", i + 40, env.get_stage());
    }

    /* let mut ramp = Ramp::new(1.0, sample_length * 10.0);

    println!("{ramp:#?}");

    for i in 1..=10 {
        let val = ramp.next();
        println!("{i}: for {val}, is ramp active? {}", ramp.is_active());
        // println!("{i}: {ramp:#?}");
    } */

    // run_app();
}
