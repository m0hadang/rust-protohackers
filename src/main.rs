// extern crate smoke_test;
// use smoke_test::echo_server::*;

// extern crate prime_time;
// use prime_time::prime_time_server::*;

extern crate means_to_an_end;
use means_to_an_end::means_to_an_end_server::*;

static IP: &str = "0.0.0.0";
const PORT: u32 = 8000;

fn main() {
    run_server(IP, PORT);
}
