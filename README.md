# TTBackend

## Rustup: the Rust installer and version management tool

The primary way that folks install Rust is through a tool called Rustup, which is a Rust installer and version management tool.

Follow the instructions on [rust-lang](https://www.rust-lang.org/learn/get-started) or [rustup](https://rustup.rs/)

### Installing toolchain

- ```rustup default stable```
This command installes the latest stable toolchain for your native target triplet

## How to run

- ```cargo run```

## How to test

- ```cargo test```

## How create migrations

- ```cargo install sqlx-cli```

- ```sqlx migrate add create_example_table```
This command creates a .sql file in the migrations folder which will automatically migrated by start of the application
