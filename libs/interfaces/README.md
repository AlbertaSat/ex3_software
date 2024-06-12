# Interfaces for connecting components

This library provides interfaces which are use by handlers and allow them to communicate with external peripherals, such as subsystems and payloads.

## What is this

Async read and write functions take a generic interface type and a sender / receiver channel to allow for async communication with the interface.
The external handlers which use these interfaces can use these functions to send and receive data to and from the interface asynchronously (non blocking).
MPSC channels are used to communicate between the main thread and the async read and write threads.

A TcpInterface is used to faciliate communication between handlers and their associated simulated subsystem. This is to mock the actual connection with real hardware which will be made in the future.

## Testing

### Testing the TcpInterface

To test the TCP Interface you can enter the following command inside the libs/interfaces directory. Be sure you have a Tcp server available for connection on the specified port:

```@sh
    cargo test -- --nocapture 
```

To run a specific test fxn, for example 'test_tcp_interface_server', use the following command:

```@sh
    cargo test test_tcp_interface_server -- --nocapture
```