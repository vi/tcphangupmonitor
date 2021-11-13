Prototype of a Linux-only [Tokio] library that allows you to handle incoming TCP RST.
Works around inability to create empty [`tokio::io::Interest`][i] to receive only [`tokio::io::Ready::WRITE_CLOSED`][wc] without being distracted by read or write readiness.
Intended to be used to cancel operation associated with the socket when it is known that the socket cannot be used anymore.
For example, a TCP conection forwarder should terminate a forwarding based on TCP reset even if we don't have any data to send to the socket now (so are not interested in write readiness), nor can read more data from the socket due to backpressure (so are not interested in read readiness as well).

There are two tcp forwarder examples, with and without the library, with corresponding [tcptunnelchecker][ch] results available as comments:

* [naive](examples/naive_forwarder.rs) - uses just Tokio
* [careful](examples/careful_forwarder.rs) - with this additional library

Notes:

* This implementation **leaks memory** (and straightforward usage can also leak Tokio tasks). This happens in common scenario, so current version should be considered to be useful only for demonstration purposes.
* This library uses `unsafe`.
* This simplictic API is probably impossible to make non-leaky. Either wrapping `tokio::net::TcpStream` or changes to `mio` and/or `tokio` required. The latter is likely to be blocked on non-Linux support.

[Tokio]:https://github.com/tokio-rs/tokio
[i]:https://docs.rs/tokio/1.13.0/tokio/io/struct.Interest.html
[wc]:https://docs.rs/tokio/1.13.0/tokio/io/struct.Ready.html#method.is_write_closed
[ch]:https://github.com/vi/tcptunnelchecker
