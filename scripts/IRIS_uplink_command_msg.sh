#!/usr/bin/bash
# Written by Devin Headrick 
# Edited by Ben Fisher
# Summer 2024

# User must provide the path to the simulated subsystem directory on their machine as the first arg
PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

# Create a detached session using our config file to hold our windows
tmux -f .tmux.conf new-session -d -s "IRIS_uplink_command_msg"

## Create the IRIS simulated subystem components because they are tcp servers  
tmux new-window -n "SIM_IRIS_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/IRIS && python3 ./iris_simulated_server.py ; exec bash;"
#                                           ^ to continue after CTRL+C

# For now the UHF transceiver is bypassed and the GS sends msgs directly to the coms handler 

# ## Create the msg dispatcher (first component of the obc fsw because it creates ipc servers 
tmux new-window -n "MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/msg_dispatcher && make && ./msg_dispatcher; exec bash"
sleep 0.25

# Create bulk msg dispatcher 
tmux new-window -n "BULK_MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/bulk_msg_dispatcher && cargo run; exec bash"

# ## Create the hanlders and other obc fsw components (coms handler, dfgm handler, etc. )
tmux new-window -n "SIM_UHF_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/UHF && python3 ./simulated_uhf.py ; exec bash;"
tmux new-window -n "COMS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin coms_handler; exec bash"
tmux new-window -n "IRIS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin iris_handler; exec bash"

## Launch the GS simulation (this can just be a tcp client for now )
tmux new-window -n "SIM_GS" -- "trap : SIGINT; cd ../ && cargo run --bin cli_ground_station; exec bash"

tmux attach-session -t "IRIS_uplink_command_msg"
