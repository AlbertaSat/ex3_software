#!/usr/bin/bash
# Written by Kaaden RumanCam
# Summer 2024

# User must provide the path to the simulated subsystem directory on their machine as the first arg
PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

# Create a detached session using our config file to hold our windows
tmux -f .tmux.conf new-session -d -s "test_shell_handler"

tmux new-window -n "SIM_UHF" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/UHF && python3 ./simulated_uhf.py; exec bash"
#                                 ^ trap to continue after CTRL+C

## Create the msg dispatcher (first component of the obc fsw because it creates ipc servers 
tmux new-window -n "MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/msg_dispatcher && make && ./msg_dispatcher; exec bash"
sleep 0.25

# Create bulk msg dispatcher 
tmux new-window -n "BULK_MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/bulk_msg_dispatcher && cargo run; exec bash"
sleep 0.25

## Launch the shell command handler
tmux new-window -n "SHELL_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin shell_handler; exec bash"
tmux new-window -n "COMS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin coms_handler; exec bash"

## Launch the GS simulation (this can just be a tcp server for now )
tmux new-window -n "SIM_GS" -- "trap : SIGINT; cd ../ && cargo run --bin cli_ground_station; exec bash"

tmux attach-session -t "test_shell_handler"
