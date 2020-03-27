/// This module provides de/serialization for the Measurement type.

// NOTE: If `Measurement`'s chrono::Duration field should ever support
//        proper de/serialization, this entire module can be removed.

use crate::Measurement;
use chrono;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

impl Serialize for Measurement {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if let Some(nanos) = self.0.num_nanoseconds() {
            s.serialize_str(&format!("{}", nanos))
        } else {
            s.serialize_str("overflow")
        }
    }
}


struct MeasurementVisitor;
impl<'de> serde::de::Visitor<'de> for MeasurementVisitor {
    type Value = Measurement;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Measurement that is within range")
    }

    fn visit_str<E>(self, string: &str) -> Result<Measurement, E>
    where E: serde::de::Error {
        let serde_err = |msg| Err(serde::de::Error::custom(msg));
        match string {
            "overflow" => serde_err("Failed to serialize Duration: overflow"),
            _ => match string.parse() {
                Ok(n) => Ok(Measurement(chrono::Duration::nanoseconds(n))),
                Err(_from_str_err) => serde_err(
                    &format!("Failed to parse Duration: {}", string)
                ),
            }
        }
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
        assert_eq!(json_string, "\"10980000000000\"");
    }

    #[test]
    fn deserialize() {
        const JSON_STRING: &str = "\"10980000000000\"";
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
