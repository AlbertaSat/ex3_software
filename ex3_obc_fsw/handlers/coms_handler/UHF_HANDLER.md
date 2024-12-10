# UHF Handler

The UHF handler's purpose is to modify simulated operating parameters and get UHF data. Currently the UHF handler is not integrated with the rest of the OBC software.

## Usage

To use the UHF handler first start by running the "uplink_command_msg.sh" in the scripts directory.

Assuming the ex3_simulated_subsystems directory is in the same level as the ex3_software directory:

``` bash
cd scripts
./uplink_command_msg.sh ../../ex3_simulated_subsystems
```

Next, focus into the terminal labelled "SIM GS" and send a command to the UHF. Other UHF commands can be found [here](https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0).

This command sets the beacon value to "BEACON" by using opcode "4" a string of characters:

``` text
UHF 4 BEACON
```

You can observe if the beacon is was set correctly by checking the output in the "COMS_HANDLER" terminal or the "SIM_GS" terminal.

