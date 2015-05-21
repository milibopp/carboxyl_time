#[macro_use(lift)]
extern crate carboxyl;
extern crate time;
#[macro_use(lazy_static)]
extern crate lazy_static;

use carboxyl::Signal;
use time::Tm;


/// A macro for creating functions that return a handle to a static signal.
macro_rules! static_signal {
    ($t: ty, $f: expr) => { {
        // Define time signal statically, so there is only ever one unique time
        // signal. This is necessary to ensure that the system time sampled from
        // this is always the same within a single transaction.
        lazy_static! {
            static ref TIME_SIGNAL: Signal<$t> = lift!($f);
        }
        // Make a clone of the static signal.
        TIME_SIGNAL.clone()
    } }
}


/// A signal of the current local time.
pub fn now() -> Signal<Tm> {
    static_signal!(Tm, time::now)
}

/// A signal of the current UTC time.
pub fn now_utc() -> Signal<Tm> {
    static_signal!(Tm, time::now_utc)
}


#[cfg(test)]
mod test {
    use std::thread;
    use std::fmt::Debug;
    use carboxyl::{ Signal, Sink };
    use time::{ Duration, Tm };

    use super::{ now, now_utc };


    fn samples_equal<A, F: Fn() -> Signal<A>>(f: F)
        where A: PartialEq + Debug + Send + Clone + Sync + 'static,
    {
        let sink = Sink::new();
        let cmp = f().snapshot(
            &f().snapshot(&sink.stream(), |t, ()| t),
            |t0, t1| (t0, t1)
        );
        let mut events = cmp.events();
        sink.send(());
        let (t0, t1) = events.next().unwrap();
        assert_eq!(t0, t1);
    }

    #[test]
    fn now_samples_equal() {
        samples_equal(now);
    }

    #[test]
    fn now_utc_samples_equal() {
        samples_equal(now_utc);
    }

    fn consistent_with_sleep_ms<F: Fn() -> Signal<Tm>>(f: F) {
        let t0 = f().sample();
        for n in 0..5 {
            thread::sleep_ms(n);
            let dt = f().sample() - t0;
            assert!(dt > Duration::milliseconds(n as i64));
        }
    }

    #[test]
    fn now_consistent_with_sleep_ms() {
        consistent_with_sleep_ms(now);
    }

    #[test]
    fn now_utc_consistent_with_sleep_ms() {
        consistent_with_sleep_ms(now_utc);
    }
}
