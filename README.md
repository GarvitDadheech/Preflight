# Preflight

Preflight answers one question about a Solana program upgrade: **is it safe
to deploy?**

It does this by taking two builds of the same program, replaying an
identical sequence of transactions against both of them, and reporting
exactly where their behavior diverges: transactions that used to succeed
and now fail, transactions whose resulting account state changed, errors
that changed shape, and compute unit differences.

This repository is a proof of concept. It ships with its own small
example program (a counter) in two versions, so you can run the whole
workflow immediately without bringing your own program.

## Why this exists

Most Solana developers upgrade programs on the strength of a handful of
local tests and a read through the diff. There is rarely a systematic way
to ask "does this change alter how any transaction that has already
happened on-chain would behave?" The long-term idea behind Preflight is
to replay real historical transactions against a candidate upgrade before
it goes out, the same way a database migration gets dry-run against a
copy of production data.

This proof of concept does not attempt that yet. It proves the core
mechanic — record a transaction sequence, replay it against two program
binaries, diff the outcomes, score the result — end to end, on a program
small enough to read in one sitting.

## How it works

1. `examples/counter-old` is a small native Solana program: a counter
   account with `Initialize`, `Increment`, `Decrement` and `SetValue`
   instructions.
2. `examples/counter-new` is the same program with a handful of
   deliberate, realistic behavior changes: an upper bound was added to
   `Increment`/`SetValue`, `Decrement` now rejects an attempt to go below
   zero instead of clamping to it, and a couple of failure cases now
   return a custom error code instead of a generic one.
3. Preflight builds both with `cargo build-sbf`, generates a fixed
   sequence of example transactions (initialize, increment, decrement,
   an over-the-cap increment, a wrong-signer attempt, and so on), and
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

Running the bundled example produces genuine regressions to look at,
because `counter-new`'s changes are real behavior changes, not a no-op
diff.

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
                   building the example programs and running the
                   replay + compare pipeline.
    server/       The `preflight-server` binary: an HTTP API (axum)
                   wrapping the same pipeline for the web dashboard.
  examples/
    counter-old/  Baseline example program.
    counter-new/  Same program with intentional behavior changes.
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
example programs" or "replay and compare" behind the HTTP API.

`examples/counter-old` and `examples/counter-new` are deliberately kept
out of the main Cargo workspace (each declares its own empty
`[workspace]`). They are built for the SBF target by `cargo build-sbf`,
which bundles its own toolchain, and isolating them keeps that build
from having to touch the much larger dependency graph the host tooling
(`litesvm`, `clap`, and friends) pulls in.

## Running it

### Prerequisites

- Rust and Cargo (stable).
- The Solana CLI, for `cargo build-sbf`. Verify with `solana --version`.

`cargo build-sbf` downloads and manages its own compiler toolchain
(`platform-tools`) separately from your system Rust. As of this writing,
the version it selects automatically bundles a Rust compiler too old to
build one of the example program's dependencies. Preflight works around
this by always passing `--tools-version v1.54` when it invokes
`cargo build-sbf` (see `crates/cli/src/build.rs`), and downloads that
toolchain version on first use if it isn't already installed. If a
future platform-tools release moves past this, the `--tools-version`
flag on `preflight demo` can be changed or dropped.

### The fastest way to see it work

```bash
cargo run -p preflight-cli -- demo
```

This builds both example programs, replays the transaction fixture
against each of them, and writes `transactions.json`, `report.md` and
`report.json` into `./preflight-out`. The first run will take a little
longer while `cargo build-sbf` downloads its toolchain and Cargo fetches
dependencies.

### Running against your own program builds

Once you have two `.so` files:

```bash
cargo run -p preflight-cli -- run --old old.so --new new.so --out ./preflight-out
```

Note the scope of this proof of concept: `run` replays a transaction
fixture built specifically for the bundled counter program's instruction
layout (see `crates/replay/src/program_abi.rs`). Pointing it at an
unrelated program's `.so` file will build accounts and instructions it
doesn't understand and fail, rather than produce a meaningful diff. A
general-purpose version of this tool would need a way to describe an
arbitrary program's instructions and accounts (an Anchor IDL, for
example) instead of a hardcoded fixture.

## Running the dashboard

The dashboard is a Vite + React + TypeScript frontend (`client/`), styled with
Tailwind CSS and shadcn/ui and animated with Motion, talking to
`preflight-server`, a thin HTTP wrapper (`crates/server`) around the same
replay/compare pipeline the CLI uses. It runs entirely on your machine —
nothing is deployed anywhere by default. The chosen palette and typography
("Midnight Signal" + Geist, dark-mode-first) are documented in `brand.md` at
the repo root.

Start the API server (first request that needs the bundled example
programs will build them, which takes a bit; after that it's cached):

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
`client/vite.config.ts`), so no CORS setup is needed locally. From there:

- **Run bundled demo** replays the fixture against the bundled
  `counter-old`/`counter-new` example with no upload needed — the
  fastest way to see a real report.
- Uploading your own **old** and **new** `.so` files runs the same
  pipeline against them, subject to the same scope note as the CLI's
  `run` command above: they need to implement the bundled counter
  program's instruction layout to produce a meaningful result.

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

A score of 0 here is the correct answer, not a bug: the bundled example
is specifically built so several transactions behave differently between
versions. Running `preflight run --old old.so --new old.so` (the same
program against itself) will instead score 100 with nine unchanged
transactions, which is a useful sanity check that the tool isn't
reporting phantom differences.

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

## Limitations

This is a proof of concept, not a production regression tool. In
particular:

- The transaction fixture is fixed and specific to the bundled counter
  program. Nothing here replays real historical transactions, indexes
  chain history, or fetches anything from a live network — by design, as
  a POC scoped this way is much easier to reason about and verify.
- The "ABI" the replay engine uses to build instructions and decode
  account state is a hand-written copy of the example program's layout
  (`crates/replay/src/program_abi.rs`), not something derived from the
  program itself. A production version would need this to come from the
  target program (an IDL, a schema, or similar) instead of being
  hardcoded.
- The safety score is a simple fixed-weight heuristic (see
  `crates/comparator/src/lib.rs`), not a calibrated risk model.
- Everything runs inside a single in-process litesvm instance. This is
  intentional for a fast, deterministic, dependency-free POC, but it
  does not model things a real cluster does, such as cross-transaction
  timing, parallel execution, or genuine network conditions.
