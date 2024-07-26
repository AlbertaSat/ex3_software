# Logging library

This library contains functions and features that cleanly and conveniently enable logging to be used throughout the OBC FSW. It is to be implemented within each FSW component to act as a legger and history of what occured with context. Logs allow operators on the ground to review the history of events on the SC and determine what occurred between passes. They are critical in providing information for debugging errors and incorrect behavior, and allow team members to determine what went wrong so an informed solution can be developed. Log messages must be independent, and should be easily machine parsable.

Duplicate logs not of a high severity should be aggregated, such that they do not ‘fill up’ the log history and potentially cover up other important events that were logged. This can be implemented through a constraint on the time between logs of the event over a duration of n. Care should be taken when considering escalating the severity of an emitted log (think… if this error or log is generated, would this be something worth getting a call at 3am by your boss?).

## Usage

Right now the logger creates a 'log' directory in the project directory that the init_logger fxn is called in.

To use this library include it in your modules Cargo.toml file, and just call the 'init_logger' fxn at the beginning of the main loop of the program. After this is done you can then use the associated log macros to both store the log in a file, and print the log to stdin.

```@Rust
error!("Put your error message here");
```

## Log4rs

Log4rs has an architecture that is allows our logs to be written to various locations, formatted, and filtered conventiently.

### Log4rs configuration

Log4rs uses a 'yaml' file for configuration, which can be programatically configured but instead we are using a static file for init;
