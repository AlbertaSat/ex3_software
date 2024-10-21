# Interfaces for connecting components

This library provides interfaces which are used by handlers and allow them to communicate with external peripherals, such as subsystems and payloads.

## What is this

The `Interface` struct provides methods to initialize a UART connection, and read and write messages using the defined `Msg` format. This interface is used to communicate with devices onboard the satellite that use UART.

### Features

- Initialize a UART connection with a specified port and baud rate.
- Read messages from the UART interface.
- Write messages to the UART interface.

### Dependencies

- [serialport](https://crates.io/crates/serialport): A crate for serial port communication.
- `message_structure`: A module that defines the `Msg` format used for communication.

### Adding Dependencies

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
serialport = "4.0"
message_structure = { path = "../path_to_message_structure" } # Adjust the path as needed