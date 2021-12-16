use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("I have been intruduced -");
    sleep(Duration::from_secs(1));
    println!("- I'm not joking!");
}

#[link_section = ".init_array"]
pub static INITIALIZE: fn() = main;
