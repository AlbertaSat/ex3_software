# Interfaces for connecting components

This library provides interfaces which are use by handlers and allow them to communicate with external peripherals, such as subsystems and payloads.

## What is this

Read and send functions are part of the TcpInterface struct and can be called whenever a process wants to simulate communicating with a peripheral.
The external handlers which use these interfaces can use these functions to send and receive data to and from the interface asynchronously (non blocking).
Polling is used to allow for this behaviour.

A TcpInterface is used to faciliate communication between handlers and their associated simulated subsystem. This is to mock the actual connection with real hardware which will be made in the future.

## Testing

### Testing the TcpInterface

To test the TCP Interface you can enter the following command inside the ex3_shared_libs/interfaces directory. Be sure you have a Tcp server available for connection on the specified port:

```@sh
    cargo test -- --nocapture 
```

To run a specific test fxn, for example 'test_handler_read', use the following command:

```@sh
    cargo test tests::test_handler_read -- --exact --nocapture
```
