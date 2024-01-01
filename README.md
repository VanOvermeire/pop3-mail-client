# Pop3 Mail Client

One of many Pop3 clients for Rust.

This one differs from others in some additional safeties (can't compile when you've not connected and/or logged in yet) using type-state.
It also offers some convenience methods.

Despite this (and based on very limited testing), performance is about the same as that of the other projects mentioned below.

## Installation

...

## Examples

...

## Original RFC

https://www.ietf.org/rfc/rfc1939.txt

## Implemented

- stat
- list (`list` and `list_id`)
- retr (`retrieve`)
- rset (`reset`)
- dele (`delete`)
- uidl (`uidl` and `uidl_with_id`)
- noop
- top

## Not implemented

- apop

## Similar projects

- https://github.com/mattnenterprise/rust-pop3 (abandoned -> forked)
- https://github.com/falk-werner/rust-pop3-client/tree/main?tab=readme-ov-file

## TODO

- convenience methods, like last mail, last x mail ids
- move everything from lib.rs to separate files and re-export
