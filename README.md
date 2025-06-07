# crypto-scanner-agent

This project is a minimal Axum application that streams cryptocurrency gainers over WebSocket.
Incoming data is pulled from the Raydium WebSocket feed and filtered server side before being broadcast to any connected clients.
A small example client is served from the `static` directory.

```mermaid
flowchart TD
    A["Raydium WebSocket feed"] -->|"raw data"| B(crypto-scanner-agent)
    B -->|"filtered signals"| C["WebSocket clients"]
    B -->|"serves"| D["HTML client"]
```

## Architecture Overview

The server connects to Raydium using **tokio_tungstenite**, filters the incoming
messages, then broadcasts them over an Axum WebSocket route. A watch channel
acts as the bridge between the feed task and any connected clients.

```mermaid
flowchart TD
    A["Raydium WebSocket"] --> B["tokio_tungstenite stream"]
    B --> C["Signal filter"]
    C --> D["watch channel"]
    D --> E["Axum WebSocket handler"]
    E --> F["Browser clients"]
```

### Data Flow Details

```mermaid
sequenceDiagram
    participant Raydium
    participant Feeder as spawn_raydium_feed
    participant Watch as watch::channel
    participant Handler as websocket_handler
    participant Client

    Raydium-->>Feeder: trade updates
    Feeder->>Watch: send(Message)
    Client->>Handler: connect
    Handler->>Watch: subscribe
    Watch-->>Handler: broadcast Message
    Handler-->>Client: transmit JSON signal
```

## Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install).
- (Optional) Install the Shuttle CLI if you want to use Shuttle's local runner: `cargo install cargo-shuttle`.

## API Keys

The Raydium stream used here is public, so **no API keys are required**. The application works out of the box without further configuration.

### Secrets

Some binaries read configuration from environment variables supplied via Shuttle
secrets. To use the `raydium_cli` helper without passing a wallet address each
time, add an `OWNER` entry to your `Secrets.toml`:

```toml
OWNER = "YOUR_SOLANA_ADDRESS"
```

When present, the `balances` command will default to this value if no owner is
specified on the command line.

## Running the Server

1. Clone this repository and change into its directory:
   ```bash
   git clone <this-repo-url>
   cd crypto-scanner-agent
   ```
2. Build and start the service with Cargo:
   ```bash
   cargo run --release
   ```
   By default the server listens on `127.0.0.1:8000`. It exposes a WebSocket endpoint at `/websocket`, a version endpoint at `/version`, and serves a basic HTML client at the root path.
   If you see a `TlsFeatureNotEnabled` error, ensure the `rustls-tls-webpki-roots` feature for `tokio-tungstenite` is enabled in `Cargo.toml`.
3. Visit `http://localhost:8000/` in your browser to see the live feed. Each message shows a coin symbol and volume information whenever the 24h price increase exceeds 5% and the quote volume is above $1M.

### Running with Shuttle

If you have the Shuttle CLI installed, you can alternatively run
```bash
cargo shuttle run
```
which launches the application inside Shuttle's runtime. This mirrors how the service would run when deployed through Shuttle.

### Helper Scripts

For convenience, use the provided shell scripts to run or deploy via Shuttle.

```bash
./run.sh
```
Runs the service locally using `shuttle run` and the `Secrets.toml` file in the repository root. The script automatically opens `http://localhost:8000/` in your default browser, loading the web UI served from `static/index.html`. It also clears any existing log files under `logs/` before starting the server so each run begins with fresh logs.

```bash
./deploy.sh
```
Deploys the application to Shuttle using the same secrets file.

## Formatting and Linting

Rust code in this repository follows the standard formatting and linting tools provided by Cargo.

```bash
./format.sh  # runs `cargo fmt --all`
./lint.sh    # runs `cargo clippy --all-targets --all-features -- -D warnings`
```

You can also invoke them directly:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Running Tests

Execute the unit tests with Cargo:

```bash
cargo test
```

## Examples

Two optional binaries demonstrate how to call DeepSeek outside of the
crypto gainer service:

- `sentiment` showcases running several extraction agents in parallel.
- `calculator` illustrates using DeepSeek tools for simple arithmetic.
- `nautilus_example` demonstrates a placeholder integration with
  [Nautilus Trader](https://github.com/nautilus-trader/nautilus-trader). Enable
  the `nautilus` feature to compile this binary.

When issuing multiple DeepSeek requests, the examples leverage
`futures::stream::iter` with `buffer_unordered` to run calls concurrently.
Successful responses are cached so repeated runs avoid unnecessary network
traffic.

## Canonical Cargo Commands

The helper scripts above are optional. You can perform the same tasks using
standard Cargo commands:

```bash
# start the server
cargo run --release

# run the sentiment example
cargo run --bin sentiment --release


# run the calculator example (optional)
cargo run --bin calculator --release

# run the Nautilus Trader example (optional)
cargo run --bin nautilus_example --features nautilus --release

# deploy with Shuttle
cargo shuttle deploy -- --secrets backend/Secrets.toml
```

