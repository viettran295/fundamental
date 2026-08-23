# fundamental

A backend service for [vTrade](https://github.com/viettran295/vTrade) that fetches raw, up-to-date financial statement data directly from the U.S. Securities and Exchange Commission (SEC) for fundamental analysis.

## Overview

`fundamental` provides data for fundamental analysis. It connects to the SEC's EDGAR data feeds, downloads and parses financial filings (income statements, balance sheets, cash flow statements) and exposes the data through a HTTP API. Calculate industry average of financial ratios, results are cached in [DragonflyDB](https://www.dragonflydb.io/) (a Redis-compatible in-memory store) to minimize repeated heavy CPU load.

## Features

- **Live SEC data** — pulls the latest XBRL financial data directly from EDGAR
- **Industry average of financial ratios** — Calculate industry average of liquidity ratios, solvency ratios, profitability ratios
- **Scheduled ingestion** — cron-based job runner keeps data fresh automatically

## Getting Started

### Running with Docker Compose (recommended)

```bash
git clone https://github.com/viettran295/fundamental.git
cd fundamental

docker compose up -d
```

Data is persisted to `./data` on the host.

### Running locally (without Docker)

1. Start a Redis-compatible server (e.g. DragonflyDB or Redis) on `localhost:6379`.

2. Create a `.env` file in the project root:

```env
CACHE_DB_URI=redis://localhost:6379
RUST_LOG=debug
```

3. Build and run:

```bash
cargo build --release
./target/release/fundamental
```

The server will be available at `http://localhost:8001`.

## Project Structure

```
fundamental/
├── src/                  # Rust source code
├── tests/                # Integration and unit tests
├── .github/workflows/    # CI/CD pipelines
├── Cargo.toml            # Workspace dependencies
├── Dockerfile            # Multi-stage Docker image
└── docker-compose.yml    # Local orchestration
```

## Configuration

| Environment Variable | Description | Default |
|---|---|---|
| `CACHE_DB_URI` | Connection URI for DragonflyDB / Redis | `redis://localhost:6379` |

## Running Tests

```bash
# Unit + integration tests (requires Docker for testcontainers)
cargo test
```

[testcontainers](https://docs.rs/testcontainers) is used to spin up ephemeral DragonflyDB instances during integration tests, so no manual setup is required.


## Data Source

All financial data is sourced from the [SEC EDGAR](https://www.sec.gov/edgar/) public APIs. No API key or account is required. Please be mindful of the SEC's [fair access policy](https://www.sec.gov/os/accessing-edgar-data) and avoid sending excessive requests.
