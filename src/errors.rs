use core::fmt::Formatter;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::num::ParseIntError;

use rustls::pki_types::InvalidDnsNameError;

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

impl_err_with_from_str!(LoginError);

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
