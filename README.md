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

## Usage

Scripts to launch various processes (in seperate gnome-terminals) for testing and demonstration can be found in the [scripts](./scripts) directory.

TODO - Change these paths to be relative so they can be used by other members.

Gnome terminal cli commands are used in the scripts to launch a seperate terminal for each process of interest, with a defined title.

For example, to test the dfgm handler with a mock dispatcher (tcp server setup using netcat), and the simulated dfgm subsystem python program run the following command. (NOTE: You will have to modify the paths to match your local machines paths to find the simulated dfgm python program).

```@sh
bash dfgm_handler_test.sh
```

**When first cloning this repository**, you must enter the following:

```bash
cd ex3_shared_libs
git submodule update --init
```

This will allow the functions contained in that repo to be used for this code. The ex3_shared_libs directory is a git submodule containing libraries for various functions the satellites' processes use.

## Message Format

Below is the current format for how we will pass data through the OBC. It is subject to change as the project develops.

| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | Byte 5 | Byte 6 | Byte 127 |
| :----: | :----: | :----: | :----: | :----: | :----: | :----: | :----: |
| Length | MsgID  | DestID | SrcID  | Opcode | Data 0 | Data 1 | ... 0 |

The message as a whole is 128 bytes long and everything that comes after the Opcode is a vector of bytes.
