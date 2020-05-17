use std::{error, fmt, io, sync::Arc};

/// Replacement of `std::io::Error` implementing `Eq + Clone`
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error(Repr);

#[derive(PartialEq, Eq, Clone, Debug)]
enum Repr {
    Os(i32),
    Simple(io::ErrorKind),
    Custom(io::ErrorKind, ArcError),
}

#[derive(Clone, Debug)]
struct ArcError(Arc<dyn error::Error + Send + Sync>);

impl PartialEq for ArcError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ArcError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Repr::Os(os) => io::Error::from_raw_os_error(*os).fmt(f),
            Repr::Simple(kind) => io::Error::from(*kind).fmt(f),
            Repr::Custom(_, err) => err.0.fmt(f),
        }
    }
}

impl error::Error for Error {
    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn error::Error> {
        if let Repr::Custom(_, err) = &self.0 {
            err.0.cause()
        } else {
            None
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Repr::Custom(_, err) = &self.0 {
            err.0.source()
        } else {
            None
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        if let Some(os) = e.raw_os_error() {
            return Self(Repr::Os(os));
        }
        let kind = e.kind();
        Self(if let Some(inner) = e.into_inner() {
            Repr::Custom(kind, ArcError(inner.into()))
        } else {
            Repr::Simple(kind)
        })
    }
}

impl From<io::ErrorKind> for Error {
    fn from(kind: io::ErrorKind) -> Self {
        Self(Repr::Simple(kind))
    }
}

impl Error {
    pub fn new<E>(kind: io::ErrorKind, error: E) -> Self
    where
        E: Into<Arc<dyn error::Error + Send + Sync>>,
    {
        Self(Repr::Custom(kind, ArcError(error.into())))
    }

    pub fn last_os_error() -> Self {
        Self::from(io::Error::last_os_error())
    }

    pub fn from_raw_os_error(code: i32) -> Self {
        Self(Repr::Os(code))
    }

    pub fn raw_os_error(&self) -> Option<i32> {
        if let Repr::Os(os) = &self.0 {
            Some(*os)
        } else {
            None
        }
    }

    pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        if let Repr::Custom(_, err) = &self.0 {
            Some(&*err.0)
        } else {
            None
        }
    }

    pub fn into_inner(self) -> Option<Arc<dyn error::Error + Send + Sync>> {
        if let Repr::Custom(_, err) = self.0 {
            Some(err.0)
        } else {
            None
        }
    }

    pub fn kind(&self) -> io::ErrorKind {
        match &self.0 {
            Repr::Os(os) => io::Error::from_raw_os_error(*os).kind(),
            Repr::Simple(kind) | Repr::Custom(kind, _) => *kind,
        }
    }
}

#[test]
fn test_readme_example() {
    let ioe = std::fs::read_dir("/dev/null").unwrap_err();

    let e1 = Error::from(ioe);
    let e2 = e1.clone();
    assert_eq!(e1, e2);

    let e1 = Error::new(io::ErrorKind::Other, Box::from("foo"));
    let e2 = Error::new(io::ErrorKind::Other, Box::from("foo"));
    assert_ne!(e1, e2);

    fn assert_traits<T: Send + Sync + Eq + Clone + error::Error + 'static>() {}
    assert_traits::<Error>();
}
