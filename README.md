# OrdinalFix: Fixing Compilation Errors via Shortest-Path CFL Reachability

## Introduction

This repository contains source code, datasets, and appendices for OrdinalFix.

See `appendix.pdf` for appendices, which contains details of the algorithm of OrdinalFix.

## Requirement

Rust build environment (see https://rustup.rs/).

## Build

```bash
cargo build --release
```

## Run

Uncompress dataset

```
tar -xzvf dataset.tar.gz
```

1. Middleweight Java

```bash
./target/release/fixing-rs-main fix --lang mj --max-len 10 --max-new-id 10 single --input <INPUT FILE> --env <ENV FILE> --output <OUTPUT FILE>
```

For example:

```bash
./target/release/fixing-rs-main fix --lang mj --max-len 10 --max-new-id 10 single --input ./dataset/mj/m_a_1/b4755a0130758afe1f2494d534c42a9093d8f2d6/block --env ./dataset/mj/m_a_1/b4755a0130758afe1f2494d534c42a9093d8f2d6/env --output ./output_mj
```

2. C

```bash
./target/release/fixing-rs-main fix --lang c --max-len 10 --max-new-id 10 single --input <INPUT FILE> --env <ENV FILE> --output <OUTPUT FILE>
```

For example:

```bash
./target/release/fixing-rs-main fix --lang c --max-len 10 --max-new-id 10 single --input ./dataset/c/prog00000_func0.block --env ./dataset/c/prog00000_func0.env --output ./output_c
```
