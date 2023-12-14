use std::error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrType {
    Internal,
    User,
}

#[derive(Debug)]
pub struct Err {
    pub inner: Option<Box<dyn error::Error + 'static>>,
    pub err_type: ErrType,
    pub msg: String,
}

impl error::Error for Err {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.inner.as_ref().map(|b| b.as_ref())
    }
}

impl fmt::Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.err_type {
            ErrType::Internal => write!(f, "Internal error: {}\nCause: {:?}", self.msg, self.inner),
            ErrType::User => write!(f, "User error: {}", self.msg),
        }
    }
}

impl Err {
    pub fn from_error(inner: Box<dyn error::Error + 'static>, msg: String) -> Self {
        Err {
            inner: Some(inner),
            err_type: ErrType::Internal,
            msg,
        }
    }

    pub fn from_msg_internal(msg: String) -> Self {
        Err {
            inner: None,
            err_type: ErrType::Internal,
            msg,
        }
    }

    pub fn from_user(msg: String) -> Self {
        Err {
            inner: None,
            err_type: ErrType::User,
            msg,
        }
    }
}
