#!/usr/bin/bash
# Written by Devin Headrick 
# Summer 2024

gnome-terminal -t IPC_SERVER -- sh -c 'cd ../ex3_obc_fsw/dev_tools/ipc_server_dev_tool && cargo run cmd_msg_dummy_handler; exec bash'
sleep 0.25

gnome-terminal -t TCP_SERVER -- sh -c 'nc -l 127.0.0.1 1807; exec bash'
sleep 0.25

gnome-terminal -t DUMMY_HANDLER -- sh -c 'cd ../ex3_obc_fsw/handlers/dummy_handler && cargo run; exec bash'