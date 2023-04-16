# J utilities startup

Set to run at the beginning of the terminal session. 
<br />
Or on demand for quick utilities.

  1. Get the weather
  1. Query an API
  1. Parse a Json

## How to Install 

Build the binary

```sh 
cd ja_init
cargo build
```

Copy the binary to your `.local/bin`. Also make sure it is in your path. 

### Alternative

Simply run it from the current folder

```sh
cargo run
```

> Note: To run with args do (ie.) `cargo run -- -h`

## How to use

For help and up-to-date instructions

```sh
ja_init -h
```

The desired way to run it. 

  1. Set a secret token 
  1. Run the app with the desired city you want to query

```sh
export J_BEARER="<some_secret_bearer>"
ja_init -c "Bogota"
```

Set an alias for the execution, so it is simpler to run.
