# Preflight

Preflight answers one question about a Solana program upgrade: **is it safe
to deploy?**

It does this by taking two builds of the same program, replaying an
identical sequence of transactions against both of them, and reporting
exactly where their behavior diverges: transactions that used to succeed
and now fail, transactions whose resulting account state changed, errors
that changed shape, and compute unit differences.

Preflight ships with a built-in reference program (a counter) in two
versions, so you get a full working report the moment you run it — no
setup, no external accounts, no API keys.

## Why this exists

Most Solana developers upgrade programs on the strength of a handful of
local tests and a read through the diff. Preflight gives you a systematic,
repeatable answer instead: record a transaction sequence, replay it
against two program binaries, diff the outcomes, and score the result —
the same way a database migration gets dry-run against a copy of
production data before it ships. It runs entirely on your machine, in a
local Solana execution environment, with no RPC endpoint, no external
validator, and no data ever leaving your computer.

## How it works

1. `examples/counter-old` is a native Solana program: a counter account
   with `Initialize`, `Increment`, `Decrement` and `SetValue`
   instructions.
2. `examples/counter-new` is the upgraded version, with real behavior
   changes: an upper bound was added to `Increment`/`SetValue`,
   `Decrement` now rejects an attempt to go below zero instead of
   clamping to it, and a couple of failure cases now return a custom
   error code instead of a generic one.
3. Preflight builds both with `cargo build-sbf`, generates a deterministic
   sequence of transactions (initialize, increment, decrement, an
   over-the-cap increment, a wrong-signer attempt, and so on), and
   persists that sequence — instructions and the keypairs used to sign
   them — to `transactions.json`.
4. It replays that exact sequence against `counter-old`, then again
   against `counter-new`, inside an in-process local Solana VM
   ([litesvm](https://github.com/LiteSVM/litesvm)). Nothing touches the
   network; there is no RPC endpoint, no external validator process, and
   no shared state between the two runs beyond the transaction sequence
   itself.
5. It diffs the two sets of results transaction by transaction and
   classifies each one: unchanged, compute units changed, behavior
   changed, error changed, new failure, or new success.
6. It renders `report.md` and `report.json`, plus a safety score out of
   100 that drops as more (and more serious) differences are found.

The bundled upgrade contains real, deliberate behavior changes, so running
it produces a genuine regression report, not a no-op diff.

## Architecture

```
preflight/
  crates/
    shared/       Plain data types: the transaction fixture format,
                   execution results, and report structures. No
                   dependency on Solana or litesvm.
    replay/       Turns a fixture into real transactions and executes
                   them against a program build inside litesvm, either
                   from a file path or from raw bytes.
    comparator/   Diffs two replay runs and classifies each transaction.
                   Pure logic, no I/O.
    report/       Renders a comparison into report.md and report.json.
    cli/          The `preflight` binary (`run` / `demo` subcommands),
                   plus a small library the server also depends on for
                   building the bundled programs and running the
                   replay + compare pipeline.
    server/       The `preflight-server` binary: an HTTP API (axum)
                   wrapping the same pipeline for the web dashboard.
  examples/
    counter-old/  Baseline reference program.
    counter-new/  Upgraded version with real behavior changes.
  client/         Web dashboard (Vite + React + TypeScript) that talks
                   to preflight-server.
```

Each crate has one job, and the dependency graph only points one way:
`cli` and `server` depend on `replay`, `comparator` and `report`; all of
those depend on `shared`; `shared` depends on nothing Solana-specific.
`comparator` and `report` never touch litesvm or the filesystem-heavy
parts of the pipeline, which makes them straightforward to test or reuse
on their own. `server` reuses `cli`'s library half (`preflight_cli::build`
and `preflight_cli::pipeline`) rather than re-implementing "build the
bundled programs" or "replay and compare" behind the HTTP API.

`examples/counter-old` and `examples/counter-new` are kept out of the
main Cargo workspace (each declares its own empty `[workspace]`). They
are built for the SBF target by `cargo build-sbf`, which bundles its own
toolchain, and isolating them keeps that build from having to touch the
much larger dependency graph the host tooling (`litesvm`, `clap`, and
friends) pulls in.

## Running it

### Prerequisites

- Rust and Cargo (stable).
- The Solana CLI, for `cargo build-sbf`. Verify with `solana --version`.

Preflight always passes `--tools-version v1.54` when it invokes
`cargo build-sbf` (see `crates/cli/src/build.rs`), downloading that
toolchain version on first use if it isn't already installed. This keeps
builds consistent regardless of which platform-tools version your
Solana CLI installed by default.

### The fastest way to see it work

```bash
cargo run -p preflight-cli -- demo
```

This builds both bundled programs, replays the transaction sequence
against each of them, and writes `transactions.json`, `report.md` and
`report.json` into `./preflight-out`. The first run takes a little longer
while `cargo build-sbf` downloads its toolchain and Cargo fetches
dependencies; after that it's fast.

### Running against your own program builds

Once you have two `.so` files:

```bash
cargo run -p preflight-cli -- run --old old.so --new new.so --out ./preflight-out
```

Preflight replays the same transaction sequence against whatever builds
you point it at. For the most direct results, use two builds of the
bundled counter program (or a program that shares its instruction and
account layout, described in `crates/replay/src/program_abi.rs`) — that's
exactly the `old.so`/`new.so` pair `preflight demo` produces for you.

## Running the dashboard

The dashboard is a Vite + React + TypeScript frontend (`client/`), styled
with Tailwind CSS and shadcn/ui and animated with Motion, talking to
`preflight-server`, an HTTP API (`crates/server`) around the same
replay/compare pipeline the CLI uses. It runs entirely on your machine.
The chosen palette and typography are documented in `brand.md` at the
repo root.

Start the API server (first request that needs the bundled programs will
build them, which takes a bit; after that it's cached):

```bash
cargo run -p preflight-server
```

In a second terminal, start the frontend:

```bash
cd client
npm install   # first time only
npm run dev
```

Open the URL Vite prints (typically `http://localhost:5173`). The dev
server proxies `/api/*` requests to `preflight-server` on port 8787 (see
`client/vite.config.ts`), so no CORS setup is needed. From there:

- **Run bundled demo** replays the sequence against the bundled
  `counter-old`/`counter-new` programs with no upload needed — the
  fastest way to see a full report.
- Uploading your own **old** and **new** `.so` files runs the same
  pipeline against them — use two builds of the bundled counter program,
  or a program with a matching instruction layout, for a direct result.

`crates/server` exposes three endpoints: `GET /api/health`,
`POST /api/demo`, and `POST /api/run` (multipart form fields `old` and
`new`), each returning the same JSON shape as `report.json`.

## Expected output

A terminal run of `preflight demo` ends with something like:

```
Summary
  total transactions:     9
  unchanged:              0
  compute units changed:  4
  behavior changed:       1
  error changed:          1
  new failures:           3
  new successes:          0

Safety score: 0/100
High risk - regressions detected. Do not deploy without further review.
```

A score of 0 here is the correct answer: the bundled upgrade contains
several deliberate behavior changes, and Preflight catches every one of
them. Running `preflight run --old old.so --new old.so` (the same program
against itself) instead scores 100 with nine unchanged transactions — a
useful sanity check that the tool only reports differences that are
actually there.

`report.md` explains each difference in plain language, for example:

```
### `decrement_past_zero` - new failure

Decrement by 1000 from 50. The old program clamps to 0; the new program
treats this as an underflow error.

Succeeded on the old program (value=0) but failed on the new program:
instruction 0 failed: custom program error 2 (Underflow).
```

`report.json` contains the same information — full logs, compute unit
counts, decoded account state, and the classification for every
transaction — in a structure meant for scripts rather than people.
