# Dummy IPC client in Rust for talking to the messsage dispatcher

This is our first go at setting up a rust process as an IPC client using Unix Domain Sockets of type SOCK_SEQPACKET.

This will interact with the message dispatcher currently implemented in C.

This program polls both the stdin (fd = 0), as well as the associated unix domain socket (fifo in /tmp/...) setup by the server for a particular associated component.

User input written to stdin is sent to the message dispatcher. Messages received by the message dispatcher with the current connected component id are read.

## Usage

To run this program, first ensure the message dispatcher is running. It should have already created the unix domain socket and associated special file (fifo in /tmp)

Use the following command in the root directory of this cargo project (/ipc_client_test) to launch this program, where ```<client_id>``` is the name of the component the client is acting as (which also serves as the name of the fifo in /tmp).

```@sh
cargo run -- <client_id>
```

## How to be a Unix Domain Socket

1. Call socket() to create a unix domain socket to communicate through
2. Setup a sockaddr_un struct with the address the server is listening on (fifo path) and call connect() on that
3. read and write as desired
