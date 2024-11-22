## TCP Interfac

Read and send functions are part of the TcpInterface struct and can be called whenever a process wants to simulate communicating with a peripheral.
The external handlers which use these interfaces can use these functions to send and receive data to and from the interface asynchronously (non blocking).
Polling is used to allow for this behaviour.

A TcpInterface is used to faciliate communication between handlers and their associated simulated subsystem. This is to mock the actual connection with real hardware which will be made in the future.

### Testing

To test the TCP Interface you can enter the following command inside the ex3_shared_libs/interfaces directory. Be sure you have a Tcp server available for connection on the specified port:

```@sh
    cargo test -- --nocapture 
```

To run a specific test fxn, for example 'test_handler_read', use the following command:

```@sh
    cargo test tests::test_handler_read -- --exact --nocapture
```

## Logging library

This library contains functions and features that cleanly and conveniently enable logging to be used throughout the OBC FSW. It is to be implemented within each FSW component to act as a logger and history of what occured with context. Logs allow operators on the ground to review the history of events on the SC and determine what occurred between passes. They are critical in providing information for debugging errors and incorrect behavior, and allow team members to determine what went wrong so an informed solution can be developed. Log messages must be independent, and should be easily machine parsable.

Duplicate logs not of a high severity should be aggregated, such that they do not ‘fill up’ the log history and potentially cover up other important events that were logged. This can be implemented through a constraint on the time between logs of the event over a duration of n. Care should be taken when considering escalating the severity of an emitted log (think… if this error or log is generated, would this be something worth getting a call at 3am by your boss?).

### Usage

Right now the logger creates a 'log' directory in the project directory that the init_logger fxn is called in.

To use this library include it in your modules Cargo.toml file, and just call the 'init_logger' fxn at the beginning of the main loop of the program. After this is done you can then use the associated log macros to both store the log in a file, and print the log to stdin.

```@Rust
error!("Put your error message here");
```

### Log4rs

Log4rs has an architecture that is allows our logs to be written to various locations, formatted, and filtered conventiently.

Log4rs uses a 'yaml' file for configuration, which can be programatically configured but instead we are using a static file for init;