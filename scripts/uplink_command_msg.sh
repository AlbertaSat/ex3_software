#!/usr/bin/bash
# Written by Devin Headrick 
# Summer 2024

# User must provide the path to the simulated subsystem directory on their machine as the first arg
PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

## Create the simulated subystem components (dfgm and uhf transciever) - because they are tcp servers  
gnome-terminal -t SIM_DFGM_SUBSYSTEM -- sh -c "cd $PATH_TO_SIM_SUBS/DFGM && python3 ./dfgm_subsystem.py ; bash exec;"
gnome-terminal -t SIM_IRIS_SUBSYSTEM -- sh -c "cd $PATH_TO_SIM_SUBS/IRIS && python3 ./iris_simulated_server.py ; bash exec;"
# For now the UHF transceiver is bypassed and the GS sends msgs directly to the coms handler 

# ## Create the msg dispatcher (first component of the obc fsw because it creates ipc servers 
gnome-terminal -t MSG_DISPATCHER -- sh -c 'cd ../ex3_obc_fsw/msg_dispatcher && make && ./msg_dispatcher; exec bash'
sleep 0.25

# Create bulk msg dispatcher 
gnome-terminal -t BULK_MSG_DISPATCHER -- sh -c 'cd ../ex3_obc_fsw/bulk_msg_dispatcher && cargo run; exec bash'
sleep 0.25

# ## Create the hanlders and other obc fsw components (coms handler, dfgm handler, etc. )
gnome-terminal -t DFGM_HANDLER -- sh -c 'cd ../ && cargo run --bin dfgm_handler; exec bash'
gnome-terminal -t COMS_HANDLER -- sh -c 'cd ../ && cargo run --bin coms_handler; exec bash'
gnome-terminal -t IRIS_HANDLER -- sh -c 'cd ../ && cargo run --bin iris_handler; exec bash'

## Launch the GS simulation (this can just be a tcp server for now )
gnome-terminal -t SIM_GS -- sh -c 'cd ../ && cargo run --bin cli_ground_station; bash exec'
