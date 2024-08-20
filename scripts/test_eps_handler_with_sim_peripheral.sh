#!/usr/bin/bash
# Written by Devin Headrick 
# Summer 2024

# This is for testing the handler as simple as possible - Only the handler, sim subsystem and 'ipc server dev tool' are used

#  [IPC SERVER DEV TOOL] <--> [EPS HANDLER] <--> [SIMULATED EPS SUBSYSTEM]

#   INSTEAD OF:

# [GS] <--> [SIM UHF] <--> [COMS HANDLER] <--> [MSG DISPATCHER] <--> [EPS HANDLER] <--> [SIM EPS SUBSYSTEM]

PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

gnome-terminal -t SIM_EPS_SUBSYSTEM -- sh -c "cd $PATH_TO_SIM_SUBS/EPS && python3 ./eps_subsystem.py ; bash exec;"

gnome-terminal -t IPC_SERVER_DEV_TOOL -- sh -c 'cd ../ex3_obc_fsw/dev_tools/ipc_server_dev_tool && cargo run -- dummy_handler; exec bash'
sleep 0.25

gnome-terminal -t EPS_HANDLER -- sh -c 'cd ../ex3_obc_fsw/handlers/eps_handler && cargo run; exec bash'

