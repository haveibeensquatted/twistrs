# Twistr — [![Build & Test](https://github.com/JuxhinDB/twistrs/actions/workflows/rust.yml/badge.svg)](https://github.com/JuxhinDB/twistrs/actions/workflows/rust.yml) ![Status](https://img.shields.io/static/v1?label=Status&message=beta&color=orange) [![docs](https://docs.rs/twistrs/badge.svg)](https://docs.rs/twistrs/) ![crates.io](https://img.shields.io/crates/v/twistrs.svg) [![Discord invite](https://dcbadge.vercel.app/api/server/w8tksBQQq5?style=flat)](https://discord.gg/w8tksBQQq5)

<img align="left" width="20%" height="20%" src="res/logo-x1024.png">

> Twistr is a domain name permutation library powered by Rust. It aims to directly port the well-known [dnstwist](https://github.com/elceef/dnstwist) tool allowing for fast and flexible interfacing capabilities with the core libraries based on client's requirements.

<br/><br/><br/><br/><br/><br/>

## Quickstart

This is particularly helpful if you're from outside the Rust space and would like to get up and running quickly. 

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. `git clone https://github.com/JuxhinDB/twistrs.git`
3. `cd examples/twistrs-cli`
4. `cargo r -- github.com`

Keep in mind that this will not run with a release build and will be naturally slower, however it should allow you to explore some of the functionality.

## Usage

The core library is composed of the domain permutation module.

```rust
use twistrs::filter::Permissive;
use twistrs::permutate::Domain;

fn main() {
    let domain = Domain::new("google.com").unwrap();
    for permutation in domain.all(&Permissive) {
        println!("{} {:?}", permutation.domain.fqdn, permutation.kind);
    }
}
```

### Allocation-free API

For high-throughput use cases, use the visitor API to avoid allocating a new `String`/`Domain` per permutation:

```rust
use twistrs::filter::Permissive;
use twistrs::permutate::Domain;

fn main() {
    let domain = Domain::new("google.com").unwrap();
    domain.visit_all(&Permissive, |p| {
        println!("{} {:?}", p.domain.fqdn, p.kind);
    });
}
```

## Features

- Granular control over permutation algorithms (e.g. homoglyphs)
- Allocation-free visitor API (`Domain::visit_all`)
- Exceptionally fast end-to-end results
- Core library allowing easy extensions (i.e. CLI, API & streams)

#### Miscellaneous
- [x] [Blog post](https://blog.digital-horror.com/twistrs)
- [x] [HaveIBeenSquatted](https://haveibeensquatted.com/) 

---

## Frequently Asked Questions

Q: If I want to use a different set of dictionaries to the one provided out of the box by the libary, how can I achieve that?

A: Currently the library (for ease-of-use) bakes the dictionaries into the final binary through a build script. To customise this, you would need to update the relevant files under [`twistrs/data`](twistrs/data) and compile the library using `cargo b` or `cargo b --release`. You can also reference the library in your own Cargo.toml, pointing to a local copy.

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Twistrs by you, shall be licensed as MIT, without any additional
terms or conditions.
