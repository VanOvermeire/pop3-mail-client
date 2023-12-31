use std::io::{Write};
use std::marker::PhantomData;
use std::mem;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::{ClientConnection, StreamOwned};
use reader::read_response;
use crate::errors::{ConnectionError, DeleteError, ListError, LoginError, NoopError, ResetError, RetrieveError, StatError, TopError, UIDLError};
use crate::reader::{read_multi_response};
use crate::responses::{ListResponse, RetrieveResponse, StatResponse, TopResponse, UIDLItem, UIDLResponse};

use crate::client_config::create_rustls_config;

mod client_config;
mod reader;
mod errors;
mod responses;

pub trait Pop3ClientState {}

pub struct Pop3ConnectingState;

pub struct Pop3AuthorizationState;

pub struct Pop3TransactionState;

impl Pop3ClientState for Pop3ConnectingState {}

impl Pop3ClientState for Pop3AuthorizationState {}

impl Pop3ClientState for Pop3TransactionState {}

pub struct Pop3Client<T: Pop3ClientState> {
    stream: StreamOwned<ClientConnection, TcpStream>,
    type_state: PhantomData<T>,
}

impl<T: Pop3ClientState> Drop for Pop3Client<T> {
    fn drop(&mut self) {
        let _ = self.invoke("QUIT");
    }
}

impl<T: Pop3ClientState> Pop3Client<T> {
    fn invoke(&mut self, command: &str) -> Result<usize, String> {
        Ok(self.stream.write(format!("{command}\r\n").as_bytes()).map_err(|err| err.to_string())?)
    }

    fn read_response(&mut self) -> Result<String, String> {
        read_response(&mut self.stream)
    }

    fn read_multi_response(&mut self) -> Result<String, String> {
        read_multi_response(&mut self.stream)
    }
}

impl Pop3Client<Pop3TransactionState> {
    pub fn stat(&mut self) -> Result<StatResponse, StatError> {
        self.invoke("STAT")?;
        let response = self.read_response()?;
        Ok(response.try_into()?)
    }

    pub fn list(&mut self) -> Result<ListResponse, ListError> {
        self.invoke("LIST")?;
        let response = self.read_multi_response()?;
        Ok(response.try_into()?)
    }

    pub fn list_id(&mut self, message_id: i32) -> Result<ListResponse, ListError> {
        self.invoke(&format!("LIST {message_id}"))?;
        let response = self.read_response()?;
        Ok(response.try_into()?)
    }

    // TODO also retrieve using a &mut impl Write
    pub fn retrieve_as_string(&mut self, message_id: i32) -> Result<RetrieveResponse, RetrieveError> {
        self.invoke(&format!("RETR {message_id}"))?;
        let response = self.read_multi_response()?;
        Ok(RetrieveResponse {
            message_id,
            data: response,
        })
    }

    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.invoke("RSET")?;
        self.read_response()?;
        Ok(())
    }

    pub fn delete(&mut self, message_id: i32) -> Result<(), DeleteError> {
        self.invoke(&format!("DELE {message_id}"))?;
        self.read_response()?;
        Ok(())
    }

    pub fn noop(&mut self) -> Result<(), NoopError> {
        self.invoke("NOOP")?;
        self.read_response()?;
        Ok(())
    }

    pub fn uidl(&mut self) -> Result<UIDLResponse, UIDLError> {
        self.invoke("UIDL")?;
        let response = self.read_multi_response()?;
        Ok(response.try_into()?)
    }

    pub fn uidl_with_id(&mut self, message_id: i32) -> Result<UIDLItem, UIDLError> {
        self.invoke(&format!("UIDL {message_id}"))?;
        let response = self.read_response()?;
        Ok(response.try_into()?)
    }

    pub fn top(&mut self, message_id: i32, number_of_lines: i32) -> Result<TopResponse, TopError> {
        self.invoke(&format!("TOP {message_id} {number_of_lines}"))?;
        let response = self.read_multi_response()?;
        Ok(TopResponse {
            message_id,
            number_of_lines,
            data: response,
        })
    }
}

impl Pop3Client<Pop3AuthorizationState> {
    pub fn login(mut self, username: &str, password: &str) -> Result<Pop3Client<Pop3TransactionState>, LoginError> {
        self.invoke(&format!("USER {username}"))?;
        self.read_response()?;
        self.invoke(&format!("PASS {password}"))?;
        self.read_response()?;

        let r: MaybeUninit<StreamOwned<ClientConnection, TcpStream>> = MaybeUninit::uninit();
        // we consume the current client, so as long as we don't do anything with tls after this call this should be OK
        let stream = mem::replace(&mut self.stream, unsafe { r.assume_init() });
        let _ = ManuallyDrop::new(self); // do have to avoid drop, which would use the stream

        Ok(Pop3Client {
            stream,
            type_state: Default::default(),
        })
    }
}

impl Pop3Client<Pop3ConnectingState> {
    pub fn connect(Pop3Connection { host, port }: Pop3Connection) -> Result<Pop3Client<Pop3AuthorizationState>, ConnectionError> {
        let config = create_rustls_config()?;
        let server_name = host.to_string().try_into()?;
        let connection = ClientConnection::new(Arc::new(config), server_name)?;
        let tcp_stream = TcpStream::connect(format!("{}:{}", host, port))?;
        let mut stream = StreamOwned::new(connection, tcp_stream);

        match read_response(&mut stream) {
            Ok(_) => {
                Ok(Pop3Client {
                    stream,
                    type_state: Default::default(),
                })
            }
            Err(_) => {
                return Err(ConnectionError {
                    message: format!("POP3 server for {} is *not* ready", host),
                });
            }
        }
    }
}

pub struct Pop3Connection<'a> {
    host: &'a str,
    port: u16,
}

impl Pop3Connection<'_> {
    pub fn new(host: &str, port: u16) -> Pop3Connection {
        Pop3Connection { host, port }
    }

    pub fn outlook() -> Pop3Connection<'static> {
        Pop3Connection {
            host: "outlook.office365.com",
            port: 995,
        }
    }

    pub fn gmail() -> Pop3Connection<'static> {
        Pop3Connection {
            host: "pop.gmail.com",
            port: 995,
        }
    }
}
