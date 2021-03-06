/// A library to measure the wall-clock time of Rust expressions.

mod error;

// TODO: If / When possible, replace this with derived De/Serialize impls.
#[cfg(feature = "enable_serde")] mod serialize;

pub use error::{MeasureErr, MeasureResult};
pub use chrono::{Duration, Utc};
use std::fmt;
use std::ops;


const NS_PER_US: u64   = 1e3 as u64;
const NS_PER_MS: u64   = 1e6 as u64;
const NS_PER_SEC: u64  = 1e9 as u64;
const NS_PER_MIN: u64  = 60 * NS_PER_SEC;
const NS_PER_HOUR: u64 = 60 * NS_PER_MIN;


/// This macro measures the execution time of an expression,
/// then returns a `(result, measurement)` tuple where:
/// - `result` is the result of executing the expression on its own
/// - `measurement` has type `Measurement`.
#[macro_export]
macro_rules! measure {
    ($e:expr) => {{
        let pre = $crate::Utc::now();
        let result = { $e };
        let post = $crate::Utc::now();
        let delta = post.signed_duration_since(pre);
        (result,  $crate::Measurement::from(delta))
    }}
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Measurement(chrono::Duration);


impl Measurement {
    pub fn zero() -> Self { Self(chrono::Duration::zero()) }
}

impl Default for Measurement {
    fn default() -> Self { Self::zero() }
}

impl ops::Add for Measurement {
    type Output = MeasureResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let duration = self.0.checked_add(&rhs.0).ok_or(MeasureErr::Overflow)?;
        Ok(Self::from(duration))
    }
}

impl ops::Sub for Measurement {
    type Output = MeasureResult<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        let duration = self.0.checked_sub(&rhs.0).ok_or(MeasureErr::Underflow)?;
        Ok(Self::from(duration))
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.num_nanoseconds().map(|nanos| nanos as u64) {
            None => write!(f, "overflow"),
            Some(nanos) if nanos < NS_PER_US => write!(f, "{} ns", nanos),
            Some(nanos) if nanos < NS_PER_MS => {
                let micros: u64 = nanos / NS_PER_US;
                let nanos: u64 = nanos % NS_PER_US;
                if nanos > 0 {
                    write!(f, "{} µs {} ns", micros, nanos)
                } else {
                    write!(f, "{} µs", micros)
                }
            },
            Some(nanos) if nanos < NS_PER_SEC => {
                let millis: u64 = nanos / NS_PER_MS;
                let micros: u64 = (nanos % NS_PER_MS) / NS_PER_US;
                if micros > 0 {
                    write!(f, "{} ms {} µs", millis, micros)
                } else {
                    write!(f, "{} ms", millis)
                }
            },
            Some(nanos) if nanos < NS_PER_MIN => {
                let secs: u64 = nanos / NS_PER_SEC;
                let millis: u64 = (nanos % NS_PER_SEC) / NS_PER_MS;
                if millis > 0 {
                    write!(f, "{} s {} ms", secs, millis)
                } else {
                    write!(f, "{} s", secs)
                }
            },
            Some(nanos) if nanos < NS_PER_HOUR => {
                let mins: u64 = nanos / NS_PER_MIN;
                let secs: u64 = (nanos % NS_PER_MIN) / NS_PER_SEC;
                if secs > 0 {
                    write!(f, "{} m {} s", mins, secs)
                } else {
                    write!(f, "{} m", mins)
                }
            },
            Some(nanos) => {
                let hours: u64 = nanos / NS_PER_HOUR;
                let mins: u64 = (nanos % NS_PER_HOUR) / NS_PER_MIN;
                if mins > 0 {
                    write!(f, "{} h {} m", hours, mins)
                } else {
                    write!(f, "{} h", hours)
                }
            },
        }
    }
}


impl From<Measurement> for chrono::Duration {
    fn from(m: Measurement) -> chrono::Duration { m.0 }
}

impl From<chrono::Duration> for Measurement {
    fn from(d: chrono::Duration) -> Self { Self(d) }
}





#[cfg(test)]
mod tests {
    use crate::Measurement;
    use chrono::Duration;

    #[test]
    fn readme_md_example() {
        use std::fs::File;
        use std::io::Read;

        let (contents, measurement) = measure! {{
            let mut file = File::open("Cargo.lock")
                .expect("failed to open Cargo.lock");
            let mut contents = vec![];
            file.read_to_end(&mut contents)
                .expect("failed to read Cargo.lock");
            String::from_utf8(contents)
                .expect("failed to extract contents to String")
        }};

        println!("contents: {:?}", contents);
        println!("opening and reading Cargo.lock took {}", measurement);
    }

    #[test]
    fn format_hours_one_chunk() {
        let one_chunk = Measurement(Duration::hours(10));
        assert_eq!("10 h", format!("{}", one_chunk));
    }

    #[test]
    fn format_hours_two_chunks() {
        let (hours, mins) = (Duration::hours(3), Duration::minutes(3));
        let two_chunks = Measurement(hours.checked_add(&mins).unwrap());
        assert_eq!("3 h 3 m", format!("{}", two_chunks));
    }

    #[test]
    fn format_minutes_one_chunk() {
        let one_chunk = Measurement(Duration::minutes(10));
        assert_eq!("10 m", format!("{}", one_chunk));
    }

    #[test]
    fn format_minutes_two_chunks() {
        let (mins, secs) = (Duration::minutes(3), Duration::seconds(3));
        let two_chunks = Measurement(mins.checked_add(&secs).unwrap());
        assert_eq!("3 m 3 s", format!("{}", two_chunks));
    }

    #[test]
    fn format_seconds_one_chunk() {
        let one_chunk = Measurement(Duration::seconds(10));
        assert_eq!("10 s", format!("{}", one_chunk));
    }

    #[test]
    fn format_seconds_two_chunks() {
        let (secs, millis) = (Duration::seconds(3), Duration::milliseconds(3));
        let two_chunks = Measurement(secs.checked_add(&millis).unwrap());
        assert_eq!("3 s 3 ms", format!("{}", two_chunks));
    }

    #[test]
    fn format_milliseconds_one_chunk() {
        let one_chunk = Measurement(Duration::milliseconds(10));
        assert_eq!("10 ms", format!("{}", one_chunk));
    }

    #[test]
    fn format_milliseconds_two_chunks() {
        let millis = Duration::milliseconds(3);
        let micros = Duration::microseconds(3);
        let two_chunks = Measurement(millis.checked_add(&micros).unwrap());
        assert_eq!("3 ms 3 µs", format!("{}", two_chunks));
    }

    #[test]
    fn format_microseconds_one_chunk() {
        let one_chunk = Measurement(Duration::microseconds(10));
        assert_eq!("10 µs", format!("{}", one_chunk));
    }

    #[test]
    fn format_microseconds_two_chunks() {
        let micros = Duration::microseconds(3);
        let nanos = Duration::nanoseconds(3);
        let two_chunks = Measurement(micros.checked_add(&nanos).unwrap());
        assert_eq!("3 µs 3 ns", format!("{}", two_chunks));
    }

    #[test]
    fn format_nanoseconds_one_chunk() {
        let one_chunk = Measurement(Duration::nanoseconds(10));
        assert_eq!("10 ns", format!("{}", one_chunk));
    }
}
