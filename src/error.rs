#[cfg(feature = "enable_serde")] use serde;
use std::error::Error;
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


#[cfg(feature = "enable_serde")]
pub(crate) fn serde_err_from<E>(err: MeasureErr) -> E
where E: serde::de::Error {
    serde::de::Error::custom(match err {
        MeasureErr::Overflow => "arithmetic overflow",
        MeasureErr::Underflow => "arithmetic underflow",
        MeasureErr::ParseIntError(IntErrorKind::Empty) =>
            "cannot parse integer from empty string",
        MeasureErr::ParseIntError(IntErrorKind::InvalidDigit) =>
            "invalid digit found in string",
        MeasureErr::ParseIntError(IntErrorKind::Overflow) =>
            "number too large to fit in target type",
        MeasureErr::ParseIntError(IntErrorKind::Underflow) =>
            "number too small to fit in target type",
        MeasureErr::ParseIntError(IntErrorKind::Unknown { ref msg }) =>
            msg,
    })
}

impl From<ParseIntError> for MeasureErr {
    fn from(err: ParseIntError) -> MeasureErr {
        match err.description() {
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
