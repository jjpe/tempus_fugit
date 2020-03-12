/// This module provides de/serialization for the Measurement type.

// NOTE: If `Measurement`'s chrono::Duration field should ever support
//        proper de/serialization, this entire module can be removed.

use crate::{Measurement, MeasureErr};
use crate::error;
use chrono;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

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
    fn strip_prefix<'s, E>(prefix: &str, string: &'s str) -> Result<&'s str, E>
    where E: serde::de::Error {
        if !string.starts_with(prefix) {
            let msg = format!("Invalid Measurement string: {}", string);
            return Err(serde::de::Error::custom(msg))
        }
        let prefix_len = prefix.len();
        Ok(&string[prefix_len ..]) // Strip the prefix
    }

    fn parse_part<E>(string: &str) -> Result<(Measurement, &str), E>
    where E: serde::de::Error {
        lazy_static! { static ref NUM: Regex = Regex::new(r"^(\d+)").unwrap(); }
        match NUM.find(string) {
            None => {
                let reason = format!("{} does not start with a number", string);
                let msg = format!("Failed to parse: {}", reason);
                Err(serde::de::Error::custom(msg))
            },
            Some(m) => {
                let num_idx = m.end();
                let num_str = &string[.. num_idx];
                let num = num_str.parse::<i64>()
                    .map_err(MeasureErr::from)
                    .map_err(error::serde_err_from)?;
                let measurement = match &string[num_idx .. num_idx + 1] {
                    "W" => Measurement(chrono::Duration::weeks(num)),
                    "D" => Measurement(chrono::Duration::days(num)),
                    "H" => Measurement(chrono::Duration::hours(num)),
                    "M" => Measurement(chrono::Duration::minutes(num)),
                    "S" => Measurement(chrono::Duration::seconds(num)),
                    unit => {
                        let msg = format!("Invalid date/time unit: {}", unit);
                        return Err(serde::de::Error::custom(msg));
                    },
                };
                Ok((measurement, &string[num_idx + 1 ..]))

            },
        }
    }
}

impl<'de> serde::de::Visitor<'de> for MeasurementVisitor {
    type Value = Measurement;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Measurement that is within range")
    }

    fn visit_str<E>(self, string: &str) -> Result<Measurement, E>
    where E: serde::de::Error {
        let mut string: &str = MeasurementVisitor::strip_prefix("P", string)?;
        let mut measurement = Measurement::zero();
        while !string.is_empty() && !string.starts_with("T") {
            // parse the date component
            let (m, s) = MeasurementVisitor::parse_part(string)?;
            measurement = (measurement + m).map_err(error::serde_err_from)?;
            string = s;
        }

        let mut string: &str = MeasurementVisitor::strip_prefix("T", string)?;
        while !string.is_empty() {
            // parse the time component
            let (m, s) = MeasurementVisitor::parse_part(string)?;
            measurement = (measurement + m).map_err(error::serde_err_from)?;
            string = s;
        }

        Ok(measurement)
    }

    fn visit_string<E>(self, string: String) -> Result<Measurement, E>
    where E: serde::de::Error {
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
    use crate::Measurement;
    use chrono::Duration;
    use serde_json;

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
}
