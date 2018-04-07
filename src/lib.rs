extern crate chrono;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate serde_json;

use regex::Regex;
pub use chrono::{Duration, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt;

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
        let delta = Measurement::from(post.signed_duration_since(pre));
        (result,  delta)
    }}
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Measurement(chrono::Duration);

impl Measurement {
    pub fn zero() -> Self { Measurement(chrono::Duration::zero()) }
}

impl From<Measurement> for chrono::Duration {
    fn from(m: Measurement) -> chrono::Duration { m.0 }
}

impl From<chrono::Duration> for Measurement {
    fn from(d: chrono::Duration) -> Measurement { Measurement(d) }
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

impl Serialize for Measurement {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let remains: chrono::Duration = self.0;

        let num_days = remains.num_days();
        let remains = remains - chrono::Duration::days(num_days);
        let num_hours = remains.num_hours();
        let remains = remains - chrono::Duration::hours(num_hours);
        let num_minutes = remains.num_minutes();
        let remains = remains - chrono::Duration::minutes(num_minutes);
        let num_seconds = remains.num_seconds();
        let _ = remains - chrono::Duration::seconds(num_seconds);

        #[inline(always)]
        fn add(value: i64, marker: &str, buffer: &mut String) {
            buffer.push_str(&format!("{}", value));
            buffer.push_str(marker);
        };

        let mut buffer = String::from("P");
        add(num_days,    "D", &mut buffer);
        buffer.push_str("T");
        add(num_hours,   "H", &mut buffer);
        add(num_minutes, "M", &mut buffer);
        add(num_seconds, "S", &mut buffer);
        s.serialize_str(&buffer)
    }
}


struct MeasurementVisitor;
impl MeasurementVisitor {
    fn strip_prefix<'s, E>(expected: &str, string: &'s str)
                           -> Result<&'s str, E> where E: serde::de::Error {
        if !string.starts_with(expected) {
            let msg = format!("Invalid Measurement string: {}", string);
            return Err(serde::de::Error::custom(msg))
        }
        Ok(&string[1 ..]) // Strip the expected prefix
    }

    fn parse_part<E>(mut measurement: Measurement, string: &str)
                     -> Result<(Measurement, &str), E>
    where E: serde::de::Error {
        lazy_static! { static ref NUM: Regex = Regex::new(r"^(\d+)").unwrap(); }

        let string = match NUM.find(string) {
            None => string,
            Some(m) => {
                let num_idx = m.end();
                let num_str = &string[.. num_idx];
                let num = num_str.parse::<i64>().map_err(|parse_err| {
                    serde::de::Error::custom(parse_err.description())
                })?;

                measurement = match &string[num_idx .. num_idx + 1] {
                    "W" => Measurement(chrono::Duration::weeks(num)),
                    "D" => Measurement(chrono::Duration::days(num)),
                    "H" => Measurement(chrono::Duration::hours(num)),
                    "M" => Measurement(chrono::Duration::minutes(num)),
                    "S" => Measurement(chrono::Duration::seconds(num)),
                    unit => {
                        let msg = format!("Invalid date/time unit: {}", unit);
                        return Err(serde::de::Error::custom(msg))
                    },
                };

                &string[num_idx + 1 ..]
            },
        };

        Ok((measurement, string))
    }
}

impl<'de> serde::de::Visitor<'de> for MeasurementVisitor {
    type Value = Measurement;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Measurement that is within range")
    }

    fn visit_str<E>(self, string: &str)
                    -> Result<Measurement, E> where E: serde::de::Error {
        let mut string: &str = MeasurementVisitor::strip_prefix("P", string)?;
        let mut measurement = Measurement::zero();
        while !string.is_empty() && !string.starts_with("T") {
            let (d, s) = MeasurementVisitor::parse_part(measurement, string)?;
            measurement = d;
            string = s;
        }

        let mut string: &str = MeasurementVisitor::strip_prefix("T", string)?;
        while !string.is_empty() {
            let (d, s) = MeasurementVisitor::parse_part(measurement, string)?;
            measurement = d;
            string = s;
        }

        Ok(measurement)
    }

    fn visit_string<E>(self, string: String)
                       -> Result<Measurement, E> where E: serde::de::Error {
        self.visit_str(&string)
    }
}

impl<'de> Deserialize<'de> for Measurement {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(MeasurementVisitor)
    }
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
            .expect("failed to serialize to JSON");
        let deserialized = serde_json::from_str(&json_string)
            .expect("failed to deserialize from JSON");;
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
