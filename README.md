# ex3_obc_fsw
A cargo workspace for all communication and processes that run on the Ex-Alta 3 OBC.

A [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) is used for large Rust projects to consolidate multiple related packages into one location.

To build, run:

```bash
cargo build
```

in the project's root directory.

When the project is built, you can run:

```bash
cargo run --bin handler
```

to run the *handler* binary. You **DO NOT** need to specify the file path as cargo looks for the handler directory, and other nested directories, on its' own for this command.

You can also use:

```bash
cargo run --bin handler && cargo run --bin message_dispatcher
```

to run multiple binaries one after another. This can be done with any number of `cargo run`'s.

## Scheduler
Expects two lines of input from stdin:

1. The command number
2. The time of execution of the command in the format *YYYY-MM-DD HH:MM:SS*

***IMPORTANT:*** The scheduler expects time arguments in **UTC** to compare the current time with the time of the command.

#### Example Command:
```bash
cargo run --bin scheduler
1
2024-06-22 22:34:50
```

where 1 is the command and the line below it is the execution time.

The scheduler will also make two new directories when it runs:

- A *scheduler_log* folder will contain logs that are made as part of the code.
- A *saved_commands* folder which will have the command saved in it's own timestamped file. As of now, if multiple commands are given, a new file will be created for each one. There is a rolling file system in place for this that holds 2 KB of files.