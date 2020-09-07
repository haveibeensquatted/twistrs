# Twistr ![Status](https://img.shields.io/static/v1?label=Status&message=beta&color=orange) ![Rust](https://github.com/JuxhinDB/twistrs/workflows/Rust/badge.svg?branch=master)

## [docs](https://docs.rs/twistrs)

<img align="left" width="20%" height="20%" src="res/logo-x1024.png">

> Twistr is a Domain name permutation and enumeration library powered by Rust. It aims to directly port the well-known [dnstwist](https://github.com/elceef/dnstwist) tool allowing for fast and flexible interfacing capabilities with the core libraries based on client's requirements.

<br/><br/><br/><br/><br/><br/>

## Usage

The core library is composed of the domain permutation module and the domain enrichment module that can be used individually or chained together.

The following is a boiled-down version of the [twistrs-cli example](examples/twistrs-cli/src/main.rs) that uses [tokio mpsc](https://docs.rs/tokio/0.2.22/tokio/sync/mpsc/index.html).

```rust
use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

use tokio::sync::mpsc;

let domain = Domain::new("google.com").unwrap();
let permutations = domain.all().unwrap();

let (tx, mut rx) = mpsc::channel(1000);

for permutation in permutations {
    let domain_metadata = DomainMetadata::new(permutation.clone());
    let mut tx = tx.clone();

    tokio::spawn(async move
        if let Err(_) = tx.send((i, v.clone(), domain_metadata.dns_resolvable().await)).await {
            println!("received dropped");
            return;
        }

        drop(tx);
    });

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

#### Domain Enrichment Features

- [x] DNS lookup
- [x] MX parsing
- [ ] SMTP Banner
- [ ] HTTP Banner
- [ ] GeoIP Lookup (Cached)
- [ ] WhoIs Lookup

#### Miscellaneous

- [ ] Benchmarking
- [x] Concurrent
- [ ] Blog post
- [x] [Crates.io](https://crates.io/crates/twistrs)

---

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Twistrs by you, shall be licensed as MIT, without any additional
terms or conditions.
