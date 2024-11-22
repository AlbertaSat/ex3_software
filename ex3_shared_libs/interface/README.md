# Interfaces for connecting components

This library provides interfaces which are used by handlers and allow them to communicate with external peripherals, such as subsystems and payloads. 

## What is this

The `Interface` struct provides methods to initialize a UART connection, and read and write messages using the defined `Msg` format as well as writing and reading raw bytes from devices. This interface is used to communicate with devices onboard the satellite that use UART or other serial connections.

### Features

- Initialize a UART connection with a TTY device path and baud rate.
- Read messages from the UART interface.
- Write messages to the UART interface.

## I2C Interface
This library allows userspace programs to communicate with devices via the I2C communication protocol. It includes an I2cDeviceInterface Structure that allows the user to read and write message structures as well as raw bytes over the I2C interface.

### Features
- Constructor to construct the I2C interface.
- read and send implementation for trait Interface allows user to read and write message structures using I2C interface.
- read and send function implemented allowing for communication using raw bytes.
