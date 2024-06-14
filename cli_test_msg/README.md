# CLI Test Msg
This program will create a TCP client that can be used to test a single programs ability to deserialize and Msg struct that is defined in libs/message_structure.

### Running
```bash
cargo run <port>
```

It creates its own default Msg struct to pass along the socket. It is then up to the program to handle it. 