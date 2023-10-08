use creative_coding_project::app::run_app;
use creative_coding_project::dsp::adsr::*;
use creative_coding_project::dsp::ramp::*;
use creative_coding_project::prelude::*;

fn main() {
    let sample_length = 
    let mut env = AdsrEnvelope::new();
    println!("{env:#?}");

    for i in 1..=10 {
        env.next(true);
        println!("{i}: {env:#?}");
    }

    // run_app();
}
