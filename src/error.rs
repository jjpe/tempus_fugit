#[cfg(feature = "enable_serde")]
use serde_derive::{Deserialize, Serialize};

pub type MeasureResult<T> = Result<T, MeasureErr>;

#[cfg(feature = "enable_serde")]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum MeasureErr {
    Overflow,
    Underflow,
}

#[cfg(not(feature = "enable_serde"))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeasureErr {
    Overflow,
    Underflow,
}
