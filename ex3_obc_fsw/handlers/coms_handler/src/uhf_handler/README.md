# UHF Handler

The UHF handler's purpose is to modify simulated operating parameters and get UHF data. Currently the UHF handler is not integrated with the rest of the OBC software.

## Usage

The UHF handler can be run using the test.sh script and the tests within the mod.rs file.

First cd into the uhf_handler directory:

``` bash
cd ex3_obc_fsw/handlers/coms_handler/uhf_handler
```

Next run the test.sh with your path to the simulated subsystems directory. (Assuming simulated subsystems directory is in the same level as you ex3_software directory)

``` bash
./test.sh ../../../../../../ex3_simulated_subsystems/
```

After running the test script with the correct arguments you will see two terminals gnome terminals spawn. One for the simulated UHF and one for the simulated gs (generic_client.py program). Now that the simulated subsystems are running you can try either one of the tests in the bottom of the mod.rs file.

__Note__: Be aware that if you run test_setting test function first you must close the terminals for simulated UHF and simulated groundstation if you want to run the test_getting function immediately afer. This is because the test_setting function modifies the parameters on the UHF to ones different then the default parameters the simulated UHF is initialized with. The test_getting function uses assert statements to check for default parameters of the UHF so this test will fail if you do not close the simulated subsystems terminals first and then rerun the script.

## UHF Parameters and data

As mentioned previously the UHF handler is not integrated with the rest of the ex3 OBC software. It can only be tested using the simulated UHF (Which is also not currently integrated with OBC software). In the future both the UHF ahndler and simulated UHF will be integrated with the OBC software.

For now the UHF handler modifies simulated parameters on the UHF. For now these are generalized parameters for the purpose of testing and are not representative of actual UHF parameters. The following are simulated parameters the simulated UHF currently has:

- __Beacon String__: This is the string that is sent down to the GS as a beacon. By default it is the string "beacon". This value can be changed by the UHF handler by using the SetBeacon opcode. You can get the UHF beacon string value by the UHF handler by using the GetBeacon opcode.
- __Mode__: This is the "Mode" of the UHF that refers to the frequency and baud rate of the UHF. As of now this parameter provides no real functionality and is a dummy value to be changed and read by the UHF handler. By default its value is 0, and can be any valid integer. Can be set using SetMode opcode, and get using GetMode opcode.
- __Baud Rate__: As of now this parameter provides no real functionality and is a dummy value to be changed and read by the UHF handler. By default its value is 9600, and can be any valid integer. Can be set using SetBaudRate opcode, and get using GetBaudRate opcode.
