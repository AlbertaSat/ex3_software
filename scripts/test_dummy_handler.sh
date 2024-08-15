#!/usr/bin/bash
# Written by Devin Headrick 
# Summer 2024

# This is for testing the dummy handler as simple as possible - Only the handler, dummy sim subsystem and 'ipc server dev tool' are used
#  - The ipc server dev tool takes the place of the msg dispatcher and is used to inject and read commands into the handler direclty 

#  [IPC SERVER DEV TOOL] <--> [DUMMY HANDLER] <--> [SIMULATED DUMMY SUBSYSTEM]

#   INSTEAD OF:

# [GS] <--> [SIM UHF] <--> [COMS HANDLER] <--> [MSG DISPATCHER] <--> [DUMMY HANDLER] <--> [SIM DUMMY SUBSYSTEM]

PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

gnome-terminal -t SIM_DUMMY_SUBSYSTEM -- sh -c "cd $PATH_TO_SIM_SUBS/DUMMY && python3 ./simulated_dummy.py ; bash exec;"

gnome-terminal -t IPC_SERVER_DEV_TOOL -- sh -c 'cd ../ex3_obc_fsw/dev_tools/ipc_server_dev_tool && cargo run -- dummy_handler; exec bash'
sleep 0.25

gnome-terminal -t DUMMY_HANDLER -- sh -c 'cd ../ex3_obc_fsw/handlers/dummy_handler && cargo run; exec bash'

