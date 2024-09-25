#![no_std]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! Provide `core::error::Error` for `embedded-hal` Errors using a newtype wrapper.

use core::{error, fmt};

/// Wrap a HAL `Error` and store its `ErrorKind` to provide [`core::error::Error`]
///
/// Uses `E: Debug` for `Debug` and `Display` and the
/// stored `ErrorKind` as [`core::error::Error::source()`].
pub struct Error<E, K> {
    inner: E,
    kind: K,
}

impl<E, K> Error<E, K> {
    /// Extract the inner `Error`
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<E, K> core::ops::Deref for Error<E, K> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E: fmt::Debug, K> fmt::Display for Error<E, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<E: fmt::Debug, K> fmt::Debug for Error<E, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<E: fmt::Debug, K: error::Error + 'static> error::Error for Error<E, K> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.kind)
    }
}

macro_rules! impl_from {
    ($($mod:ident)::+) => {
        impl<E: $($mod ::)+ Error> From<E> for Error<E, $($mod ::)+ ErrorKind> {
            fn from(inner: E) -> Self {
                let kind = inner.kind();
                Self { inner, kind }
            }
        }
    };
}

impl_from!(embedded_hal::digital);
impl_from!(embedded_hal::i2c);
impl_from!(embedded_hal::pwm);
impl_from!(embedded_hal::spi);
impl_from!(embedded_can);
impl_from!(embedded_hal_nb::serial);
impl_from!(embedded_io);

#[cfg(test)]
mod tests {
    use super::*;
    use core::error::Error as _;
    use embedded_hal::digital::{self, Error as _};
    use thiserror::Error;

    mod hal {
        use super::*;
        #[derive(Debug)]
        pub struct HalError;
        impl digital::Error for HalError {
            fn kind(&self) -> digital::ErrorKind {
                digital::ErrorKind::Other
            }
        }
        pub struct Pin;
        impl digital::ErrorType for Pin {
            type Error = HalError;
        }
        impl digital::OutputPin for Pin {
            fn set_high(&mut self) -> Result<(), Self::Error> {
                Err(HalError)
            }
            fn set_low(&mut self) -> Result<(), Self::Error> {
                unimplemented!()
            }
        }
    }

    mod driver {
        use super::*;
        #[derive(Debug, Error)]
        pub enum DriverError<E: digital::Error> {
            #[error("Hal")]
            Hal(#[from] Error<E, digital::ErrorKind>),
            // ...
        }
        impl<E: digital::Error> From<E> for DriverError<E> {
            fn from(value: E) -> Self {
                Error::from(value).into()
            }
        }
        pub fn action<P: digital::OutputPin>(pin: &mut P) -> Result<(), DriverError<P::Error>> {
            Ok(pin.set_high()?)
        }
    }

    // user
    #[test]
    fn it_works() {
        use driver::*;
        use hal::*;

        let driver_err = action(&mut Pin).unwrap_err();
        let err_dyn = driver_err.source().unwrap();
        let err: &Error<HalError, digital::ErrorKind> = err_dyn.downcast_ref().unwrap();
        let hal_err: &HalError = err; // Deref
        assert!(matches!(hal_err.kind(), digital::ErrorKind::Other));
        let kind_dyn = err_dyn.source().unwrap();
        let kind: &digital::ErrorKind = kind_dyn.downcast_ref().unwrap();
        assert!(matches!(kind, digital::ErrorKind::Other));
        assert!(kind_dyn.source().is_none());
    }

    #[test]
    #[ignore]
    fn with_anyhow() -> anyhow::Result<()> {
        use driver::*;
        use hal::*;

        Ok(action(&mut Pin)?)
    }
}
