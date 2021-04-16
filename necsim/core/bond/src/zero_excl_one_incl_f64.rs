use core::{convert::TryFrom, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ZeroExclOneInclF64Error(f64);

impl fmt::Display for ZeroExclOneInclF64Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} is not in (0.0, 1.0].", self.0)
    }
}

#[derive(Copy, Clone, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(try_from = "f64")]
pub struct ZeroExclOneInclF64(f64);

impl TryFrom<f64> for ZeroExclOneInclF64 {
    type Error = ZeroExclOneInclF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Debug for ZeroExclOneInclF64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct ZeroExclOneInclF64Range(f64);

        impl fmt::Debug for ZeroExclOneInclF64Range {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "0.0 < {} <= 1.0", self.0)
            }
        }

        fmt.debug_tuple("ZeroExclOneInclF64")
            .field(&ZeroExclOneInclF64Range(self.0))
            .finish()
    }
}

impl ZeroExclOneInclF64 {
    /// # Errors
    ///
    /// Returns `ZeroExclOneInclF64Error` if not `0.0 < value <= 1.0`
    pub fn new(value: f64) -> Result<Self, ZeroExclOneInclF64Error> {
        if value > 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(ZeroExclOneInclF64Error(value))
        }
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}