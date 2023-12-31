# Pop3 Mail Client

One of many Pop3 clients for Rust.

This one differs from others in some additional safeties (can't compile when you've not connected and/or logged in yet) using type-state.
It also offers some convenience methods.

Performance is about the same as that of the other projects mentioned below.

## Implemented

- stat
- list
- retr (retrieve)
- rset (reset)
- dele (delete)
- noop

## Not implemented

- update, which could potentially remove messages, so seemed safer not to add this without a use case for it

## Original RFC

https://www.ietf.org/rfc/rfc1939.txt

## Similar projects

- https://github.com/mattnenterprise/rust-pop3 (abandoned -> forked)
- https://github.com/falk-werner/rust-pop3-client/tree/main?tab=readme-ov-file

## TODO

- top, uidl, apop
- convenience methods, like last 10 mails
- documentation
- error to combine all errors
- github actions setup

