#!/usr/bin/bash
# Written by Rowan Rasmusson
# Summer 2024

# User must provide the path to the simulated subsystem directory on their machine as the first arg
PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

# Create a detached session using our config file to hold our windows
tmux -f .tmux.conf new-session -d -s "downlink_payload_data"

## Create the simulated subystem components (DFGM, UHF) - because they are tcp servers  
tmux new-window -n "SIM_UHF_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/UHF && python3 ./simulated_uhf.py ; exec bash"
tmux new-window -n "SIM_DFGM_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/DFGM && python3 ./dfgm_subsystem.py ; exec bash"
#                                             ^ to continue after CTRL+C
sleep 0.25
# Create bulk msg dispatcher 
tmux new-window -n "BULK_MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/bulk_msg_dispatcher && cargo run; exec bash"
sleep 0.5

# ## Create the hanlders and other obc fsw components (coms handler, dfgm handler )
tmux new-window -n "DFGM_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin dfgm_handler; exec bash"
tmux new-window -n "COMS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin coms_handler; exec bash"
sleep 0.25


# ## Create the msg dispatcher (first component of the obc fsw because it creates ipc servers 
tmux new-window -n "CMD_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/cmd_dispatcher && cargo run --bin cmd_dispatcher; exec bash"
sleep 0.25


## Launch the GS simulation (this can just be a tcp client for now )
tmux new-window -n "SIM_GS" -- "trap : SIGINT; cd ../ && cargo run --bin cli_ground_station; exec bash"

tmux attach-session -t "downlink_payload_data"
