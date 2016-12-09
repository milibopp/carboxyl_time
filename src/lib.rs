#[macro_use(lift)]
extern crate carboxyl;
extern crate time;
#[macro_use(lazy_static)]
extern crate lazy_static;

use std::thread;
use carboxyl::{ Stream, Signal, Sink };
use time::{ Duration, Tm };


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


/// A stream that regularly fires an event.
///
/// The stream events describe the duration passed since the last occurence of
/// an event (or for the first one the duration since the creation of the
/// stream).
///
/// This function tries to regularly fire an event after the specified interval
/// has passed. Of course, it might take longer than the specified time to
/// process the event. In that case as many events as necessary will be skipped
/// to keep up the pace.
pub fn every(interval: Duration) -> Stream<Duration> {
    // Setup sink and stream
    let sink = Sink::new();
    let stream = sink.stream();
    // Spawn a thread
    thread::spawn({
        let mut last = time::now();
        move || loop {
            let togo = last + interval - time::now();
            if togo < Duration::zero() {
                let passed = interval * (1 + togo.num_milliseconds() / interval.num_milliseconds()) as i32;
                sink.send(passed);
                last = last + passed;
            } else {
                thread::sleep_ms(togo.num_milliseconds() as u32);
            }
        }
    });
    stream
}


/// Integrate a signal over time.
pub fn integrate<A, B, F>(a: &Signal<A>, initial: B, dt: Duration, f: F) -> Signal<B>
    where A: Clone + Send + Sync + 'static,
          B: Clone + Send + Sync + 'static,
          F: Fn(B, A, Duration) -> B + Send + Sync + 'static,
{
    a.snapshot(&every(dt), |a, dt| (a, dt))
        .fold(initial, move |b, (a, dt)| f(b, a, dt))
}


#[cfg(test)]
mod test {
    use std::thread;
    use std::fmt::Debug;
    use carboxyl::{ Signal, Sink };
    use time::{ Duration, Tm };

    use super::{ now, now_utc, every };


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

    #[test]
    fn every_timing() {
        let dt = Duration::microseconds(10000);
        let ms = now().snapshot(&every(dt), |t, _| t);
        let mut events = ms.events();
        // Throw away the first one, as timings will be somewhat off
        events.next();
        // Compare the next two allowing for some 5%-tolerance
        let t1 = events.next().unwrap();
        let t2 = events.next().unwrap();
        let delta = t2 - t1;
        assert!(delta < Duration::microseconds(10500));
        assert!(delta > Duration::microseconds(9500));
    }
}
