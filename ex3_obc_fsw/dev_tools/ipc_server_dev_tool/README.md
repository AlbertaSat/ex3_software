# IPC Server Dev Tool

This is a tool created to act as an IPC server that emits messages based on a number the user enters in the terminal.

i.e. open this and run '1' will send a message of opcode '0' to the dummy handler

## Usage

run the script in the 'scripts' directory at the ex3_software root directory.

```@sh
bash test_dummy_handler <name_of_ipc_client_to_connect_with>
```

## TODO

- Every opcode should have an 'example' command - this can be used to emit that example command when the associated opcode is entered
