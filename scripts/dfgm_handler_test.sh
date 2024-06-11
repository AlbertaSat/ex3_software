#!/bin/bash

#Passing args to gnome-terminal
#gnome-terminal -e sh -c 'some commands here' sh "$variable1" "$variable2" "etc."

# Test the DFGM Handler 

# name of worktree
worktree_name="dfgm_handler"

#Start DFGM sim 
gnome-terminal -t DFGM_sim_subsystem -- sh -c 'cd /home/devin/albertasat/ex3/sim_subs/ex3_simulated_subsystems/DFGM/ && python3 dfgm_subsystem.py; exec bash'

# Start mock dispatcher 
gnome-terminal -t MOCK_DISPATCHER -- sh -c 'nc -l 1900; exec bash'

# Create new terminal and start DFGM handler 
gnome-terminal -t DFMG_HANDLER -- sh -c 'cd /home/devin/albertasat/ex3/obc/dfgm_hander/target/debug/ && ./dfgm_handler; exec bash'