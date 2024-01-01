use std::io::Write;
use std::marker::PhantomData;
use std::net::TcpStream;
use std::sync::Arc;

use rustls::{ClientConnection, StreamOwned};

use reader::read_response;

use crate::client_config::create_rustls_config;
use crate::errors::{ConnectionError, DeleteError, ListError, NoopError, ResetError, RetrieveError, StatError, TopError, UIDLError};
use crate::reader::read_multi_response;
use crate::responses::{ListResponse, RetrieveResponse, StatResponse, TopResponse, UIDLItem, UIDLResponse};

mod client_config;
mod reader;
mod errors;
mod responses;


pub struct Pop3Client {
    stream: StreamOwned<ClientConnection, TcpStream>,
}

impl Drop for Pop3Client {
    fn drop(&mut self) {
        let _ = self.invoke("QUIT");
    }
}

impl Pop3Client {
    pub fn builder() -> Pop3ClientBuilder<Pop3ClientBuilderCredsUsername> {
        Pop3ClientBuilder {
            host: None,
            port: None,
            username: None,
            password: None,
            type_state: Default::default(),
        }
    }

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

pub trait Pop3ClientBuilderState {}

pub struct Pop3ClientBuilderCredsUsername {}
pub struct Pop3ClientBuilderCredsPassword {}
pub struct Pop3ClientBuilderConnect {}

impl Pop3ClientBuilderState for Pop3ClientBuilderCredsUsername {}
impl Pop3ClientBuilderState for Pop3ClientBuilderCredsPassword {}
impl Pop3ClientBuilderState for Pop3ClientBuilderConnect {}

pub struct Pop3ClientBuilder<T: Pop3ClientBuilderState> {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    type_state: PhantomData<T>,
}

impl Pop3ClientBuilder<Pop3ClientBuilderCredsUsername> {
    pub fn username(self, user: &str) -> Pop3ClientBuilder<Pop3ClientBuilderCredsPassword> {
        Pop3ClientBuilder {
            host: self.host,
            port: self.port,
            username: Some(user.to_string()),
            password: self.password,
            type_state: Default::default(),
        }
    }

    pub fn no_login(self) -> Pop3ClientBuilder<Pop3ClientBuilderConnect> {
        Pop3ClientBuilder {
            host: self.host,
            port: self.port,
            username: None,
            password: None,
            type_state: Default::default(),
        }
    }
}

impl Pop3ClientBuilder<Pop3ClientBuilderCredsPassword> {
    pub fn password(self, password: &str) -> Pop3ClientBuilder<Pop3ClientBuilderConnect> {
        Pop3ClientBuilder {
            host: self.host,
            port: self.port,
            username: self.username,
            password: Some(password.to_string()),
            type_state: Default::default(),
        }
    }
}

impl Pop3ClientBuilder<Pop3ClientBuilderConnect> {
    pub fn connect(self, Pop3Connection { host, port }: Pop3Connection) -> Result<Pop3Client, ConnectionError> {
        let config = create_rustls_config()?;
        let server_name = host.to_string().try_into()?;
        let connection = ClientConnection::new(Arc::new(config), server_name)?;
        let tcp_stream = TcpStream::connect(format!("{}:{}", host, port))?;
        let stream = StreamOwned::new(connection, tcp_stream);

        let mut client = Pop3Client {
            stream,
        };

        client.read_response()?;

        // if the client was created with a username and password, we need to login
        if let (Some(user), Some(pass)) = (self.username, self.password) {
            client.invoke(&format!("USER {user}"))?;
            client.read_response()?;
            client.invoke(&format!("PASS {pass}"))?;
            client.read_response()?;
        }

        Ok(client)
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
