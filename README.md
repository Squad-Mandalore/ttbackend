# TTBackend

## Prerequisites

### Rustup: the Rust installer and version management tool

The primary way that folks install Rust is through a tool called Rustup, which is a Rust installer and version management tool.

Follow the instructions on [rust-lang](https://www.rust-lang.org/learn/get-started) or [rustup](https://rustup.rs/)

### Installing toolchain

- ```rustup default stable```
This command installes the latest stable toolchain for your native target triplet

### Installing sqlx cli

With Rust toolchain
- ```$ cargo install sqlx-cli```

With pacman (on Arch)
- ```$ pacman -Sy sqlx-cli```

### Database: Postgres

Install postgres on your system and create a database.
The application uses dotenvy to get the environment varaibles.

To add your database url do (never push your .env file):

```
echo DATABASE_URL=postgres://username:password@localhost/database_name > .env
```

```
sqlx database create
sqlx migrate run
```

### Other important variables
```
echo -e "JWT_SECRET=jwt_secret\nPEPPER=hatschuuuuu\nSALT_LENGTH=16\nKEYCHAIN_NUMBER=42" >> .env
```

## How to run

- ```cargo run```

## How to test

- ```cargo test```
- If you have tested too much and want to clear your ephemeral sqlx test databases run (dk if it runs on windows too, also dc):
  ``` psql -U postgres -d postgres -t -c "SELECT 'DROP DATABASE IF EXISTS ' || datname || ';' FROM pg_database WHERE datname LIKE '_sqlx_test_%';" | psql -U postgres -d postgres ```
