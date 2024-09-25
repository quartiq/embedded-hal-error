#![no_std]

use core::error;
use core::fmt::{Debug, Display};

use embedded_hal as eh;

pub struct Error<E, K> {
    inner: E,
    kind: K,
}

impl<E: Debug, K> Display for Error<E, K> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
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

impl<E: eh::digital::Error> From<E> for Error<E, eh::digital::ErrorKind> {
    fn from(inner: E) -> Self {
        let kind = inner.kind();
        Self { inner, kind }
    }
}

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
    fn driver<P: digital::OutputPin>(p: &mut P) -> Result<(), DriverError<P::Error>> {
        p.set_high().map_err(Error::from)?;
        if p.set_low().is_err() {
            Err(DriverError::Logic)
        } else {
            Ok(())
        }
    }

    #[test]
    fn it_works() {
        let err = driver(&mut Pin).unwrap_err();
        let src = err.source().unwrap().source().unwrap();
        assert!(src.source().is_none());
        // println!("{err}: {src}");
    }
}
