# CLI Test Msg

This program will create a TCP client that can be used to test a single programs ability to deserialize and Msg struct that is defined in ex3_shared_libs/message_structure.

## Running

```@bash
cargo run <port>
```

It creates its own default Msg struct to pass along the socket. It is then up to the program to handle it.

## Testing Scheduler

To send a command to the scheduler you specify a time for the command to run.

```@bash
cargo run <port> <execution_time>
```

The execution time must be in a string. An example command would be:

```@bash
cargo run 1901 "2024-06-17 22:22:22"
```
