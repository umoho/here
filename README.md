# Here

It's a simple RESTful API web app, which can post the information (IPv4, IPv6) to the server, and user can check the IP of the client.

## Usage

Use `cargo` to build.

```bash
# Make a directory
mkdir here

# And go inside.
cd here

# Build the release version
cargo build --release

# Run the `server` part
cargo run --release --bin server

# Run the `client` part
cargo run --release --bin client

# And you can open the features for more printing. Example:
cargo run --release --bin client --features debug-printing
```

And then you can copy the `server` and `client` binaries.

When the server run, it will put a database file at the present working directory.

## Configuration

We use TOML.

When first run, you should input the config information at the cmdline,
just follow the tips.

The config file will be put at the present working directory.

The config of the server seems like:

```toml
# Example: bind = "0.0.0.0:8080"
bind = "<Address>"
```

The config of the client seems like:

```toml
# Example: account = "umoho"
account = "<Your Account>"
# Example: passwd = "password"
passwd = "<Your Password, Optional>"
# Example: api_url = "http://localhost:8080/here"
api_url = "<The API URL>"
```
