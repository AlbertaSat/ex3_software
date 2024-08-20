#!/usr/bin/bash
# Written by Devin Headrick 
# Summer 2024

# This is for testing the handler as simple as possible - Only the handler and 'ipc server dev tool' are built and run,
# making it easier to test the handler without the need for the entire system

#  [IPC SERVER DEV TOOL] <--> [EPS HANDLER] 

#   INSTEAD OF:

# [GS] <--> [SIM UHF] <--> [COMS HANDLER] <--> [MSG DISPATCHER] <--> [EPS HANDLER] <--> [SIM EPS SUBSYSTEM]

gnome-terminal -t IPC_SERVER_DEV_TOOL -- sh -c 'cd ../ex3_obc_fsw/dev_tools/ipc_server_dev_tool && cargo run -- eps_handler; exec bash'
sleep 0.25

gnome-terminal -t EPS_HANDLER -- sh -c 'cd ../ex3_obc_fsw/handlers/eps_handler && cargo run; exec bash'

