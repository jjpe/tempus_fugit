extern crate chrono;

pub use chrono::{Duration, Utc};

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
        let delta: $crate::Duration = post.signed_duration_since(pre);
        (result,  delta)
    }}
}


/// A trait to display duration outputs of the
/// `measurement!` macro in a human-readable way.
pub trait MeasureDisplay {
    fn human_readable(&self) -> String;
}

impl MeasureDisplay for chrono::Duration {
    fn human_readable(&self) -> String {
        if self.num_nanoseconds().is_none() {
            return String::from("overflow");
        }

        const NS_PER_US: u64   = 1e3 as u64;
        const NS_PER_MS: u64   = 1e6 as u64;
        const NS_PER_SEC: u64  = 1e9 as u64;
        const NS_PER_MIN: u64  = 60 * NS_PER_SEC;
        const NS_PER_HOUR: u64 = 60 * NS_PER_MIN;

        match self.num_nanoseconds().unwrap() as u64 {
            nanos if nanos < NS_PER_US => format!("{} ns", nanos),
            nanos if nanos < NS_PER_MS => {
                let micros: u64 = nanos / NS_PER_US;
                let nanos: u64 = nanos % NS_PER_US;
                if nanos > 0 {
                    format!("{} µs {} ns", micros, nanos)
                } else {
                    format!("{} µs", micros)
                }
            },
            nanos if nanos < NS_PER_SEC => {
                let millis: u64 = nanos / NS_PER_MS;
                let micros: u64 = (nanos % NS_PER_MS) / NS_PER_US;
                if micros > 0 {
                    format!("{} ms {} µs", millis, micros)
                } else {
                    format!("{} ms", millis)
                }
            },
            nanos if nanos < NS_PER_MIN => {
                let secs: u64 = nanos / NS_PER_SEC;
                let millis: u64 = (nanos % NS_PER_SEC) / NS_PER_MS;
                if millis > 0 {
                    format!("{} s {} ms", secs, millis)
                } else {
                    format!("{} s", secs)
                }
            },
            nanos if nanos < NS_PER_HOUR => {
                let mins: u64 = nanos / NS_PER_MIN;
                let secs: u64 = (nanos % NS_PER_MIN) / NS_PER_SEC;
                if secs > 0 {
                    format!("{} m {} s", mins, secs)
                } else {
                    format!("{} m", mins)
                }
            },
            nanos => {
                let hours: u64 = nanos / NS_PER_HOUR;
                let mins: u64 = (nanos % NS_PER_HOUR) / NS_PER_MIN;
                if mins > 0 {
                    format!("{} h {} m", hours, mins)
                } else {
                    format!("{} h", hours)
                }
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use chrono::Duration;
    use ::MeasureDisplay;

    #[test]
    fn human_readable_hours() {
        let foo = Duration::hours(10);
        assert_eq!("10 h", foo.human_readable());

        let bar = Duration::hours(3)
            .checked_add(&Duration::minutes(3))
            .unwrap();
        assert_eq!("3 h 3 m", bar.human_readable());
    }

    #[test]
    fn human_readable_minutes() {
        let foo = Duration::minutes(10);
        assert_eq!("10 m", foo.human_readable());

        let bar = Duration::minutes(3)
            .checked_add(&Duration::seconds(3))
            .unwrap();
        assert_eq!("3 m 3 s", bar.human_readable());
    }

    #[test]
    fn human_readable_seconds() {
        let foo = Duration::seconds(10);
        assert_eq!("10 s", foo.human_readable());

        let bar = Duration::seconds(3)
            .checked_add(&Duration::milliseconds(3))
            .unwrap();
        assert_eq!("3 s 3 ms", bar.human_readable());
    }

    #[test]
    fn human_readable_milliseconds() {
        let foo = Duration::milliseconds(10);
        assert_eq!("10 ms", foo.human_readable());

        let bar = Duration::milliseconds(3)
            .checked_add(&Duration::microseconds(3))
            .unwrap();
        assert_eq!("3 ms 3 µs", bar.human_readable());
    }

    #[test]
    fn human_readable_microseconds() {
        let foo = Duration::microseconds(10);
        assert_eq!("10 µs", foo.human_readable());

        let bar = Duration::microseconds(3)
            .checked_add(&Duration::nanoseconds(3))
            .unwrap();
        assert_eq!("3 µs 3 ns", bar.human_readable());
    }

    #[test]
    fn human_readable_nanoseconds() {
        let foo = Duration::nanoseconds(10);
        assert_eq!("10 ns", foo.human_readable());
    }
}
