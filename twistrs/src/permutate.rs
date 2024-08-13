//! The permutation module exposes functionality around generating
//! multiple valid variations of a given domain. Note that this
//! module is _only_ concerned with generating possible permutations
//! of a given domain.
//!
//! For details on how to validate whether domains are actively used,
//! please see `enrich.rs`.
//!
//! Example:
//!
//! ```
//! use twistrs::permutate::{Domain, Permutation};
//!
//! let domain = Domain::new("google.com").unwrap();
//! let domain_permutations: Vec<Permutation> = domain.all().expect("error permuting domains").collect();
//! ```
//!
//! Additionally the permutation module can be used independently
//! from the enrichment module.
use crate::constants::{ASCII_LOWER, HOMOGLYPHS, KEYBOARD_LAYOUTS, VOWELS};
use crate::error::Error;

use std::collections::HashSet;

use addr::parser::DomainName;
use addr::psl::List;
use itertools::Itertools;
use serde::Serialize;

// Include further constants such as dictionaries that are
// generated during compile time.
include!(concat!(env!("OUT_DIR"), "/data.rs"));

use crate::tlds::{TLDS, TLDS_EXTENDED};

/// Wrapper around an FQDN to perform permutations against.
#[derive(Clone, Hash, Default, Debug, Serialize, Eq, PartialEq)]
pub struct Domain {
    /// The domain FQDN to generate permutations from.
    pub fqdn: String,

    /// The top-level domain of the FQDN (e.g. `.com`).
    pub tld: String,

    /// The remainder of the domain (e.g. `google`).
    pub domain: String,
}

#[derive(Clone, Debug, Serialize, Hash, Eq, PartialEq)]
pub struct Permutation {
    pub domain: Domain,
    pub kind: PermutationKind,
}

#[derive(Clone, Copy, Serialize, Hash, Debug, Eq, PartialEq)]
pub enum PermutationKind {
    Addition,
    Bitsquatting,
    Hyphenation,
    Insertion,
    Omission,
    Repetition,
    Replacement,
    Subdomain,
    Transposition,
    VowelSwap,
    Keyword,
    Tld,
    Homoglyph,
}

#[derive(Clone, thiserror::Error, Debug)]
pub enum PermutationError {
    #[error("invalid domain name, (expected {expected:?}, found {found:?})")]
    InvalidDomain { expected: String, found: String },

    #[error("error generating homoglyph permutation (domain {domain:?}, homoglyph {homoglyph:?})")]
    InvalidHomoglyph { domain: String, homoglyph: String },
}

impl Domain {
    /// Wrap a desired FQDN into a `Domain` container. Internally
    /// will perform additional operations to break the domain into
    /// one or more chunks to be used during domain permutations.
    pub fn new(fqdn: &str) -> Result<Domain, Error> {
        let parsed_domain =
            List.parse_domain_name(fqdn)
                .map_err(|_| PermutationError::InvalidDomain {
                    expected: "valid domain name that can be parsed".to_string(),
                    found: fqdn.to_string(),
                })?;
        let root_domain = parsed_domain
            .root()
            .ok_or(PermutationError::InvalidDomain {
                expected: "valid domain name with a root domain".to_string(),
                found: fqdn.to_string(),
            })?;

        let tld = parsed_domain.suffix().to_string();

        // Verify that the TLD is in the list of known TLDs, this requires that
        // the TLD data list is already ordered, otherwise the result of the
        // binary search is meaningless. We also assume that all TLDs generated
        // are lowercase already.
        if TLDS_EXTENDED.binary_search(&tld.as_str()).is_ok() {
            let domain = Domain {
                fqdn: fqdn.to_string(),
                tld,
                domain: root_domain
                    .find('.')
                    .and_then(|offset| root_domain.get(..offset))
                    // this should never error out since `root_domain` is a valid domain name
                    .ok_or(PermutationError::InvalidDomain {
                        expected: "valid domain name with a root domain".to_string(),
                        found: fqdn.to_string(),
                    })?
                    .to_string(),
            };

            Ok(domain)
        } else {
            let err = PermutationError::InvalidDomain {
                expected: "valid domain tld in the list of accepted tlds globally".to_string(),
                found: tld,
            };

            Err(err.into())
        }
    }

    /// Generate any and all possible domain permutations for a given `Domain`.
    ///
    /// Returns `Iterator<String>` with an iterator of domain permutations
    /// and includes the results of all other individual permutation methods.
    ///
    /// Any future permutations will also be included into this function call
    /// without any changes required from any client implementations.
    pub fn all(&self) -> Result<impl Iterator<Item = Permutation> + '_, Error> {
        Ok(self
            .addition()
            .chain(self.bitsquatting())
            .chain(self.hyphentation())
            .chain(self.insertion())
            .chain(self.omission())
            .chain(self.repetition())
            .chain(self.replacement())
            .chain(self.subdomain())
            .chain(self.transposition())
            .chain(self.vowel_swap())
            .chain(self.keyword())
            .chain(self.tld())
            .chain(self.homoglyph()?))
    }

    /// Add every ASCII lowercase character between the Domain
    /// (e.g. `google`) and top-level domain (e.g. `.com`).
    pub fn addition(&self) -> impl Iterator<Item = Permutation> + '_ {
        ASCII_LOWER.iter().filter_map(move |c| {
            let fqdn = format!("{}{}.{}", self.domain, c, self.tld);

            if let Ok(domain) = Domain::new(fqdn.as_str()) {
                Some(Permutation {
                    domain,
                    kind: PermutationKind::Addition,
                })
            } else {
                None
            }
        })
    }

    /// Following implementation takes inspiration from the following content:
    ///
    ///  - <`https://github.com/artemdinaburg/bitsquat-script/blob/master/bitsquat.py`>
    ///  - <`http://dinaburg.org/bitsquatting.html`>
    ///
    /// Go through each char in the domain and XOR it against 8 separate masks:
    ///
    ///  00000001 ^ chr
    ///  00000010 ^ chr
    ///  00000100 ^ chr
    ///  00001000 ^ chr
    ///  00010000 ^ chr
    ///  00100000 ^ chr
    ///  01000000 ^ chr
    ///  10000000 ^ chr
    ///
    /// Then check if the resulting bit operation falls within ASCII range.
    pub fn bitsquatting(&self) -> impl Iterator<Item = Permutation> + '_ {
        let permutations = self
            .fqdn
            .chars()
            .flat_map(move |c| {
                (0..8).filter_map(move |mask_index| {
                    let mask = 1 << mask_index;

                    // Can the below panic? Should we use a wider range (u32)?
                    let squatted_char: u8 = mask ^ (c as u8);

                    // Make sure we remain with ASCII range that we are happy with
                    if ((48..=57).contains(&squatted_char))
                        || ((97..=122).contains(&squatted_char))
                        || squatted_char == 45
                    {
                        Some((1..self.fqdn.len()).map(move |idx| {
                            let mut permutation = self.fqdn.to_string();
                            permutation.insert(idx, squatted_char as char);
                            permutation
                        }))
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .filter_map(move |fqdn| {
                if let Ok(domain) = Domain::new(fqdn.as_str()) {
                    Some(Permutation {
                        domain,
                        kind: PermutationKind::Bitsquatting,
                    })
                } else {
                    None
                }
            });

        permutations
    }

    /// Permutation method that replaces ASCII characters with multiple homoglyphs
    /// similar to the respective ASCII character.
    pub fn homoglyph(&self) -> Result<impl Iterator<Item = Permutation> + '_, Error> {
        // @CLEANUP(jdb): Tidy this entire mess up
        let mut result_first_pass: HashSet<Permutation> = HashSet::new();
        let mut result_second_pass: HashSet<Permutation> = HashSet::new();

        for ws in 1..self.fqdn.len() {
            for i in 0..(self.fqdn.len() - ws) + 1 {
                let win: String = self.fqdn[i..i + ws].to_string().chars().collect();
                let mut j = 0;

                while j < ws {
                    let c: char = win
                        .chars()
                        .nth(j)
                        .ok_or(PermutationError::InvalidHomoglyph {
                            domain: self.fqdn.to_string(),
                            homoglyph: win.to_string(),
                        })?;

                    if let Some(glyph) = HOMOGLYPHS.get(&c) {
                        for g in glyph.chars().collect::<Vec<char>>() {
                            let new_win = win.replace(c, &g.to_string());

                            let fqdn =
                                format!("{}{}{}", &self.fqdn[..i], &new_win, &self.fqdn[i + ws..]);

                            if let Ok(domain) = Domain::new(fqdn.as_str()) {
                                result_first_pass.insert(Permutation {
                                    domain,
                                    kind: PermutationKind::Homoglyph,
                                });
                            }
                        }
                    }

                    j += 1;
                }
            }
        }

        for permutation in &result_first_pass {
            for ws in 1..self.fqdn.len() {
                for i in 0..(self.fqdn.len() - ws) + 1 {
                    // We need to do this as we are dealing with UTF8 characters
                    // meaning that we cannot simple iterate over single byte
                    // values (as certain characters are composed of two or more)
                    let win: String = permutation.domain.fqdn.chars().collect::<Vec<char>>()
                        [i..i + ws]
                        .iter()
                        .collect();
                    let mut j = 0;

                    while j < ws {
                        let c: char =
                            win.chars()
                                .nth(j)
                                .ok_or(PermutationError::InvalidHomoglyph {
                                    domain: self.fqdn.to_string(),
                                    homoglyph: win.to_string(),
                                })?;

                        if let Some(glyph) = HOMOGLYPHS.get(&c) {
                            for g in glyph.chars().collect::<Vec<char>>() {
                                let new_win = win.replace(c, &g.to_string());
                                let fqdn = format!(
                                    "{}{}{}",
                                    &self.fqdn[..i],
                                    &new_win,
                                    &self.fqdn[i + ws..]
                                );

                                if let Ok(domain) = Domain::new(fqdn.as_str()) {
                                    result_second_pass.insert(Permutation {
                                        domain,
                                        kind: PermutationKind::Homoglyph,
                                    });
                                }
                            }
                        }

                        j += 1;
                    }
                }
            }
        }

        Ok((&result_first_pass | &result_second_pass).into_iter())
    }

    /// Permutation method that inserts hyphens (i.e. `-`) between each
    /// character in the domain where valid.
    pub fn hyphentation(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn
            .chars()
            .skip(1)
            .enumerate()
            .filter_map(move |(i, _)| {
                let mut permutation = self.fqdn.to_string();
                permutation.insert(i, '-');

                if let Ok(domain) = Domain::new(permutation.as_str()) {
                    Some(Permutation {
                        domain,
                        kind: PermutationKind::Hyphenation,
                    })
                } else {
                    None
                }
            })
    }

    /// Permutation method that inserts specific characters that are close to
    /// any character in the domain depending on the keyboard (e.g. `Q` next
    /// to `W` in qwerty keyboard layout.
    pub fn insertion(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn
            .chars()
            .skip(1) // We don't want to insert at the beginning of the domain...
            .take(self.fqdn.len() - 2) // ...or at the end of the domain.
            .enumerate()
            .flat_map(move |(i, c)| {
                KEYBOARD_LAYOUTS.iter().filter_map(move |layout| {
                    layout
                        .get(&c) // Option<&[char]>
                        .map(move |keyboard_chars| {
                            keyboard_chars.chars().filter_map(move |keyboard_char| {
                                let mut permutation = self.fqdn.to_string();
                                permutation.insert(i, keyboard_char);

                                if let Ok(domain) = Domain::new(permutation.as_str()) {
                                    Some(Permutation {
                                        domain,
                                        kind: PermutationKind::Insertion,
                                    })
                                } else {
                                    None
                                }
                            })
                        })
                })
            })
            .flatten()
    }

    /// Permutation method that selectively removes a character from the domain.
    pub fn omission(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn.chars().enumerate().filter_map(move |(i, _)| {
            let mut permutation = self.fqdn.to_string();
            permutation.remove(i);

            if let Ok(domain) = Domain::new(permutation.as_str()) {
                Some(Permutation {
                    domain,
                    kind: PermutationKind::Omission,
                })
            } else {
                None
            }
        })
    }

    /// Permutation method that repeats characters twice provided they are
    /// alphabetic characters (e.g. `google.com` -> `gooogle.com`).
    pub fn repetition(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn.chars().enumerate().filter_map(move |(i, c)| {
            if c.is_alphabetic() {
                let permutation = format!("{}{}{}", &self.fqdn[..=i], c, &self.fqdn[i + 1..]);

                if let Ok(domain) = Domain::new(permutation.as_str()) {
                    return Some(Permutation {
                        domain,
                        kind: PermutationKind::Repetition,
                    });
                }
            }

            None
        })
    }

    /// Permutation method similar to insertion, except that it replaces a given
    /// character with another character in proximity depending on keyboard layout.
    pub fn replacement(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn
            .chars()
            .skip(1) // We don't want to insert at the beginning of the domain...
            .take(self.fqdn.len() - 2) // ...or at the end of the domain.
            .enumerate()
            .flat_map(move |(i, c)| {
                KEYBOARD_LAYOUTS.iter().filter_map(move |layout| {
                    layout.get(&c).map(move |keyboard_chars| {
                        keyboard_chars.chars().filter_map(move |keyboard_char| {
                            let permutation = format!(
                                "{}{}{}",
                                &self.fqdn[..i],
                                keyboard_char,
                                &self.fqdn[i + 1..]
                            );

                            if let Ok(domain) = Domain::new(permutation.as_str()) {
                                Some(Permutation {
                                    domain,
                                    kind: PermutationKind::Replacement,
                                })
                            } else {
                                None
                            }
                        })
                    })
                })
            })
            .flatten()
    }

    pub fn subdomain(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn
            .chars()
            .take(self.fqdn.len() - 3)
            .enumerate()
            .tuple_windows()
            .filter_map(move |((_, c1), (i2, c2))| {
                if !['-', '.'].iter().all(|x| [c1, c2].contains(x)) {
                    let permutation = format!("{}.{}", &self.fqdn[..i2], &self.fqdn[i2..]);

                    if let Ok(domain) = Domain::new(permutation.as_str()) {
                        return Some(Permutation {
                            domain,
                            kind: PermutationKind::Subdomain,
                        });
                    }
                }

                None
            })
    }

    /// Permutation method that swaps out characters in the domain (e.g.
    /// `google.com` -> `goolge.com`).
    pub fn transposition(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn
            .chars()
            .enumerate()
            .tuple_windows()
            .filter_map(move |((i1, c1), (i2, c2))| {
                if c1 != c2 {
                    let permutation =
                        format!("{}{}{}{}", &self.fqdn[..i1], c2, c1, &self.fqdn[i2 + 1..]);
                    if let Ok(domain) = Domain::new(permutation.as_str()) {
                        return Some(Permutation {
                            domain,
                            kind: PermutationKind::Transposition,
                        });
                    }
                }

                None
            })
    }

    /// Permutation method that swaps vowels for other vowels (e.g.
    /// `google.com` -> `gougle.com`).
    pub fn vowel_swap(&self) -> impl Iterator<Item = Permutation> + '_ {
        self.fqdn
            .chars()
            .enumerate()
            .filter_map(move |(i, c)| {
                if VOWELS.contains(&c) {
                    Some(VOWELS.iter().filter_map(move |vowel| {
                        let permutation =
                            format!("{}{}{}", &self.fqdn[..i], vowel, &self.fqdn[i + 1..]);

                        if *vowel == c {
                            None
                        } else if let Ok(domain) = Domain::new(permutation.as_str()) {
                            Some(Permutation {
                                domain,
                                kind: PermutationKind::VowelSwap,
                            })
                        } else {
                            None
                        }
                    }))
                } else {
                    None
                }
            })
            .flatten()
    }

    /// Permutation mode that appends and prepends common keywords to the
    /// domain in the following order:
    ///
    /// 1. Prepend keyword and dash (e.g. `foo.com` -> `word-foo.com`)
    /// 2. Prepend keyword (e.g. `foo.com` -> `wordfoo.com`)
    /// 3. Append keyword and dash (e.g. `foo.com` -> `foo-word.com`)
    /// 4. Append keyword and dash (e.g. `foo.com` -> `fooword.com`)
    pub fn keyword(&self) -> impl Iterator<Item = Permutation> + '_ {
        KEYWORDS.iter().flat_map(move |keyword| {
            vec![
                format!("{}-{}.{}", &self.domain, keyword, &self.tld),
                format!("{}{}.{}", &self.domain, keyword, &self.tld),
                format!("{}-{}.{}", keyword, &self.domain, &self.tld),
                format!("{}{}.{}", keyword, &self.domain, &self.tld),
            ]
            .into_iter()
            .filter_map(move |fqdn| {
                if let Ok(domain) = Domain::new(fqdn.as_str()) {
                    return Some(Permutation {
                        domain,
                        kind: PermutationKind::Keyword,
                    });
                }

                None
            })
        })
    }

    /// Permutation method that replaces all TLDs as variations of the
    /// root domain passed.
    pub fn tld(&self) -> impl Iterator<Item = Permutation> + '_ {
        TLDS.iter().filter_map(move |tld| {
            let fqdn = format!("{}.{}", &self.domain, tld);

            if let Ok(domain) = Domain::new(fqdn.as_str()) {
                return Some(Permutation {
                    domain,
                    kind: PermutationKind::Tld,
                });
            }

            None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = d.all().unwrap().collect();

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_addition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.addition().collect());

        assert_eq!(permutations.len(), ASCII_LOWER.len());
    }

    #[test]
    fn test_bitsquatting_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.bitsquatting().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_homoglyph_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.homoglyph().unwrap().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_hyphenation_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.hyphentation().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_insertion_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.insertion().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_omission_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.omission().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_repetition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.repetition().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_replacement_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.replacement().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_subdomain_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.subdomain().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_transposition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.transposition().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_vowel_swap_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.vowel_swap().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_keyword_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.keyword().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_tld_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.tld().collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_domain_idna_filtering() {
        // Examples taken from IDNA Punycode RFC:
        // https://tools.ietf.org/html/rfc3492#section-7.1
        let idns: Vec<Permutation> = vec![
            // List of invalid domains
            String::from("i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd"),
            String::from("4dbcagdahymbxekheh6e0a7fei0b"),
            String::from("rpublique-numrique-bwbm"),
            String::from("fiqs8s"),
            String::from("acadmie-franaise-npb1a-google.com"),
            String::from("google.com.acadmie-franaise-npb1a"),
            // List of valid domains
            String::from("acadmie-franaise-npb1a"),
            String::from("google.com"),
            String::from("phishdeck.com"),
            String::from("xn--wgbl6a.icom.museum"),
            String::from("xn--80aaxgrpt.icom.museum"),
        ]
        .into_iter()
        .filter_map(|idn| {
            if let Ok(domain) = Domain::new(idn.as_str()) {
                Some(Permutation {
                    domain,
                    kind: PermutationKind::Addition,
                })
            } else {
                None
            }
        })
        .collect();

        let filtered_domains: Vec<Permutation> = idns.into_iter().collect();
        dbg!(&filtered_domains);

        assert_eq!(filtered_domains.len(), 5);
    }

    #[test]
    fn test_domains_empty_permutations_regression() {
        let domains: Vec<Domain> = vec!["ox.ac.uk", "oxford.ac.uk", "cool.co.nz"]
            .into_iter()
            .map(|fqdn| Domain::new(fqdn).unwrap())
            .collect();

        for domain in domains {
            let permutations: Vec<_> = dbg!(domain.all().unwrap().collect());
            assert!(!permutations.is_empty());
        }
    }
}
