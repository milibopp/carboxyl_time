//! A stupid program that prints tick every second.

extern crate carboxyl_time;
extern crate time;

use carboxyl_time::every;
use time::Duration;

fn main() {
    for () in every(Duration::seconds(1)).events() {
        println!("tick");
    }
}
