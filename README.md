# Merkle Tree PIR

A benchmarking suite to measure Private Information Retrieval (PIR) performance for fetching Merkle tree nodes without revealing queries to the database host. Main goal is to measure overhead when creating inclusion proofs privately.

For this specific use, PIR can be utilized only when working with trees with a deterministic structure based on leaf count. Our main assumption here is that we don't have the full tree data (e.g. dealing with large trees), so we would need to calculate our merkle path without it. If we were to have the full tree data, we don't need PIR.

Trees are built using the [Semaphore](https://github.com/semaphore-protocol/semaphore-rs) Lean Incremental Merkle Tree (append-only binary tree) with the Poseidon hash function. Trees will be generated automatically if not found.

## Quickstart

Try the `single_10` benchmark to test PIR on a tree with 2^10 leaves:

```bash
git clone https://github.com/brech1/mt-pir.git
cd mt-pir
cargo build --release
./target/release/mt-pir bench single 10
```

Results are in `./target/criterion/single_10/`. View them:

```bash
open target/criterion/report/index.html
```

## Setup

Install [Rust](https://rustup.rs/) and set up:

```bash
git clone https://github.com/brech1/mt-pir.git
cd mt-pir
cargo build --release
```

## Tree Generation

Generate a LeanIMT with 2^n leaves, saved to `./trees/tree_<exponent>.bin`:

```bash
./target/release/mt-pir gen-tree <exponent>
```

Example:

```bash
# Tree with 1024 (2^10) leaves
./target/release/mt-pir gen-tree 10
```

## Running Benchmarks

Benchmarks test PIR performance (setup time, query/answer sizes) for single or batch queries on trees with 2^n leaves. Results go to `./target/criterion/`. Trees are auto-generated if missing.

### Commands

Run all benchmarks:

```bash
./target/release/mt-pir bench all
```

Run single-query benchmarks:

```bash
# Specific tree size
./target/release/mt-pir bench single <exponent>

# All tree sizes
./target/release/mt-pir bench single all
```

Run batch-query benchmarks:

```bash
# Specific tree size
./target/release/mt-pir bench batch <exponent>

# All tree sizes
./target/release/mt-pir bench batch all
```

### Available Benchmarks

| Benchmark   | Description                     |
| ----------- | ------------------------------- |
| `single_10` | Single-element PIR, 2^10 leaves |
| `single_12` | Single-element PIR, 2^12 leaves |
| `single_14` | Single-element PIR, 2^14 leaves |
| `single_16` | Single-element PIR, 2^16 leaves |
| `single_20` | Single-element PIR, 2^20 leaves |
| `single_24` | Single-element PIR, 2^24 leaves |
| `batch_10`  | Batch PIR, 2^10 leaves          |
| `batch_12`  | Batch PIR, 2^12 leaves          |
| `batch_14`  | Batch PIR, 2^14 leaves          |
| `batch_16`  | Batch PIR, 2^16 leaves          |
| `batch_20`  | Batch PIR, 2^20 leaves          |
| `batch_24`  | Batch PIR, 2^24 leaves          |

## Reports

Explore performance metrics and graphs in Criterion reports:

```
target/
├── criterion/
│   ├── report/
│   │   ├── index.html
```

Open `index.html` in a browser for detailed results.
