//! Integrate some signal.

#[macro_use(lift)]
extern crate carboxyl;
extern crate carboxyl_time;
extern crate time;

use carboxyl_time::{ every, now, integrate };
use time::{ Tm, Duration, Timespec };

fn float(time: Tm) -> f64 {
    let Timespec { sec, nsec } = time.to_timespec();
    sec as f64 + nsec as f64 * 1e-9
}

fn main() {
    let ns = lift!(float, &now());
    let dt = Duration::milliseconds(20);
    let sig = integrate(&ns, 0.0, dt, |b, a, dt| b + a * dt.num_milliseconds() as f64 / 1e3);
    for dt in sig.snapshot(&every(dt), |x, _| x).events() {
        println!("tick {:e}", dt);
    }
}
