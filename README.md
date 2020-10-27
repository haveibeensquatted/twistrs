# Twistr — ![Rust](https://github.com/JuxhinDB/twistrs/workflows/Rust/badge.svg?branch=master) ![Status](https://img.shields.io/static/v1?label=Status&message=beta&color=orange) [![docs](https://docs.rs/twistrs/badge.svg)](https://docs.rs/twistrs/) ![crates.io](https://img.shields.io/crates/v/twistrs.svg) 

<img align="left" width="20%" height="20%" src="res/logo-x1024.png">

> Twistr is a Domain name permutation and enumeration library powered by Rust. It aims to directly port the well-known [dnstwist](https://github.com/elceef/dnstwist) tool allowing for fast and flexible interfacing capabilities with the core libraries based on client's requirements.

<br/><br/><br/><br/><br/><br/>

## Quickstart

This is particularly helpful if you're from outside the Rust space and would like to get up and running quickly. 

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. `git clone https://github.com/JuxhinDB/twistrs.git`
3. `cd examples/twistrs-cli`
4. `cargo r -- github.com`

Keep in mind that this will not run with a release build and will be naturally slower, however it should allow you to explore some of the functionality.

## Usage

The core library is composed of the domain permutation module and the domain enrichment module that can be used individually or chained together.

The following is a boiled-down version of the [twistrs-cli example](examples/twistrs-cli/src/main.rs) that uses [tokio mpsc](https://docs.rs/tokio/0.2.22/tokio/sync/mpsc/index.html).

```rust
use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let domain = Domain::new("google.com").unwrap();
    let permutations = domain.all().unwrap();

    let (tx, mut rx) = mpsc::channel(1000);

    for permutation in permutations {
        let domain_metadata = DomainMetadata::new(permutation.clone());
        let mut tx = tx.clone();

        tokio::spawn(async move
            if let Err(_) = tx.send((permutation.clone(), domain_metadata.dns_resolvable().await)).await {
                println!("received dropped");
                return;
            }

            drop(tx);
        });
    }

    drop(tx);

    while let Some(i) = rx.recv().await {
        println!("{:?}", i);
    }
}
```

## Features

- Granular control over Permutation or Enrichment modules
  + Use specific permutation algorithms (e.g. homoglyphs)
  + Use specific data enrichment methods (e.g. DNS lookup)
- Concurrency out of the box
- Exceptionally fast end-to-end results
- Core library allowing easy extensions (i.e. CLI, API & streams)

#### Permutation Modes

- [x] Addition
- [x] Bit Squatting
- [x] Homoglyph
- [x] Hyphenation
- [x] Insertion
- [x] Omission
- [x] Repetition
- [x] Replacement
- [x] Sub-domain
- [x] Transposition
- [x] Vowel-swap
- [x] Dictionary
- [x] TLD addition

#### Domain Enrichment Features

- [x] DNS lookup
- [x] MX parsing
- [ ] SMTP Banner
- [x] HTTP Banner
- [x] GeoIP Lookup (Cached)
- [x] WhoIs Lookup

#### Miscellaneous

- [x] Benchmarking
- [x] Concurrent
- [x] [Blog post](https://blog.digital-horror.com/twistrs)
- [x] [Crates.io](https://crates.io/crates/twistrs)

---

## Frequently Asked Questions

Q: If I want to use a different set of dictionaries to the one provided out of the box by the libary, how can I achieve that?

A: Currently the library (for ease-of-use) bakes the dictionaries into the final binary through a build script. To customise this, you would need to update the [dictionary files](./twistrs/dictionaries/) and compile the library using `cargo b` or `cargo b --release`. You can also reference the library in your own Cargo.toml, pointing to a local copy.

Q: How does the cached GeoIP lookup work?

A: Currently requires the client to supply their own [`maxminddb`](https://docs.rs/maxminddb/0.15.0/maxminddb/struct.Reader.html) reader and dataset. Twistrs at this point in time
is mostly an auxillliary wrapper to streamline processing of the DomainMetadata results.

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Twistrs by you, shall be licensed as MIT, without any additional
terms or conditions.
