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