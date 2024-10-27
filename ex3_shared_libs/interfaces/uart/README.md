# Interfaces for connecting components

This library provides interfaces which are used by handlers and allow them to communicate with external peripherals, such as subsystems and payloads. 

## What is this

The `Interface` struct provides methods to initialize a UART connection, and read and write messages using the defined `Msg` format as well as writing and reading raw bytes from devices. This interface is used to communicate with devices onboard the satellite that use UART or other serial connections.

### Features

- Initialize a UART connection with a TTY device path and baud rate.
- Read messages from the UART interface.
- Write messages to the UART interface.

### Dependencies

- [serialport](https://crates.io/crates/serialport): A crate for serial port communication.
- `message_structure`: A module that defines the `Msg` format used for communication.
