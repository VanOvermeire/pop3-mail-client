use std::io::Write;
use std::marker::PhantomData;
use std::net::TcpStream;
use std::sync::Arc;

use rustls::{ClientConnection, StreamOwned};

use reader::read_response;

use crate::client_config::create_rustls_config;
use crate::reader::read_multi_response;
use crate::responses::{ItemResponse, ListResponse, RetrieveResponse, StatResponse, TopResponse, UIDLItem, UIDLResponse};

mod client_config;
mod reader;
mod errors;
mod responses;

pub use errors::*;

/// The Pop3Client allows you to connect to a POP3 server and perform actions on it
pub struct Pop3Client {
    stream: StreamOwned<ClientConnection, TcpStream>,
}

impl Drop for Pop3Client {
    fn drop(&mut self) {
        let _ = self.invoke("QUIT");
    }
}

impl Pop3Client {
    /// Create the Pop3Client builder which will set up the Pop3Client
    pub fn builder() -> Pop3ClientBuilder<Pop3ClientBuilderCredsUsername> {
        Pop3ClientBuilder {
            host: None,
            port: None,
            username: None,
            password: None,
            type_state: Default::default(),
        }
    }

    /// Stat requests the number of messages and size in the inbox
    pub fn stat(&mut self) -> Result<StatResponse, StatError> {
        self.invoke("STAT")?;
        let response = self.read_response()?;
        Ok(response.try_into()?)
    }

    /// List generates a list of all message ids, with sizes
    pub fn list(&mut self) -> Result<ListResponse, ListError> {
        self.invoke("LIST")?;
        let response = self.read_multi_response()?;
        Ok(response.try_into()?)
    }

    /// List with a given message_id will return the id and size for that message_Id
    pub fn list_id(&mut self, message_id: i32) -> Result<ItemResponse, ListError> {
        self.invoke(&format!("LIST {message_id}"))?;
        let response = self.read_response()?;
        Ok(response.try_into()?)
    }

    /// List the last x messages
    pub fn list_last(&mut self, number_of_messages: i32) -> Result<ListResponse, ListError> {
        self.invoke(&format!("LIST"))?;
        let response = self.read_multi_response()?;
        let response: ListResponse = response.try_into()?;
        let last_ten = response.messages
            .into_iter()
            .rev()
            .take(number_of_messages as usize)
            .rev()
            .collect();
        Ok(ListResponse {
            messages: last_ten,
        })
    }

    /// Retrieve as string retrieves the content of the message as a string
    pub fn retrieve_as_string(&mut self, message_id: i32) -> Result<RetrieveResponse, RetrieveError> {
        self.invoke(&format!("RETR {message_id}"))?;
        let response = self.read_multi_response()?;
        Ok(RetrieveResponse {
            message_id,
            data: response,
        })
    }

    /// Retrieve the content of the last message as a string
    pub fn retrieve_last_as_string(&mut self) -> Result<RetrieveResponse, RetrieveError> {
        let last = self.list()?;
        let last_message = last.messages.last().ok_or(RetrieveError {
            message: "no messages available".to_string(),
        })?;
        self.invoke(&format!("RETR {}", last_message.message_id))?;
        let response = self.read_multi_response()?;
        Ok(RetrieveResponse {
            message_id: -1,
            data: response,
        })
    }

    /// Retrieve the content of the message and pass it into a writer
    pub fn retrieve(&mut self, message_id: i32, writer: &mut impl Write) -> Result<(), RetrieveError> {
        let as_string = self.retrieve_as_string(message_id)?;
        writer.write(as_string.data.as_bytes())?;
        Ok(())
    }

    /// Retrieve the content of the last message and pass it into a writer
    pub fn retrieve_last(&mut self, writer: &mut impl Write) -> Result<(), RetrieveError> {
        let as_string = self.retrieve_last_as_string()?;
        writer.write(as_string.data.as_bytes())?;
        Ok(())
    }

    /// Reset unmarks all messages that were set as deleted
    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.invoke("RSET")?;
        self.read_response()?;
        Ok(())
    }

    /// Delete marks a given message, by its message_id, as deleted
    pub fn delete(&mut self, message_id: i32) -> Result<(), DeleteError> {
        self.invoke(&format!("DELE {message_id}"))?;
        self.read_response()?;
        Ok(())
    }

    /// Noop is a no-op, which returns nothing. Can be used to test the connection
    pub fn noop(&mut self) -> Result<(), NoopError> {
        self.invoke("NOOP")?;
        self.read_response()?;
        Ok(())
    }

    /// UIDL generates a list of all message ids plus their unique ids
    pub fn uidl(&mut self) -> Result<UIDLResponse, UIDLError> {
        self.invoke("UIDL")?;
        let response = self.read_multi_response()?;
        Ok(response.try_into()?)
    }

    /// UIDL with a given message_id will return the message_id and its unique id
    pub fn uidl_with_id(&mut self, message_id: i32) -> Result<UIDLItem, UIDLError> {
        self.invoke(&format!("UIDL {message_id}"))?;
        let response = self.read_response()?;
        Ok(response.try_into()?)
    }

    /// Top retrieves the number_of_lines of the message (chosen by its message_id)
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

/// The builder for the POP3 client
pub struct Pop3ClientBuilder<T: Pop3ClientBuilderState> {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    type_state: PhantomData<T>,
}

impl Pop3ClientBuilder<Pop3ClientBuilderCredsUsername> {
    /// Set the username for the POP3 client connection
    pub fn username(self, user: &str) -> Pop3ClientBuilder<Pop3ClientBuilderCredsPassword> {
        Pop3ClientBuilder {
            host: self.host,
            port: self.port,
            username: Some(user.to_string()),
            password: self.password,
            type_state: Default::default(),
        }
    }

    /// If you do not have a username and password, use this method to acknowledge that, allowing you to
    /// connect to the server without credentials
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
    /// Set the password for the POP3 client connection
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
    /// Connect to the POP3 server using the details specified in Pop3Connection
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

/// The connection details of the POP3 server
pub struct Pop3Connection<'a> {
    host: &'a str,
    port: u16,
}

impl Pop3Connection<'_> {
    /// Create a new Pop3Connection with the given host and port
    pub fn new(host: &str, port: u16) -> Pop3Connection {
        Pop3Connection { host, port }
    }

    /// Create a new Pop3Connection with the host and port of (Microsoft) Outlook
    pub fn outlook() -> Pop3Connection<'static> {
        Pop3Connection {
            host: "outlook.office365.com",
            port: 995,
        }
    }

    /// Create a new Pop3Connection with the host and port of (Google) Gmail
    pub fn gmail() -> Pop3Connection<'static> {
        Pop3Connection {
            host: "pop.gmail.com",
            port: 995,
        }
    }
}
