# jemalloc-ctl
[![CircleCI](https://circleci.com/gh/sfackler/jemalloc-ctl.svg?style=shield)](https://circleci.com/gh/sfackler/jemalloc-ctl)

[Documentation](https://docs.rs/jemalloc-ctl)

jemalloc control and introspection.

## Example

```rust
use std::thread;
use std::time::Duration;

loop {
    // many statistics are cached and only updated when the epoch is advanced.
    jemalloc_ctl::epoch().unwrap();

    let allocated = jemalloc_ctl::stats::allocated().unwrap();
    let resident = jemalloc_ctl::stats::resident().unwrap();
    println!("{} bytes allocated/{} bytes resident", allocated, resident);
    thread::sleep(Duration::from_secs(10));
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
