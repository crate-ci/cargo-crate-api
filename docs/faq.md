## What about `semverver`?

[semverver](https://github.com/rust-lang/rust-semverver) uses a compiler driver to diff rlibs:
- This will thoroughly catch breaking changes for a given target and feature set.
- By being a compiler driver, it is coupled to a specific nightly toolchain (highly unstable APIs) and can be finicky to install.
- At the moment, this is limited to diffing rlibs and won't catch breaking changes in your `Cargo.toml`

## What about `cargo-breaking`?

[cargo-breaking](https://github.com/iomentum/cargo-breaking)'s current release
is built on the equivalent of
[cargo-expand](https://crates.io/crates/cargo-expand) and then parsing the
resulting output with [`syn`](docs.rs/syn):
- Requires recreating Rust's `pub` rules (at this time it is [severely lacking](Requires recreating all )).
- Technically, this is using nightly features behind the scenes but they are
  fairly static, being compatible with a wide range of nightly versions.
- It only supports different against a previous git revision and wipes away your working tree in doing so.

There is a branch to switch to rustc.  This seems like it'd make fairly similar to `semverer`.

## So how is `cargo-crate-api` different?

`cargo-crate-api` started in a
[conversation with cargo-breaking authors](https://github.com/iomentum/cargo-breaking/issues/40)
on alternative approaches.  The idea of building on top of `rustdoc -wjson`
came up.  `cargo-crate-api` started as a proof-of-concept for the idea.  Further
collaboration is still being determined.

Highlights:
- Leverages rustdoc for visibility logic
- Coupled to nightly for rustdoc json output but we are hoping this has a lower rate of churn than rustc APIs
- Can diff against git revisions (without destroying your working tree) and other paths
- Includes checking for breaking changes in `Cargo.toml`

## Why not report back a suggested version?

`cargo-breaking` can output what version to bump to.
[For now](https://github.com/epage/cargo-crate-api/issues/14) we forego this out of
concern for people exclusively relying on `cargo-crate-api` to determine whether
there are breaking changes, rather than using it as a safety to catch
unexpected breaking changes.
