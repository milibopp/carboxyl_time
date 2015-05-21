#[macro_use(lift)]
extern crate carboxyl;
extern crate clock_ticks;
#[macro_use(lazy_static)]
extern crate lazy_static;

use carboxyl::Signal;
use clock_ticks::precise_time_ns;


/// Return a signal representing the system time.
pub fn time() -> Signal<u64> {
    // Define time signal statically, so there is only ever one unique time
    // signal. This is necessary to ensure that the system time sampled from
    // this is always the same within a single transaction.
    lazy_static! {
        static ref TIME_SIGNAL: Signal<u64> = lift!(precise_time_ns);
    }
    // Make a clone of the static signal.
    TIME_SIGNAL.clone()
}


#[cfg(test)]
mod test {
    use std::thread;
    use carboxyl::Sink;

    use super::time;


    #[test]
    fn time_samples_equal() {
        let sink = Sink::new();
        let cmp = time().snapshot(
            &time().snapshot(&sink.stream(), |t, ()| t),
            |t0, t1| (t0, t1)
        );
        let mut events = cmp.events();
        sink.send(());
        let (t0, t1) = events.next().unwrap();
        assert_eq!(t0, t1);
    }

    #[test]
    fn time_consistent_with_sleep_ms() {
        let t0 = time().sample();
        for n in 0..5 {
            thread::sleep_ms(n);
            let dt = time().sample() - t0;
            assert!(dt > n as u64 * 1_000_000);
        }
    }
}
