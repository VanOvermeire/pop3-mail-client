# Pop3 Mail Client

One of many Pop3 clients for Rust.

This one differs from others in some additional safeties (can't compile when you've not connected and/or logged in yet) using type-state.
It also offers some convenience methods.

Despite this, based on very limited testing, performance is about the same as that of the other projects mentioned below.

## Installation

`cargo install pop3-mail-client`

Or add this to `Cargo.toml`:

```toml
[dependencies]
pop3-mail-client = "0.1.0"
```

## Examples

Create the client using its builder and starting calling the client methods.

```rust
use pop3_mail_client::{Pop3Connection, Pop3Client, Pop3Error};

fn main() -> Result<(), Pop3Error> {
    // create a client that connects to Microsoft outlook
    // there is also a helper for gmail, and a `new` method for connecting to any server
    let mut connection = Pop3Client::builder()
        .username("test@gmail.com")
        .password("some-pass")
        .connect(Pop3Connection::outlook())?;

    // retrieve stats
    let stats = connection.stat()?;
    println!("{stats:?}");

    // or list messages and print their ids
    let lists = connection.list()?;
    lists.messages.iter().for_each(|m| println!("{}", m.message_id));

    // retrieve a message as a string
    let res = connection.retrieve_as_string(9999);
    println!("{:?}", res);

    Ok(())
}

```

`Pop3Error` is a union of all possible errors.

Alternatively, you can also match on the specific errors returned by each method:

```rust
use pop3_mail_client::{Pop3Connection, Pop3Client, Pop3Error, ListError};

fn main() -> Result<(), Pop3Error> {
    let mut connection = Pop3Client::builder()
        .username("test@gmail.com")
        .password("some-pass")
        .connect(Pop3Connection::outlook())?;

    match connection.list() {
        // we got back a list of messages
        Ok(list) => {
            list.messages.iter().for_each(|m| println!("{}", m.message_id));
        }
        // list might return a ListError
        Err(ListError { message }) => {
            println!("An error: {}", message);
        }
    }
    
    Ok(())
}
```

## Errors

`Pop3Error` is a union (enum) of the following errors:

- ConnectionError
- StatError
- ListError
- RetrieveError
- DeleteError
- ResetError
- NoopError
- TopError
- UIDLError

## Implemented commands

Name of the command, plus the name in this implementation.

- stat
- list (`list` and `list_id`)
- retr (`retrieve` or `retrieve_as_string`)
- rset (`reset`)
- dele (`delete`)
- uidl (`uidl` and `uidl_with_id`)
- noop
- top

## Convenience commands

- `list_last` (list last x message ids and sizes)
- `retrieve_last_as_string` (retrieve the last email as a string)
- `retrieve_last` (retrieve the last email and pass it to a writer)

## Not implemented

- apop

## Original RFC

https://www.ietf.org/rfc/rfc1939.txt

## Similar projects

- https://github.com/mattnenterprise/rust-pop3 (abandoned -> forked)
- https://github.com/falk-werner/rust-pop3-client (works fine)

## TODO

- more convenience methods?
- move everything from lib.rs to separate files and re-export?
