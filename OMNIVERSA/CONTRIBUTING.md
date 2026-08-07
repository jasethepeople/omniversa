# Contributing to OMNIVERSA

Thank you for your interest in contributing to the Multiphysics Synthetic Market Sovereign.

## Development Setup

```bash
git clone https://github.com/omniversa/omniversa.git
cd omniversa
cargo build --release
cargo test --workspace
```

## Code Standards

- All Rust code must pass `cargo fmt` and `cargo clippy -- -D warnings`
- All new features require tests
- All engine modifications require benchmark comparisons
- Documentation is mandatory for all public APIs

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request with detailed description

## Academic Contributions

We particularly welcome:
- Peer review of mathematical formulations
- Empirical validation studies against historical market data
- Improvements to physical analogies and their numerical implementations
- Visualization enhancements for the 4D manifold

## Security

For security disclosures, please email security@omniversa.io.
