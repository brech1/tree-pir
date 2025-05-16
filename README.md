# Tree Private Information Retrieval

[![License][mit-badge]][mit-url]
[![Build][actions-badge]][actions-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/brech1/tree-pir/blob/master/LICENSE
[actions-badge]: https://github.com/brech1/tree-pir/actions/workflows/build.yml/badge.svg
[actions-url]: https://github.com/brech1/tree-pir/actions?query=branch%3Amaster

Benchmarking tool for evaluating PIR performance when fetching tree nodes without revealing queries to the database host. The primary focus is measuring the overhead of privately generating Merkle inclusion proofs for large trees.

The main assumption is that the client don't have the full tree data but still need to compute the Merkle path. So, you may only use this with trees with a deterministic structure for amount of leaves. If the full tree data is available, PIR is unnecessary.

Trees are being generated using the [Semaphore](https://github.com/semaphore-protocol/semaphore-rs) group module for Lean Incremental Merkle Trees, an append-only binary tree with the Poseidon hash function. Trees are automatically generated if not found.

Batch benchmarks retrieve `n` elements for trees with `2^n` leaves, corresponding to the elements needed for a Merkle proof in a LeanIMT (one element per level).

## Respire

The PIR scheme used in these benchmarks is Respire, from the paper [Respire: High-Rate PIR for Databases with Small Records](https://eprint.iacr.org/2024/1165). It was selected for:

- Efficient small-record retrieval
- Batch query support
- [Rust Implementation](https://github.com/AMACB/respire)

## Quickstart

Test PIR on a tree with `2^12` leaves using the `single_12` benchmark:

```bash
git clone https://github.com/brech1/tree-pir.git
cd tree-pir
cargo build --release
./target/release/tree-pir bench single 12
```

Results are saved in `./target/criterion/single_12/`. View them in a browser:

```bash
open target/criterion/report/index.html
```

## Prerequisites

- [Rust](https://rustup.rs/)

## Setup

Clone the repository and build the project:

```bash
git clone https://github.com/brech1/tree-pir.git
cd tree-pir
cargo build --release
```

The binary is located at `./target/release/tree-pir`.

## Tree Generation

Generate a LeanIMT with `2^n` leaves, saved as `./trees/tree_<exponent>.bin`:

```bash
./target/release/tree-pir gen-tree <exponent>
```

Example:

```bash
# Generate a tree with 2^12 (4096) leaves, 8191 total elements
./target/release/tree-pir gen-tree 12
```

Trees are automatically generated during benchmarking if they're not found.

## Running Benchmarks

Benchmarks measure PIR performance (setup time, query/answer sizes) for single or batch queries on trees with `2^n` leaves.

### Commands

Run all benchmarks:

```bash
./target/release/tree-pir bench all
```

Run single-element PIR benchmarks:

```bash
# Specific tree size
./target/release/tree-pir bench single <exponent>

# All supported tree sizes
./target/release/tree-pir bench single all
```

Run batch PIR benchmarks:

```bash
# Specific tree size
./target/release/tree-pir bench batch <exponent>

# All supported tree sizes
./target/release/tree-pir bench batch all
```

### Supported Benchmarks

| Benchmark   | Description                         |
| ----------- | ----------------------------------- |
| `single_12` | Single-element PIR, `2^12` leaves   |
| `single_16` | Single-element PIR, `2^16` leaves   |
| `single_20` | Single-element PIR, `2^20` leaves   |
| `single_24` | Single-element PIR, `2^24` leaves   |
| `batch_12`  | 12-element Batch PIR, `2^12` leaves |
| `batch_16`  | 16-element Batch PIR, `2^16` leaves |
| `batch_20`  | 20-element Batch PIR, `2^20` leaves |
| `batch_24`  | 24-element Batch PIR, `2^24` leaves |

## Results

Benchmark results are generated using Criterion and saved in:

```
target/criterion/
├── report/
│   ├── index.html
```
