# GS Cli for testing OBC

This will act as a quick and dirty GS for sending commands to the OBC 

Create TCP stream with user specified port, writes bytes to it, and closes

All fields right now are bytes

Internal OBC packets structure (for now) is as follows, where EVERY command has a len, dest, opcode, and the data portion is optional as it depends on the structure of a particular command. For example a command to get housekeeping may only use len | dest | opcode, but a command to tell IRIS to capture an image may be len | dest | opcode | latitude | longitude ....

| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | ... | Byte 63 |
| :----: | :----: | :----: | :----: | :----: | :----: | :----: |
| Length | Dest   | Opcode | Data 0 | Data 1 | ... | 0 |

Data values are input as a string slice by user and converted into a byte vector

```@text
Usage: <obc_port> <subsystem_id> <subsystem_op_code> optional:<data> ...
```

Example command:

```@bash
    cargo run -- 8001 2 4 data_here
```
