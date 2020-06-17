# Twistr ![Status](https://img.shields.io/static/v1?label=Status&message=alpha&color=yellow) ![Rust](https://github.com/JuxhinDB/twistrs/workflows/Rust/badge.svg?branch=master)
---

<img align="left" width="20%" height="20%" src="res/logo-x1024.png">

> Twistr is a Domain name permutation and enumeration library powered by Rust & Rayon. It aims to directly port the well-known [dnstwist](https://github.com/elceef/dnstwist) tool while being much faster and allowing for much more flexible interfacing capabilities with the core libraries based on the client's requirements.

<br/><br/><br/><br/>

---

> This project is still a work-in-progress and the core library interface is bound to change soon. The following are a list of action items and features to implement before releasing an initial beta version.

## Features

- Granular control over Permutation engine and Data enrichment engine
  + Use specific permutation algorithms (e.g. homoglyphs)
  + Use specific data enrichment methods (e.g. DNS lookup)
- Concurrency out of the box
- Exceptionally fast end-to-end results
- Core library allowing easy extensions (i.e. CLI, API & streams)

#### Permutation Modes

- Addition ✅
- Bit Squatting ✅
- Homoglyph ✅
- Hyphenation ✅
- Insertion ✅
- Omission ✅
- Repetition ✅
- Replacement ✅
- Sub-domain ✅
- Transposition ✅
- Vowel-swap ✅

#### Domain Enrichment Features

- DNS lookup ✅
- MX parsing ✅
- SMTP Banner ❌
- HTTP Banner ❌
- GeoIP Lookup (Cached) ❌
- WhoIs Lookup ❌

#### Miscellaneous

- Benchmarking ❌
- Concurrent ✅
- Blog post ❌
- Crates.io ❌

---

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tokio by you, shall be licensed as MIT, without any additional
terms or conditions.
