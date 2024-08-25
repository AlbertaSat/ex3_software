#!/bin/bash

# Check if the correct number of arguments is provided
if [ "$#" -lt 1 ]; then
    echo "ERROR: Requires argument: <path to ex3_simulated_subsystems dir>"
    exit 1
fi

# Get the path to ex3_simulated_subsystems directory from the first argument
simulated_subsystems_dir="$1"

# Directory for UHF inside the ex3_simulated_subsystems directory
sim_uhf_dir="$simulated_subsystems_dir/UHF"

# Spawn terminal for simulated UHF
gnome-terminal --title="SIM_UHF" -- bash -c "cd $sim_uhf_dir; sleep 0.5; python3 simulated_uhf.py; exec bash"

# Spawn terminal for fake gs
gnome-terminal --title="GS" -- bash -c "cd $sim_uhf_dir; sleep 0.5; python3 generic_client.py 1235; exec bash"
