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
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<E: fmt::Debug, K> fmt::Debug for Error<E, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<E: fmt::Debug, K: error::Error + 'static> error::Error for Error<E, K> {
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
    use embedded_hal::digital::{self, Error as _};
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
        #[error("Hal")]
        Hal(#[from] Error<E, digital::ErrorKind>),
        #[error("logic")]
        Logic,
    }
    impl<E: digital::Error> From<E> for DriverError<E> {
        fn from(value: E) -> Self {
            Error::from(value).into()
        }
    }

    fn driver<P: digital::OutputPin>(pin: &mut P) -> Result<(), DriverError<P::Error>> {
        pin.set_low().or(Err(DriverError::Logic))?;
        Ok(pin.set_high()?)
    }

    // user
    #[test]
    fn it_works() {
        let driver_err = driver(&mut Pin).unwrap_err();
        let pin_err = driver_err.source().unwrap();
        assert!(matches!(
            pin_err
                .downcast_ref::<Error<PinError, digital::ErrorKind>>()
                .unwrap()
                .kind(), // Deref
            digital::ErrorKind::Other
        ));
        let kind = pin_err.source().unwrap();
        assert!(matches!(
            kind.downcast_ref::<digital::ErrorKind>().unwrap(),
            digital::ErrorKind::Other
        ));
        assert!(kind.source().is_none());
        // println!("{driver_err:?}: {driver_err}\n{pin_err:?}: {pin_err}\n{kind:?}: {kind}");
    }

    #[test]
    #[ignore]
    fn with_anyhow() -> anyhow::Result<()> {
        Ok(driver(&mut Pin)?)
    }
}
