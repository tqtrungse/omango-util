# Omango-Util

This is an utilities crate.<br />

- [Hint](src/hint.rs): support `likely` and `unlikely`.
- [Backoff](src/backoff.rs): retry with support reduce CPU cache coherence.
- [CachePadded](src/cache_padded.rs): aligns CPU cache line for types.
- [RwSpinLock](src/lock.rs): concurrent thread safe read-write locking for the fast changes.
- [Defer](src/defer.rs): support `Golang's defer`.

## Table of Contents

- [Usage](#usage)
- [Compatibility](#compatibility)
- [License](#license)

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
omango-util = "0.1.6"
```

## Compatibility

The minimum supported Rust version is `1.57`.

## License

The crate is licensed under the terms of the MIT
license. See [LICENSE](LICENSE) for more information.

#### Third party software

This product includes copies and modifications of software developed by third parties:

* [src/backoff.rs](src/backoff.rs) includes copies and modifications of code from Crossbeam-Utils,
  licensed under the MIT License and the Apache License, Version 2.0.

* [src/cache_padded.rs](src/cache_padded.rs) includes copies and modifications of code from Crossbeam-Utils,
  licensed under the MIT License and the Apache License, Version 2.0.

See the source code files for more details.

The third party licenses can be found in [here](https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-utils#LICENSE).
