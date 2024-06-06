# Scheduler
Expects two lines of input from stdin:

1. The command number
2. The time of execution of the command in the format *YYYY-MM-DD HH:MM:SS*

***IMPORTANT:*** The scheduler expects time arguments in **UTC** to compare the current time with the time of the command.

### Example Command:
```bash
cargo run --bin scheduler
1
2024-06-22 22:34:50
```

where 1 is the command and the line below it is the execution time.

The scheduler will also make two new directories when it runs:

- A *scheduler_log* folder will contain logs that are made as part of the code.
- A *saved_commands* folder which will have the command saved in it's own timestamped file. As of now, if multiple commands are given, a new file will be created for each one. There is a rolling file system in place for this that holds 2 KB of files.

The scheduler will "run" a command once it receives another command and passes through a check of the saved_commands directory again. Any command that is stored there whose time has passed or is now will be executed.