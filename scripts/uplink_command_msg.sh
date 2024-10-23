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

# Create a detached session using our config file to hold our windows
# IRIS commented out for now while work gets done on the handler
tmux -f .tmux.conf new-session -d -s "uplink_command_msg"

# Create the msg dispatcher (first component of the obc fsw because it creates ipc servers 
tmux new-window -n "CMD_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/cmd_dispatcher && cargo run --bin cmd_dispatcher; exec bash"
sleep 0.25

# Create the simulated subystem components (dfgm and uhf transciever) - because they are tcp servers  
tmux new-window -n "SIM_DFGM_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/DFGM && python3 ./dfgm_subsystem.py; exec bash"
# tmux new-window -n "SIM_IRIS_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/IRIS && python3 ./iris_simulated_server.py; exec bash"

# For now the UHF transceiver is bypassed and the GS sends msgs directly to the coms handler
tmux new-window -n "SIM_UHF" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/UHF && python3 ./simulated_uhf.py; exec bash"
sleep 0.25

# Bulk Dispatcher ommited as this script only focuses on uplink

# Create the hanlders and other obc fsw components (coms handler, dfgm handler, etc. )
tmux new-window -n "DFGM_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin dfgm_handler; exec bash"
tmux new-window -n "COMS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin coms_handler; exec bash"

# tmux new-window -n "IRIS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin iris_handler; exec bash"
sleep 0.25



# Launch the GS simulation (this can just be a tcp client for now )
tmux new-window -n "SIM_GS" -- "trap : SIGINT; cd ../ && cargo run --bin cli_ground_station; exec bash"

tmux attach-session -t "uplink_command_msg"
