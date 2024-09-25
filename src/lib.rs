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
    mod hal {
        use embedded_hal::digital;

        #[derive(Debug)]
        pub struct Error;
        impl digital::Error for Error {
            fn kind(&self) -> digital::ErrorKind {
                digital::ErrorKind::Other
            }
        }

        pub struct Pin;
        impl digital::ErrorType for Pin {
            type Error = Error;
        }
        impl digital::OutputPin for Pin {
            fn set_high(&mut self) -> Result<(), Self::Error> {
                Err(Error)
            }
            fn set_low(&mut self) -> Result<(), Self::Error> {
                unimplemented!()
            }
        }
    }

    mod driver {
        use embedded_hal::digital;

        #[derive(Debug, thiserror::Error)]
        pub enum Error<E: digital::Error> {
            #[error("Hal")]
            Hal(#[from] crate::Error<E, digital::ErrorKind>),
            // ...
        }
        impl<E: digital::Error> From<E> for Error<E> {
            fn from(value: E) -> Self {
                crate::Error::from(value).into()
            }
        }

        pub fn action<P: digital::OutputPin>(pin: &mut P) -> Result<(), Error<P::Error>> {
            Ok(pin.set_high()?)
        }
    }

    #[test]
    fn inspect() {
        use core::error::Error as _;
        use embedded_hal::digital::{Error as _, ErrorKind};

        let driver_err = driver::action(&mut hal::Pin).unwrap_err();
        let err_dyn = driver_err.source().unwrap();
        let err: &crate::Error<hal::Error, ErrorKind> = err_dyn.downcast_ref().unwrap();
        let hal_err: &hal::Error = err; // Deref
        assert!(matches!(hal_err.kind(), ErrorKind::Other));
        let kind_dyn = err_dyn.source().unwrap();
        let kind: &ErrorKind = kind_dyn.downcast_ref().unwrap();
        assert!(matches!(kind, ErrorKind::Other));
        assert!(kind_dyn.source().is_none());
    }

    #[test]
    #[ignore]
    fn with_anyhow() -> anyhow::Result<()> {
        Ok(driver::action(&mut hal::Pin)?)
    }
}
