# CHANGELOG.md

## 1.0.0 (2025-01-11)

After over five years, we've finally gone about releasing our 1.0 release. This release shapes
the library into a state that we feels meets all the needs we, and the community, have, while 
also trimming some of the lingering issues we had in the past.

The following is a breakdown of the notable changes:

* :warning: The `enrich` module has been removed entirely. This is a breaking change, as it will
  require consumers to implement their own enrichment, and re-introduce certain structs if
  necessary.
* Introduced a zero-allocation API for generating permutations. Ideal for environments with very
  tight performance budgets.
* Introduced a streaming version of the API such that permutations can be streamed to the called.
  This is a very niche feature, but one that's particularly important for [Have I Been Squatted](https://haveibeensquatted.com/).

## 0.9.4 (2025-12-26)

Misc:
  - Update TLDs
  - (**Note** internal purposes): Added `CertificateTransparency` variant to `PermutationKind`

## 0.9.3 (2025-10-12)

Misc:
  - Significantly extended keyword list to include shopping/brands/event related keywords

## 0.9.2 (2025-09-24)

Misc:
  - Significantly extended keyword list

## 0.9.1 (2025-08-07)

Fix:
  - Apply ceiling limit to `vowel_shuffling` permutation

## 0.9.0 (2025-08-06)

> :warning:
> Minor breaking change: `hyphentation` had a typo and has now been renamed to `hyphenation`

Feat:
  - Added the `vowel_shuffle` permutation type which is a superset of the `vowel_swap` 
    permutation type.
  - Added the `hyphentation_tld_boundary` permutation type which is an edge-case on top
    of the `hyphenation` permutation type.
  
Misc:
  - Fixed typo in `hyphentation` function name (removed excess `t`).

## 0.8.4 (2025-07-31)

Misc:

  - Updated keywords to include banking and finance related keywords

## 0.8.3 (2025-06-20)

Misc:

  - Updated TLD list
  - Added `platform` to keyword list

## 0.8.2 (2025-05-23)

Feat:

  - Added `Domain::raw` which reduces TLD validation on domains
  - Updated TLD list

## 0.8.0 (2025-04-15)

Feat:

  - :warning: **breaking**: Added the `Filter` trait to allow more control
    over which permutations are created
  - Updated TLD list
  - Updated keyword list to include more operational keywords

## 0.7.5 (2025-03-06)

Feat:

  - Exposing global TLDs list to consumers
  - Added `emprende.ve` tld
  - Removed `kerrylogistics` and `lipsy` tlds

## 0.7.4 (2025-02-17)

Feat:

  - Added additional keywords including regional, geographical and generic.

## 0.7.3 (2025-02-04)

Fix:

  - Updated accepted TLDs to not include private domains

Misc:

  - Updated automatic PSL list parser

## 0.7.2 (2025-02-04)

> [!WARNING]  
> This release has been yanked due to containing private domains from the 
> public suffix list that resulted in inflated results.

Fix: 

  - Updated all public suffix list.

Misc: 

  - Added github workflow to update TLDs daily


## 0.7.1 (2025-01-29)

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
