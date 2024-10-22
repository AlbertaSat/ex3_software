#!/usr/bin/bash
# Written by Kaaden RumanCam
# Summer 2024

# User must provide the path to the simulated subsystem directory on their machine as the first arg
PATH_TO_SIM_SUBS=$1

# Create a detached session using our config file to hold our windows
tmux -f .tmux.conf new-session -d -s "test_shell_handler"

# Create bulk msg dispatcher 
tmux new-window -n "BULK_MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/bulk_msg_dispatcher && cargo run; exec bash"
sleep 0.25

tmux new-window -n "COMS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin coms_handler; exec bash"
sleep 0.25

## Launch the shell command handler
tmux new-window -n "SHELL_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin shell_handler; exec bash"
sleep 0.25

## Create the msg dispatcher (last component of the obc fsw because it creates ipc clients
tmux new-window -n "CMD_DISPATCHER" -- "trap : SIGINT; cd ../ && cargo run --bin cmd_dispatcher; exec bash"
sleep 0.25

## Launch the GS simulation (this can just be a tcp server for now )
tmux new-window -n "SIM_GS" -- "trap : SIGINT; cd ../ && cargo run --bin cli_ground_station; exec bash"

tmux attach-session -t "test_shell_handler"
