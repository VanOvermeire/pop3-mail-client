use core::fmt::Formatter;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::num::ParseIntError;

use rustls::pki_types::InvalidDnsNameError;

// helpers //

macro_rules! impl_err {
    ($err:ident) => {
        #[derive(Debug)]
        pub struct $err {
            pub message: String,
        }

        impl Display for $err {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.message)
            }
        }
        impl Error for $err {}
    }
}

macro_rules! implement_from_string {
    ($err:ident) => {
        impl From<String> for $err {
            fn from(message: String) -> Self {
                $err {
                    message,
                }
            }
        }
    };
}

macro_rules! impl_err_with_from_str {
    ($err:ident) => {
        impl_err!($err);
        implement_from_string!($err);
    };
}

macro_rules! implement_pop3_from {
    ($err: ident) => {
        impl From<$err> for Pop3Error {
            fn from(value: $err) -> Self {
                Pop3Error::$err(value)
            }
        }
    };
}

// error that can be used everywhere //

#[derive(Debug)]
pub enum Pop3Error {
    ConnectionError(ConnectionError),
    StatError(StatError),
    ListError(ListError),
    RetrieveError(RetrieveError),
    DeleteError(DeleteError),
    ResetError(ResetError),
    NoopError(NoopError),
    TopError(TopError),
    UIDLError(UIDLError),
}

impl Display for Pop3Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pop3Error::ConnectionError(err) => f.write_str(&format!("ConnectionError: {}", err.message)),
            Pop3Error::StatError(err) => f.write_str(&format!("StatError: {}", err.message)),
            Pop3Error::ListError(err) => f.write_str(&format!("ListError: {}", err.message)),
            Pop3Error::RetrieveError(err) => f.write_str(&format!("RetrieveError: {}", err.message)),
            Pop3Error::DeleteError(err) => f.write_str(&format!("DeleteError: {}", err.message)),
            Pop3Error::ResetError(err) => f.write_str(&format!("ResetError: {}", err.message)),
            Pop3Error::NoopError(err) => f.write_str(&format!("NoopError: {}", err.message)),
            Pop3Error::TopError(err) => f.write_str(&format!("TopError: {}", err.message)),
            Pop3Error::UIDLError(err) => f.write_str(&format!("UIDLError: {}", err.message)),
        }
    }
}

impl Error for Pop3Error {}

implement_pop3_from!(ConnectionError);
implement_pop3_from!(StatError);
implement_pop3_from!(ListError);
implement_pop3_from!(RetrieveError);
implement_pop3_from!(DeleteError);
implement_pop3_from!(ResetError);
implement_pop3_from!(NoopError);
implement_pop3_from!(TopError);
implement_pop3_from!(UIDLError);

// specific errors //

impl_err_with_from_str!(ConnectionError);

impl From<std::io::Error> for ConnectionError {
    fn from(value: std::io::Error) -> Self {
        ConnectionError {
            message: format!("could not set up client connection: {}", value.to_string()),
        }
    }
}

impl From<rustls::Error> for ConnectionError {
    fn from(value: rustls::Error) -> Self {
        ConnectionError {
            message: format!("could not set up client connection: {}", value.to_string()),
        }
    }
}

impl From<InvalidDnsNameError> for ConnectionError {
    fn from(value: InvalidDnsNameError) -> Self {
        ConnectionError {
            message: format!("invalid host: {}", value.to_string()),
        }
    }
}

impl_err_with_from_str!(StatError);

impl From<ParseIntError> for StatError {
    fn from(value: ParseIntError) -> Self {
        StatError {
            message: format!("could not parse stat response as numbers: {}", value.to_string()),
        }
    }
}

impl_err_with_from_str!(ListError);

impl From<ParseIntError> for ListError {
    fn from(value: ParseIntError) -> Self {
        ListError {
            message: format!("could not parse list response numbers: {}", value.to_string()),
        }
    }
}

impl_err_with_from_str!(RetrieveError);

impl From<std::io::Error> for RetrieveError {
    fn from(value: std::io::Error) -> Self {
        RetrieveError {
            message: format!("could not retrieve message: {}", value.to_string()),
        }
    }
}

impl From<ListError> for RetrieveError {
    fn from(value: ListError) -> Self {
        RetrieveError {
            message: value.message,
        }
    }
}

impl_err_with_from_str!(ResetError);

impl_err_with_from_str!(DeleteError);

impl_err_with_from_str!(NoopError);

impl_err_with_from_str!(UIDLError);

impl From<ParseIntError> for UIDLError {
    fn from(value: ParseIntError) -> Self {
        UIDLError {
            message: format!("could not parse UIDL message id as a number: {}", value.to_string()),
        }
    }
}

impl_err_with_from_str!(TopError);
