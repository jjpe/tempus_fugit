#[cfg(feature = "enable_serde")] use serde_derive::{Deserialize, Serialize};
use std::num::ParseIntError;


pub type MeasureResult<T> = Result<T, MeasureErr>;


#[cfg(feature = "enable_serde")]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum MeasureErr {
    Overflow,
    Underflow,
    ParseIntError(IntErrorKind),
}

#[cfg(not(feature = "enable_serde"))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeasureErr {
    Overflow,
    Underflow,
    ParseIntError(IntErrorKind),
}


#[cfg(feature = "enable_serde")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum IntErrorKind {
    Empty,
    InvalidDigit,
    Overflow,
    Underflow,
    Unknown { msg: String },
}

#[cfg(not(feature = "enable_serde"))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntErrorKind {
    Empty,
    InvalidDigit,
    Overflow,
    Underflow,
    Unknown { msg: String },
}


impl From<ParseIntError> for MeasureErr {
    fn from(err: ParseIntError) -> MeasureErr {
        match format!("{}", err).as_str() {
            "cannot parse integer from empty string" =>
                MeasureErr::ParseIntError(IntErrorKind::Empty),
            "invalid digit found in string" =>
                MeasureErr::ParseIntError(IntErrorKind::InvalidDigit),
            "number too large to fit in target type" =>
                MeasureErr::ParseIntError(IntErrorKind::Overflow),
            "number too small to fit in target type" =>
                MeasureErr::ParseIntError(IntErrorKind::Underflow),
            msg => MeasureErr::ParseIntError(IntErrorKind::Unknown {
                msg: String::from(msg)
            })
        }
    }
}
