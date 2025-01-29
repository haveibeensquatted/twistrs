# CHANGELOG.md

## 0.7.1 (2025-01-29

Fix:

  - Add `.gov.co` to the list of accepted TLDs

## 0.7.0 (2024-12-12)

Fix:

  - Add `.edu.au` to the list of accepted TLDs

## 0.6.8 (2024-11-29)

Fix:

  - Fix bug with mapped permutations due to multiple mutations on same string

## 0.6.8 (2024-11-12)

Fix:

  - Fix bug where TLDs such as `.co.uk` were not included within the scope
    of the TLD permutation.

## 0.6.7 (2024-10-19)

Misc:

  - Added `Deserialize` and other auxiliary traits that may be helpful
    downstream

## 0.6.6 (2024-10-06)

Features:

  - Add `DoubleVowelInsertion` permutation method that inserts all ascii
    character in between two vowels, such as `ae` -> `ave`.


## 0.6.5 (2024-08-14)

Features:

  - Add `Mapped` permutation method that maps one or more characters into 
    another set of one or more characters that are similar, or easy to miss,
    such as `d` -> `cl`, `ck` -> `kk`.

## 0.6.4 (2024-08-13)

Fix:

  - Fixed bug where structurally valid domains were not being filtered due to 
    having an invalid/unmapped tld. `Domain` creation now makes sure that the
    domain is both structurally valid, and that the tld is a valid public tld.

## 0.6.2 (2023-10-11)

Fix:

  - Remove over-aggressive domain filtering to support punycode/homoglpyh
    domains better;
  - Fix issue with short domains (e.g., `ox.ac.uk`) that result in zero
    permutation.

## 0.6.1 (2023-09-30)

Fix: 

  - Add `Serialize` trait to `Permutation` and `PermutationKind`

## 0.6.0 (2023-09-30)

### BREAKING

All permutation functions now return a new `Permutation` struct that
contains both the domain permutation as well as the kind of that 
was performed (`PermutationKind`).

Features:

  - Added `Permutation` and `PermutationKind` structs to to the 
    `permutate` module.

Fix:

  - No longer aggressively parsing domains internally causing tasks 
    to panic. Instead, domains are `.filter_map`'ed internally to 
    only keep valid domains.

## 0.5.2 (2023-07-30)

Fix:

  - Fixed bug with whois lookup which was not updating the result into the
    the correct field;
  - Fixed bug in geoip lookup to populate the results correctly

## 0.5.1 (2023-07-30)

Fix:

  - Published crate with missing who-is servers and geoip data

## 0.5.0 (2023-07-16)

Fix:

  - (**BREAKING**): Reworked error handling entirely within the library 

## 0.4.1 (2022-07-24)

Security:

  - N/A

Features:
  
  - Added `Serialize` traits to key Permutation and Enrichment structs

Fix:

  - Pinned internal dependency to stable version that allows compilation

## 0.4.0-beta (2022-07-13)

Security:

  - Number of dependencies have been bumped to resolve security bugs;

Features:

  - N/A

Fix:

  - Bumped up Hyper to allow use of Tokio 1.x.x runtimes

## 0.3.1-beta (2020-10-27)

Security:

  - N/A

Features:

  - Implemented WhoIs lookup.

Fix:

  - N/A

## 0.3.0-beta (2020-10-26)

Security:

  - N/A

Features:

  - N/A

Fix:

  - Updated interface for permutation module to return [`impl Iterator`](https://github.com/JuxhinDB/twistrs/pull/19) which is a breaking change.


## 0.2.2-beta (2020-10-26)

Security:

  - N/A

Features:

  - Implemented new GeoIP cached lookups.

Fix:

  - N/A


## 0.2.1-beta (2020-10-17)

Security:

  - N/A

Features:

  - N/A

Fix:

  - Updated TLD permutation method to only perform TLD replacement due to causing noisy results.

## 0.2.0-beta (2020-10-17)

Security:

  - N/A

Features:

  - Implement HTTP Banner enrichment method.

Fix:

  - N/A

## 0.1.3-beta (2020-10-10)

Security:

  - N/A

Features:

  - Implement TLD permutation mode.
  - Implement Dictionary permutation mode.

Fix:

  - Added domain filtering to avoid looking up dirty/invalid domains.
