# Interfaces for connecting components

This library provides interfaces which are used by handlers and allow them to communicate with external peripherals, such as subsystems and payloads.

## UART Interface
This module allows userspace programs to talk with devices via a serial port connection. It implements the Interface trait to send and recieve bytes of data over the serial port connection.

## I2C Interface
This library allows userspace programs to communicate with devices via the I2C communication protocol. It includes an I2cDeviceInterface Structure that allows the user to read and write raw bytes over the I2C interface.

## I2C Interface
This library allows userspace programs to communicate with devices via the SPI communication protocol. It includes an SpiInterface Structure that allows the user to read and write raw bytes over the SPI interface.

## IPC

This is the libary used by various OBC FSW component to communicate with eachother using IPC Unix Domain Sockets of type SOCKSEQ packet.

The library provides a Server and Client struct, and helper functions to allow the Component using them to poll for incomming connection requests or data.

### Usage
Other FSW components can use this library by importing it in their Cargo.toml file, and using the new constructors for both Server and Client types to create an assocaited interface.

Client socket inputs are read using the poll_ipc_client_sockets function, which takes a vector of IpcClient objects.

Server socket inputs are read using the poll_ipc_sever_sockets function, which takes a vector of IpcServer objects.

### IMPORTANT
When data is read the associated buffer of that object is mutated, and thus it is __UP TO THE USER OF THE INTERFACE__ to clear the buffer after they are done reading data from it, before they perform another read.

## TCP Interfac
Read and send functions are part of the TcpInterface struct and can be called whenever a process wants to simulate communicating with a peripheral.
The external handlers which use these interfaces can use these functions to send and receive data to and from the interface asynchronously (non blocking).
Polling is used to allow for this behaviour.

A TcpInterface is used to faciliate communication between handlers and their associated simulated subsystem. This is to mock the actual connection with real hardware which will be made in the future.

### Testing TCP
To test the TCP Interface you can enter the following command inside the ex3_shared_libs/interfaces directory. Be sure you have a Tcp server available for connection on the specified port:

```@sh
    cargo test -- --nocapture 
```

To run a specific test fxn, for example 'test_handler_read', use the following command:

```@sh
    cargo test tests::test_handler_read -- --exact --nocapture
```

