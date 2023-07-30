# CHANGELOG.md

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
