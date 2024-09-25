#![no_std]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! Provide `core::error::Error` for `embedded-hal` Errors using a newtype wrapper.

use core::error;
use core::fmt::{Debug, Display};
use core::ops::{Deref, DerefMut};

/// Wrap an Error and store its ErrorKind
///
/// This implements [`core::error::Error`] by using the
/// `E: Debug` for `Debug` and `Display` and the
/// `E: ErrorKind` as [`core::error::Error::source()`].
pub struct Error<E, K> {
    inner: E,
    kind: K,
}

impl<E, K> Error<E, K> {
    /// Extract the inner Error
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<E: Debug, K> Display for Error<E, K> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl<E, K> Deref for Error<E, K> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E, K> DerefMut for Error<E, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<E: Debug, K> Debug for Error<E, K> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl<E: Debug, K: error::Error + 'static> error::Error for Error<E, K> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.kind as &dyn error::Error)
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
    use embedded_hal::digital;
    use thiserror::Error;

    // hal
    #[derive(Debug)]
    struct PinError;
    impl digital::Error for PinError {
        fn kind(&self) -> digital::ErrorKind {
            digital::ErrorKind::Other
        }
    }
    struct Pin;
    impl digital::ErrorType for Pin {
        type Error = PinError;
    }
    impl digital::OutputPin for Pin {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Err(PinError)
        }
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    // driver
    #[derive(Debug, Error)]
    enum DriverError<E: digital::Error> {
        #[error("pin: {0}")]
        Pin(#[from] Error<E, digital::ErrorKind>),
        #[error("logic")]
        Logic,
    }
    fn driver<P: digital::OutputPin>(pin: &mut P) -> Result<(), DriverError<P::Error>> {
        pin.set_high().map_err(Error::from)?;
        if pin.set_low().is_err() {
            Err(DriverError::Logic)
        } else {
            Ok(())
        }
    }

    // user
    #[test]
    fn it_works() {
        let driver_err: DriverError<PinError> = driver(&mut Pin).unwrap_err();
        let pin_err = driver_err.source().unwrap(); // PinError
        let kind = pin_err.source().unwrap(); // digital::ErrorKind::Other
        assert!(kind.source().is_none());
        // println!("{driver_err:?}: {driver_err}\n{pin_err:?}: {pin_err}\n{kind:?}: {kind}");
    }
}
