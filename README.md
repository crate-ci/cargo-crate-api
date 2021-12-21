# cargo-api

> Interact with the crate's API

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/cargo-api.svg)
[![Crates Status](https://img.shields.io/crates/v/cargo-api.svg)](https://crates.io/crates/cargo-api)

## Documentation

- [Installation](#install)
- [Getting Started](#getting-started)
- [Reference](docs/reference.md)
- [FAQ](docs/faq.md)
- [Contribute](CONTRIBUTING.md)
- [CHANGELOG](CHANGELOG.md)

## Install

[Download](https://github.com/crate-ci/cargo-api/releases) a pre-built binary
(installable via [gh-install](https://github.com/crate-ci/gh-install)).

Or use rust to install:
```bash
cargo install cargo-api
```

## Getting Started

To diff your crate against the last tag, run
```bash
$ cargo api --diff
```
*(choose the git reference with `--git <REF>`)*

To help get started writing your `CHANGELOG.md`, run:
```bash
$ cargo api --diff --format md
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Crates.io]: https://crates.io/crates/cargo-api
[Documentation]: https://docs.rs/crate-api
