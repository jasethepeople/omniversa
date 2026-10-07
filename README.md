# OMNIVERSA — The Multiphysics Synthetic Market Sovereign

A Rust market-thermodynamics engine that models financial markets as a dissipative physical system across six coupled physics engines, fused into a single 0.0–1.0 **fragility index**.

## Features

- **Six physics engines** (per the repo's docs): liquidity CFD (viscosity), gravity-whale (N-body capital dynamics), cascade Hawkes (self-exciting point processes), neural panic (Hodgkin-Huxley excitation), narrative NLP (sentiment pressure), Bayesian risk (observer fusion)
- **Unified Graph Topology (UGT)** — shared market-state graph the engines operate on
- **Fragility index** — scalar 0.0 (stable) to 1.0 (fracture imminent) computed by the core orchestrator
- **WebGPU visualization** — 4D manifold / liquidity-field rendering crate
- **gRPC/protobuf contracts** — service definitions under `proto/`
- Integration test suite, benchmarks, and config defaults shipped

## Tech stack

- **Language:** Rust (edition 2021, rust 1.78+), 21 source files in a Cargo workspace
- **Key crates:** tokio, rayon, nalgebra, ndarray, faer, wgpu, serde, arrow-family (per workspace deps)
- **Infra:** Docker Compose, Terraform (under `infra/`), GitHub Actions CI + release workflows
- **Docs:** `docs/API.md`, `docs/ARCHITECTURE.md`, `docs/DEPLOYMENT.md`

## Getting started

```bash
make build        # cargo build --release
make test         # cargo test --workspace
make docker       # docker-compose -f infra/docker/docker-compose.yml up --build
```

Full build/test/lint/bench targets are listed in the `Makefile`.

## Project structure

```
OMNIVERSA/                 # the actual project lives in this subdirectory
├── crates/
│   ├── omniversa-core/    # orchestrator: manifold, topology, fragility
│   ├── liquidity-cfd/     # Navier-Stokes viscosity layer
│   ├── gravity-whale/     # N-body whale-capital layer
│   ├── cascade-hawkes/    # seismology / cascade layer
│   ├── neural-panic/      # biological excitation layer
│   ├── narrative-nlp/     # atmospheric sentiment layer
│   ├── bayesian-risk/     # HMC observer layer
│   ├── ugt-graph/         # unified graph topology
│   └── webgpu-viz/        # WebGPU manifold renderer
├── config/default.toml
├── proto/, tests/, benchmarks/, docs/
├── infra/docker/, infra/terraform/
└── scripts/, Makefile
```

## Status

Real, substantial codebase with CI, docs, and tests. **Authorship is not documented in this repo** — the workspace `Cargo.toml` and badges reference `github.com/omniversa/omniversa` and author `OMNIVERSA Sovereign Systems <sovereign@omniversa.io>`, while the crate-level code, Makefile, and integration tests are all present and self-consistent. License: AGPL-3.0. The repo's own docs describe this as research/prototype software; the "production-grade" and performance claims in its docs are unmeasured by anything in the repo (benchmarks exist but contain no published results).
