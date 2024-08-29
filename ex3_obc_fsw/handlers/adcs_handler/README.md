# ADCS Handler

To run the handler run

```bash
cargo run --bin adcs_handler
```

## Running from Groundstation to Simulated ADCS

The handler will translate commands sent from the msg_dispatcher via IPC, and send those commands to the adcs_server via TCP. To run the entire structure run the following commands:

1. In the repository root `cargo run --bin cli_ground_station`
2. Wherever the `ex3_simulated_subsystems/ADCS` directory is run, `python3 adcs_server.py 1803`
3. In the `ex3_software/ex3_obc_fsw/msg_dispatcher`, run, `make && ./msg_dispatcher`
4. In the repository root `cargo run --bin adcs_handler`
5. In the repository root `cargo run --bin bulk_msg_dispatcher`
6. In the repository root `cargo run --bin coms_handler`

## TODO

- [ ] Eventually move to more realistic ADCS packets
- [ ] Create a more realized error handling system
- [x] Move to use the `ipc` crate
- [ ] Use logging
