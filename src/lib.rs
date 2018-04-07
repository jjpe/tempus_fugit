extern crate chrono;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

mod error;
mod serialize;

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
/// then returns a (result, duration) tuple where:
/// - `result` is the result of executing the expression on its own
/// - `duration` is a chrono::Duration.
#[macro_export]
macro_rules! measure {
    ($e:expr) => {{
        let pre = $crate::Utc::now();
        let result = { $e };
        let post = $crate::Utc::now();
        let delta = $crate::Measurement::from(post.signed_duration_since(pre));
        (result,  delta)
    }}
}



#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Measurement(chrono::Duration);

impl Measurement {
    pub fn zero() -> Self { Measurement(chrono::Duration::zero()) }
}

impl ops::Add for Measurement {
    type Output = MeasureResult<Measurement>;

    fn add(self, rhs: Measurement) -> Self::Output {
        let duration = self.0.checked_add(&rhs.0).ok_or(MeasureErr::Overflow)?;
        Ok(Measurement::from(duration))
    }
}

impl ops::Sub for Measurement {
    type Output = MeasureResult<Measurement>;

    fn sub(self, rhs: Measurement) -> Self::Output {
        let duration = self.0.checked_sub(&rhs.0).ok_or(MeasureErr::Underflow)?;
        Ok(Measurement::from(duration))
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let duration = &self.0;
        if duration.num_nanoseconds().is_none() {
            return write!(f, "overflow");
        }

        match duration.num_nanoseconds().unwrap() as u64 {
            nanos if nanos < NS_PER_US => write!(f, "{} ns", nanos),
            nanos if nanos < NS_PER_MS => {
                let micros: u64 = nanos / NS_PER_US;
                let nanos: u64 = nanos % NS_PER_US;
                if nanos > 0 {
                    write!(f, "{} µs {} ns", micros, nanos)
                } else {
                    write!(f, "{} µs", micros)
                }
            },
            nanos if nanos < NS_PER_SEC => {
                let millis: u64 = nanos / NS_PER_MS;
                let micros: u64 = (nanos % NS_PER_MS) / NS_PER_US;
                if micros > 0 {
                    write!(f, "{} ms {} µs", millis, micros)
                } else {
                    write!(f, "{} ms", millis)
                }
            },
            nanos if nanos < NS_PER_MIN => {
                let secs: u64 = nanos / NS_PER_SEC;
                let millis: u64 = (nanos % NS_PER_SEC) / NS_PER_MS;
                if millis > 0 {
                    write!(f, "{} s {} ms", secs, millis)
                } else {
                    write!(f, "{} s", secs)
                }
            },
            nanos if nanos < NS_PER_HOUR => {
                let mins: u64 = nanos / NS_PER_MIN;
                let secs: u64 = (nanos % NS_PER_MIN) / NS_PER_SEC;
                if secs > 0 {
                    write!(f, "{} m {} s", mins, secs)
                } else {
                    write!(f, "{} m", mins)
                }
            },
            nanos => {
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
    fn from(d: chrono::Duration) -> Measurement { Measurement(d) }
}





#[cfg(test)]
mod tests {
    use Measurement;
    use chrono::Duration;
    use serde_json;

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
    fn serialize() {
        let (hours, mins) = (Duration::hours(3), Duration::minutes(3));
        let measurement = Measurement(hours.checked_add(&mins).unwrap());
        let json_string = serde_json::to_string(&measurement)
            .expect("failed to serialize");
        assert_eq!(json_string, "\"P0DT3H3M0S\"");
    }

    #[test]
    fn deserialize() {
        const JSON_STRING: &str = "\"P0DT3H3M0S\"";
        println!("JSON: {}", JSON_STRING);
        let deserialized = serde_json::from_str(&JSON_STRING)
            .expect("failed to deserialize");

        let (hours, mins) = (Duration::hours(3), Duration::minutes(3));
        let measurement = Measurement(hours.checked_add(&mins).unwrap());
        assert_eq!(measurement, deserialized,
                   "measurement ({}) != deserialized ({})",
                   measurement, deserialized);
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
