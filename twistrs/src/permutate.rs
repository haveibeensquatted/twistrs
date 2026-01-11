//! The permutation module exposes functionality around generating
//! multiple valid variations of a given domain. Note that this
//! module is _only_ concerned with generating possible permutations
//! of a given domain.
//!
//! Example:
//!
//! ```
//! use twistrs::{
//!   permutate::{Domain, Permutation},
//!   filter::{Filter, Permissive},
//! };
//!
//! let domain = Domain::new("google.com").unwrap();
//! let domain_permutations: Vec<Permutation> = domain.all(&Permissive).collect();
//! ```
//!
use crate::constants::{
    ASCII_LOWER, HOMOGLYPHS, KEYBOARD_LAYOUTS, MAPPED_VALUES, VOWELS, VOWEL_SHUFFLE_CEILING,
};
use crate::error::Error;
use crate::filter::{Filter, FilterRef};

use addr::parser::DomainName;
use addr::psl::List;
use itertools::{repeat_n, Itertools};
use serde::{Deserialize, Serialize};
// Include further constants such as dictionaries that are
// generated during compile time.
include!(concat!(env!("OUT_DIR"), "/data.rs"));

use crate::tlds::TLDS;

/// Wrapper around an FQDN to perform permutations against.
#[derive(Clone, Hash, Default, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Domain {
    /// The domain FQDN to generate permutations from.
    pub fqdn: String,

    /// The top-level domain of the FQDN (e.g. `.com`).
    pub tld: String,

    /// The remainder of the domain (e.g. `google`).
    pub domain: String,
}

/// A borrowed view of a parsed domain, used by allocation-free APIs.
#[derive(Clone, Copy, Hash, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DomainRef<'a> {
    /// The domain FQDN to generate permutations from.
    pub fqdn: &'a str,

    /// The top-level domain of the FQDN (e.g. `.com`).
    pub tld: &'a str,

    /// The remainder of the domain (e.g. `google`).
    pub domain: &'a str,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Permutation {
    pub domain: Domain,
    pub kind: PermutationKind,
}

/// A borrowed view of a permutation, used by allocation-free APIs.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PermutationRef<'a> {
    pub domain: DomainRef<'a>,
    pub kind: PermutationKind,
}

#[derive(Clone, Copy, Serialize, Deserialize, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
    VowelShuffle,
    DoubleVowelInsertion,
    Keyword,
    Tld,
    Homoglyph,
    Mapped,
}

#[derive(Clone, thiserror::Error, Debug)]
pub enum PermutationError {
    #[error("invalid domain name, (expected {expected:?}, found {found:?})")]
    InvalidDomain { expected: String, found: String },

    #[error("error generating homoglyph permutation (domain {domain:?}, homoglyph {homoglyph:?})")]
    InvalidHomoglyph { domain: String, homoglyph: String },
}

impl<'a> DomainRef<'a> {
    /// Parse and validate a domain name into a borrowed representation.
    pub fn new(fqdn: &'a str) -> Result<Self, Error> {
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

        let tld = parsed_domain.suffix();

        // Verify that the TLD is in the list of known TLDs, this requires that
        // the TLD data list is already ordered, otherwise the result of the
        // binary search is meaningless. We also assume that all TLDs generated
        // are lowercase already.
        if TLDS.binary_search(&tld).is_ok() {
            let domain = root_domain
                .find('.')
                .and_then(|offset| root_domain.get(..offset))
                // this should never error out since `root_domain` is a valid domain name
                .ok_or(PermutationError::InvalidDomain {
                    expected: "valid domain name with a root domain".to_string(),
                    found: fqdn.to_string(),
                })?;

            Ok(Self { fqdn, tld, domain })
        } else {
            let err = PermutationError::InvalidDomain {
                expected: "valid domain tld in the list of accepted tlds globally".to_string(),
                found: tld.to_string(),
            };

            Err(err.into())
        }
    }

    /// Specialised form of `DomainRef::new` that does not validate the TLD against
    /// the baked-in TLD list.
    pub fn raw(fqdn: &'a str) -> Result<Self, Error> {
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

        let tld = parsed_domain.suffix();
        let domain = root_domain
            .find('.')
            .and_then(|offset| root_domain.get(..offset))
            // this should never error out since `root_domain` is a valid domain name
            .ok_or(PermutationError::InvalidDomain {
                expected: "valid domain name with a root domain".to_string(),
                found: fqdn.to_string(),
            })?;

        Ok(Self { fqdn, tld, domain })
    }
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
        if TLDS.binary_search(&tld.as_str()).is_ok() {
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

    /// Specialised form of `Domain::new` that does not perform certain validations. This is
    /// enables downstream users to generate domains faster, with looser validation requirements.
    pub fn raw(fqdn: &str) -> Result<Domain, Error> {
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

        Ok(Domain {
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
        })
    }

    /// Generate any and all possible domain permutations for a given `Domain`.
    ///
    /// Returns `Iterator<String>` with an iterator of domain permutations
    /// and includes the results of all other individual permutation methods.
    ///
    /// Any future permutations will also be included into this function call
    /// without any changes required from any client implementations.
    pub fn all<'a>(&'a self, filter: &'a impl Filter) -> impl Iterator<Item = Permutation> + 'a {
        self.addition(filter)
            .chain(self.bitsquatting(filter))
            .chain(self.hyphenation(filter))
            .chain(self.hyphenation_tld_boundary(filter))
            .chain(self.insertion(filter))
            .chain(self.omission(filter))
            .chain(self.repetition(filter))
            .chain(self.replacement(filter))
            .chain(self.subdomain(filter))
            .chain(self.transposition(filter))
            .chain(self.vowel_swap(filter))
            .chain(self.vowel_shuffle(VOWEL_SHUFFLE_CEILING, filter))
            .chain(self.double_vowel_insertion(filter))
            .chain(self.keyword(filter))
            .chain(self.tld(filter))
            .chain(self.mapped(filter))
            .chain(self.homoglyph(filter))
    }

    /// Allocation-free equivalent of [`Domain::all`].
    ///
    /// The provided `buffer` is reused for each candidate, which avoids allocating a new `String`
    /// per permutation.
    pub fn visit_all_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        self.visit_addition_with_buf(filter, buffer, visit);
        self.visit_bitsquatting_with_buf(filter, buffer, visit);
        self.visit_hyphenation_with_buf(filter, buffer, visit);
        self.visit_hyphenation_tld_boundary_with_buf(filter, buffer, visit);
        self.visit_insertion_with_buf(filter, buffer, visit);
        self.visit_omission_with_buf(filter, buffer, visit);
        self.visit_repetition_with_buf(filter, buffer, visit);
        self.visit_replacement_with_buf(filter, buffer, visit);
        self.visit_subdomain_with_buf(filter, buffer, visit);
        self.visit_transposition_with_buf(filter, buffer, visit);
        self.visit_vowel_swap_with_buf(filter, buffer, visit);
        self.visit_vowel_shuffle_with_buf(VOWEL_SHUFFLE_CEILING, filter, buffer, visit);
        self.visit_double_vowel_insertion_with_buf(filter, buffer, visit);
        self.visit_keyword_with_buf(filter, buffer, visit);
        self.visit_tld_with_buf(filter, buffer, visit);
        self.visit_mapped_with_buf(filter, buffer, visit);
        self.visit_homoglyph_with_buf(filter, buffer, visit);
    }

    /// Allocation-free equivalent of [`Domain::all`].
    ///
    /// Internally uses a reusable `String` buffer.
    pub fn visit_all<FilterT, VisitT>(&self, filter: &FilterT, mut visit: VisitT)
    where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let mut buffer = String::with_capacity(self.fqdn.len() + 32);
        self.visit_all_with_buf(filter, &mut buffer, &mut visit);
    }

    /// Add every ASCII lowercase character between the Domain
    /// (e.g. `google`) and top-level domain (e.g. `.com`).
    pub fn addition<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                ASCII_LOWER
                    .iter()
                    .map(move |c| format!("{}{}.{}", self.domain, c, self.tld))
            },
            PermutationKind::Addition,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::addition`].
    pub fn visit_addition_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        for c in ASCII_LOWER {
            buffer.clear();
            buffer.push_str(&self.domain);
            buffer.push(c);
            buffer.push('.');
            buffer.push_str(&self.tld);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Addition, filter, visit);
        }
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
    pub fn bitsquatting<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn
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
            },
            PermutationKind::Bitsquatting,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::bitsquatting`].
    pub fn visit_bitsquatting_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        let len = fqdn.len();

        for c in fqdn.chars() {
            for mask_index in 0..8 {
                let mask = 1 << mask_index;

                // Can the below panic? Should we use a wider range (u32)?
                let squatted_char: u8 = mask ^ (c as u8);

                // Make sure we remain with ASCII range that we are happy with
                if ((48..=57).contains(&squatted_char))
                    || ((97..=122).contains(&squatted_char))
                    || squatted_char == 45
                {
                    let inserted = squatted_char as char;
                    for idx in 1..len {
                        if !fqdn.is_char_boundary(idx) {
                            continue;
                        }

                        buffer.clear();
                        buffer.push_str(&fqdn[..idx]);
                        buffer.push(inserted);
                        buffer.push_str(&fqdn[idx..]);
                        Self::emit_ref_candidate(
                            buffer.as_str(),
                            PermutationKind::Bitsquatting,
                            filter,
                            visit,
                        );
                    }
                }
            }
        }
    }

    /// Permutation method that replaces ASCII characters with multiple homoglyphs
    /// similar to the respective ASCII character.
    pub fn homoglyph<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                let fqdn = self.fqdn.as_str();
                fqdn.char_indices()
                    .filter_map(move |(idx, c)| {
                        HOMOGLYPHS.get(&c).map(move |glyphs| (idx, c, glyphs))
                    })
                    .flat_map(move |(idx, c, glyphs)| {
                        let next = idx + c.len_utf8();
                        glyphs.chars().map(move |g| {
                            let mut out = String::with_capacity(fqdn.len() + g.len_utf8());
                            out.push_str(&fqdn[..idx]);
                            out.push(g);
                            out.push_str(&fqdn[next..]);
                            out
                        })
                    })
            },
            PermutationKind::Homoglyph,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::homoglyph`].
    pub fn visit_homoglyph_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();

        for (idx, c) in fqdn.char_indices() {
            let Some(glyphs) = HOMOGLYPHS.get(&c) else {
                continue;
            };
            let next = idx + c.len_utf8();

            for g in glyphs.chars() {
                buffer.clear();
                buffer.push_str(&fqdn[..idx]);
                buffer.push(g);
                buffer.push_str(&fqdn[next..]);
                Self::emit_ref_candidate(
                    buffer.as_str(),
                    PermutationKind::Homoglyph,
                    filter,
                    visit,
                );
            }
        }
    }

    /// Permutation method that inserts hyphens (i.e. `-`) between each
    /// character in the domain where valid.
    pub fn hyphenation<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn.chars().skip(1).enumerate().map(move |(i, _)| {
                    let mut permutation = self.fqdn.to_string();
                    permutation.insert(i, '-');
                    permutation
                })
            },
            PermutationKind::Hyphenation,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::hyphenation`].
    pub fn visit_hyphenation_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        for idx in 0..fqdn.len().saturating_sub(1) {
            if !fqdn.is_char_boundary(idx) {
                continue;
            }

            buffer.clear();
            buffer.push_str(&fqdn[..idx]);
            buffer.push('-');
            buffer.push_str(&fqdn[idx..]);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Hyphenation, filter, visit);
        }
    }

    /// In cases of multi-level TLDs, will swap the top-level dot to a hyphen. For example
    /// `abcd.co.uk` would map to `abcd-co.uk`. Internally this still maps to the `Hyphenation`
    /// permutation kind, however is a refined subset for performance purposes. Will always yield
    /// at most, one permutation.
    pub fn hyphenation_tld_boundary<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                // `then(..)` returns `Option<String>` with a single concrete type
                // whether it is `Some` or `None`.
                (self.tld.contains('.'))
                    .then(|| format!("{}-{}", self.domain, self.tld))
                    .into_iter() // Option → IntoIter (0‒1 items)
            },
            PermutationKind::Hyphenation,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::hyphenation_tld_boundary`].
    pub fn visit_hyphenation_tld_boundary_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        if !self.tld.contains('.') {
            return;
        }

        buffer.clear();
        buffer.push_str(&self.domain);
        buffer.push('-');
        buffer.push_str(&self.tld);
        Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Hyphenation, filter, visit);
    }

    /// Permutation method that inserts specific characters that are close to
    /// any character in the domain depending on the keyboard (e.g. `Q` next
    /// to `W` in qwerty keyboard layout.
    pub fn insertion<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn
                    .chars()
                    .skip(1) // We don't want to insert at the beginning of the domain...
                    .take(self.fqdn.len() - 2) // ...or at the end of the domain.
                    .enumerate()
                    .flat_map(move |(i, c)| {
                        KEYBOARD_LAYOUTS.iter().flat_map(move |layout| {
                            layout
                                .get(&c) // Option<&[char]>
                                .into_iter()
                                .map(move |keyboard_chars| {
                                    keyboard_chars.chars().map(move |keyboard_char| {
                                        let mut permutation = self.fqdn.to_string();
                                        permutation.insert(i, keyboard_char);
                                        permutation.to_string()
                                    })
                                })
                        })
                    })
                    .flatten()
            },
            PermutationKind::Insertion,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::insertion`].
    pub fn visit_insertion_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        let len = fqdn.len();

        // Mirror the current iterator implementation, including its indexing semantics.
        for (i, c) in fqdn.chars().skip(1).take(len.saturating_sub(2)).enumerate() {
            for layout in KEYBOARD_LAYOUTS.iter() {
                let Some(keyboard_chars) = layout.get(&c) else {
                    continue;
                };

                for keyboard_char in keyboard_chars.chars() {
                    if !fqdn.is_char_boundary(i) {
                        continue;
                    }

                    buffer.clear();
                    buffer.push_str(&fqdn[..i]);
                    buffer.push(keyboard_char);
                    buffer.push_str(&fqdn[i..]);
                    Self::emit_ref_candidate(
                        buffer.as_str(),
                        PermutationKind::Insertion,
                        filter,
                        visit,
                    );
                }
            }
        }
    }

    /// Permutation method that selectively removes a character from the domain.
    pub fn omission<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn.chars().enumerate().map(move |(i, _)| {
                    let mut permutation = self.fqdn.to_string();
                    permutation.remove(i);
                    permutation
                })
            },
            PermutationKind::Omission,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::omission`].
    pub fn visit_omission_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        for (idx, c) in fqdn.char_indices() {
            let next = idx + c.len_utf8();
            buffer.clear();
            buffer.push_str(&fqdn[..idx]);
            buffer.push_str(&fqdn[next..]);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Omission, filter, visit);
        }
    }

    /// Permutation method that repeats characters twice provided they are
    /// alphabetic characters (e.g. `google.com` -> `gooogle.com`).
    pub fn repetition<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn.chars().enumerate().filter_map(move |(i, c)| {
                    if c.is_alphabetic() {
                        Some(format!("{}{}{}", &self.fqdn[..=i], c, &self.fqdn[i + 1..]))
                    } else {
                        None
                    }
                })
            },
            PermutationKind::Repetition,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::repetition`].
    pub fn visit_repetition_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();

        for (idx, c) in fqdn.char_indices() {
            if !c.is_alphabetic() {
                continue;
            }

            let next = idx + c.len_utf8();
            buffer.clear();
            buffer.push_str(&fqdn[..next]);
            buffer.push(c);
            buffer.push_str(&fqdn[next..]);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Repetition, filter, visit);
        }
    }

    /// Permutation method similar to insertion, except that it replaces a given
    /// character with another character in proximity depending on keyboard layout.
    pub fn replacement<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn
                    .chars()
                    .skip(1) // We don't want to insert at the beginning of the domain...
                    .take(self.fqdn.len() - 2) // ...or at the end of the domain.
                    .enumerate()
                    .flat_map(move |(i, c)| {
                        KEYBOARD_LAYOUTS.iter().filter_map(move |layout| {
                            layout.get(&c).map(move |keyboard_chars| {
                                keyboard_chars.chars().map(move |keyboard_char| {
                                    format!(
                                        "{}{}{}",
                                        &self.fqdn[..i],
                                        keyboard_char,
                                        &self.fqdn[i + 1..]
                                    )
                                })
                            })
                        })
                    })
                    .flatten()
            },
            PermutationKind::Replacement,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::replacement`].
    pub fn visit_replacement_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        let len = fqdn.len();

        for (i, c) in fqdn.chars().skip(1).take(len.saturating_sub(2)).enumerate() {
            for layout in KEYBOARD_LAYOUTS.iter() {
                let Some(keyboard_chars) = layout.get(&c) else {
                    continue;
                };

                for keyboard_char in keyboard_chars.chars() {
                    if !fqdn.is_char_boundary(i) {
                        continue;
                    }

                    // Mirror the current iterator implementation, including its indexing semantics.
                    buffer.clear();
                    buffer.push_str(&fqdn[..i]);
                    buffer.push(keyboard_char);
                    let replace_end = (i + 1).min(fqdn.len());
                    buffer.push_str(&fqdn[replace_end..]);
                    Self::emit_ref_candidate(
                        buffer.as_str(),
                        PermutationKind::Replacement,
                        filter,
                        visit,
                    );
                }
            }
        }
    }

    pub fn subdomain<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn
                    .chars()
                    .take(self.fqdn.len() - 3)
                    .enumerate()
                    .tuple_windows()
                    .filter_map(move |((_, c1), (i2, c2))| {
                        if ['-', '.'].iter().all(|x| [c1, c2].contains(x)) {
                            None
                        } else {
                            Some(format!("{}.{}", &self.fqdn[..i2], &self.fqdn[i2..]))
                        }
                    })
            },
            PermutationKind::Subdomain,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::subdomain`].
    pub fn visit_subdomain_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        let len = fqdn.len();

        for (i2, (c1, c2)) in fqdn
            .chars()
            .take(len.saturating_sub(3))
            .tuple_windows::<(_, _)>()
            .enumerate()
            .map(|(idx, (c1, c2))| (idx + 1, (c1, c2)))
        {
            if ['-', '.'].iter().all(|x| [c1, c2].contains(x)) {
                continue;
            }

            if !fqdn.is_char_boundary(i2) {
                continue;
            }

            buffer.clear();
            buffer.push_str(&fqdn[..i2]);
            buffer.push('.');
            buffer.push_str(&fqdn[i2..]);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Subdomain, filter, visit);
        }
    }

    /// Permutation method that swaps out characters in the domain (e.g.
    /// `google.com` -> `goolge.com`).
    pub fn transposition<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn.chars().enumerate().tuple_windows().filter_map(
                    move |((i1, c1), (i2, c2))| {
                        if c1 == c2 {
                            None
                        } else {
                            Some(format!(
                                "{}{}{}{}",
                                &self.fqdn[..i1],
                                c2,
                                c1,
                                &self.fqdn[i2 + 1..]
                            ))
                        }
                    },
                )
            },
            PermutationKind::Transposition,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::transposition`].
    pub fn visit_transposition_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        let bytes = fqdn.as_bytes();

        for i in 0..bytes.len().saturating_sub(1) {
            let c1 = bytes[i] as char;
            let c2 = bytes[i + 1] as char;
            if c1 == c2 {
                continue;
            }

            buffer.clear();
            buffer.push_str(&fqdn[..i]);
            buffer.push(c2);
            buffer.push(c1);
            let suffix_start = (i + 2).min(fqdn.len());
            buffer.push_str(&fqdn[suffix_start..]);
            Self::emit_ref_candidate(
                buffer.as_str(),
                PermutationKind::Transposition,
                filter,
                visit,
            );
        }
    }

    /// Permutation method that swaps vowels for other vowels (e.g.
    /// `google.com` -> `gougle.com`).
    pub fn vowel_swap<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn
                    .chars()
                    .enumerate()
                    .filter_map(move |(i, c)| {
                        if VOWELS.contains(&c.to_ascii_lowercase()) {
                            Some(VOWELS.iter().filter_map(move |vowel| {
                                if *vowel == c {
                                    None
                                } else {
                                    Some(format!(
                                        "{}{}{}",
                                        &self.fqdn[..i],
                                        vowel,
                                        &self.fqdn[i + 1..]
                                    ))
                                }
                            }))
                        } else {
                            None
                        }
                    })
                    .flatten()
            },
            PermutationKind::VowelSwap,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::vowel_swap`].
    pub fn visit_vowel_swap_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();

        for (idx, c) in fqdn.char_indices() {
            if !VOWELS.contains(&c.to_ascii_lowercase()) {
                continue;
            }

            let next = idx + c.len_utf8();
            for vowel in VOWELS {
                if vowel == c {
                    continue;
                }

                buffer.clear();
                buffer.push_str(&fqdn[..idx]);
                buffer.push(vowel);
                buffer.push_str(&fqdn[next..]);
                Self::emit_ref_candidate(
                    buffer.as_str(),
                    PermutationKind::VowelSwap,
                    filter,
                    visit,
                );
            }
        }
    }

    /// A superset of [`vowel_swap`][`vowel_swap`], which computes the multiple cartesian product
    /// of all vowels found in the domain, and maps them against their indices.
    ///
    /// * `ceil`: limit the upperbound exponent of possible permutations that can be generated
    ///   (i.e., 5^{ceil}) where 5 is the number of possible vowels, and `{ceil}` is the
    ///   number of products to generate
    pub fn vowel_shuffle<'a>(
        &'a self,
        ceil: usize,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                let vowel_positions = self
                    .domain
                    .chars()
                    .enumerate()
                    .filter_map(|(i, c)| if VOWELS.contains(&c) { Some(i) } else { None })
                    .collect_vec();

                // |cartesian_product| = |VOWELS|^n = 5^n
                let products =
                    repeat_n(VOWELS, vowel_positions.len().min(ceil)).multi_cartesian_product();

                products.map(move |replacement| {
                    // build the new label
                    let mut label: Vec<char> = self.domain.chars().collect();
                    for (pos, &new_vowel) in vowel_positions.iter().zip(&replacement) {
                        label[*pos] = new_vowel;
                    }
                    let fqdn = format!("{}.{}", label.iter().collect::<String>(), self.tld);
                    fqdn
                })
            },
            PermutationKind::VowelShuffle,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::vowel_shuffle`].
    pub fn visit_vowel_shuffle_with_buf<FilterT, VisitT>(
        &self,
        ceil: usize,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let vowel_positions = self
            .domain
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if VOWELS.contains(&c) { Some(i) } else { None })
            .collect_vec();

        let n = vowel_positions.len().min(ceil);
        if n == 0 {
            return;
        }

        let mut digits = vec![0usize; n];
        loop {
            buffer.clear();

            let mut v = 0usize;
            for (i, c) in self.domain.chars().enumerate() {
                if v < n && vowel_positions[v] == i {
                    buffer.push(VOWELS[digits[v]]);
                    v += 1;
                } else {
                    buffer.push(c);
                }
            }
            buffer.push('.');
            buffer.push_str(&self.tld);

            Self::emit_ref_candidate(
                buffer.as_str(),
                PermutationKind::VowelShuffle,
                filter,
                visit,
            );

            // Increment base-|VOWELS| counter.
            let mut carry = true;
            for d in digits.iter_mut().rev() {
                if !carry {
                    break;
                }
                *d += 1;
                if *d == VOWELS.len() {
                    *d = 0;
                } else {
                    carry = false;
                }
            }

            if carry {
                break;
            }
        }
    }

    /// Permutation method that inserts every lowercase ascii character between
    /// two vowels.
    pub fn double_vowel_insertion<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                self.fqdn
                    .chars()
                    .enumerate()
                    .tuple_windows()
                    .filter_map(move |((i1, c1), (i2, c2))| {
                        if VOWELS.contains(&c1.to_ascii_lowercase())
                            && VOWELS.contains(&c2.to_ascii_lowercase())
                        {
                            Some(ASCII_LOWER.iter().map(move |inserted| {
                                format!("{}{inserted}{}", &self.fqdn[..=i1], &self.fqdn[i2..])
                            }))
                        } else {
                            None
                        }
                    })
                    .flatten()
            },
            PermutationKind::DoubleVowelInsertion,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::double_vowel_insertion`].
    pub fn visit_double_vowel_insertion_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        let fqdn = self.fqdn.as_str();
        let bytes = fqdn.as_bytes();

        for i in 0..bytes.len().saturating_sub(1) {
            let c1 = bytes[i] as char;
            let c2 = bytes[i + 1] as char;

            if !(VOWELS.contains(&c1.to_ascii_lowercase())
                && VOWELS.contains(&c2.to_ascii_lowercase()))
            {
                continue;
            }

            for inserted in ASCII_LOWER {
                buffer.clear();
                buffer.push_str(&fqdn[..=i]);
                buffer.push(inserted);
                buffer.push_str(&fqdn[i + 1..]);
                Self::emit_ref_candidate(
                    buffer.as_str(),
                    PermutationKind::DoubleVowelInsertion,
                    filter,
                    visit,
                );
            }
        }
    }

    /// Permutation mode that appends and prepends common keywords to the
    /// domain in the following order:
    ///
    /// 1. Prepend keyword and dash (e.g. `foo.com` -> `word-foo.com`)
    /// 2. Prepend keyword (e.g. `foo.com` -> `wordfoo.com`)
    /// 3. Append keyword and dash (e.g. `foo.com` -> `foo-word.com`)
    /// 4. Append keyword and dash (e.g. `foo.com` -> `fooword.com`)
    pub fn keyword<'a>(
        &'a self,
        filter: &'a impl Filter,
    ) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                KEYWORDS.iter().flat_map(move |keyword| {
                    vec![
                        format!("{}-{}.{}", &self.domain, keyword, &self.tld),
                        format!("{}{}.{}", &self.domain, keyword, &self.tld),
                        format!("{}-{}.{}", keyword, &self.domain, &self.tld),
                        format!("{}{}.{}", keyword, &self.domain, &self.tld),
                    ]
                    .into_iter()
                })
            },
            PermutationKind::Keyword,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::keyword`].
    pub fn visit_keyword_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        for keyword in KEYWORDS {
            // 1. Append keyword and dash (e.g. `foo.com` -> `foo-word.com`)
            buffer.clear();
            buffer.push_str(&self.domain);
            buffer.push('-');
            buffer.push_str(keyword);
            buffer.push('.');
            buffer.push_str(&self.tld);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Keyword, filter, visit);

            // 2. Append keyword (e.g. `foo.com` -> `fooword.com`)
            buffer.clear();
            buffer.push_str(&self.domain);
            buffer.push_str(keyword);
            buffer.push('.');
            buffer.push_str(&self.tld);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Keyword, filter, visit);

            // 3. Prepend keyword and dash (e.g. `foo.com` -> `word-foo.com`)
            buffer.clear();
            buffer.push_str(keyword);
            buffer.push('-');
            buffer.push_str(&self.domain);
            buffer.push('.');
            buffer.push_str(&self.tld);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Keyword, filter, visit);

            // 4. Prepend keyword (e.g. `foo.com` -> `wordfoo.com`)
            buffer.clear();
            buffer.push_str(keyword);
            buffer.push_str(&self.domain);
            buffer.push('.');
            buffer.push_str(&self.tld);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Keyword, filter, visit);
        }
    }

    /// Permutation method that replaces all TLDs as variations of the
    /// root domain passed.
    pub fn tld<'a>(&'a self, filter: &'a impl Filter) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                TLDS.iter()
                    .map(move |tld| format!("{}.{}", &self.domain, tld))
            },
            PermutationKind::Tld,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::tld`].
    pub fn visit_tld_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        for tld in TLDS {
            buffer.clear();
            buffer.push_str(&self.domain);
            buffer.push('.');
            buffer.push_str(tld);
            Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Tld, filter, visit);
        }
    }

    /// Permutation method that maps one or more characters into another
    /// set of one or more characters that are similar, or easy to miss,
    /// such as `d` -> `cl`, `ck` -> `kk`.
    pub fn mapped<'a>(&'a self, filter: &'a impl Filter) -> impl Iterator<Item = Permutation> + 'a {
        Self::permutation(
            move || {
                let mut results = vec![];

                for (key, values) in MAPPED_VALUES.entries() {
                    if self.domain.contains(key) {
                        let parts = self.domain.split(key);

                        for mapped_value in *values {
                            let result = format!(
                                "{domain}.{tld}",
                                domain = parts.clone().join(mapped_value),
                                tld = self.tld
                            );
                            results.push(result);
                        }
                    }
                }

                results.into_iter()
            },
            PermutationKind::Mapped,
            filter,
        )
    }

    /// Allocation-free equivalent of [`Domain::mapped`].
    pub fn visit_mapped_with_buf<FilterT, VisitT>(
        &self,
        filter: &FilterT,
        buffer: &mut String,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        for (key, values) in MAPPED_VALUES.entries() {
            if !self.domain.contains(key) {
                continue;
            }

            for mapped_value in *values {
                buffer.clear();

                let mut split = self.domain.split(key).peekable();
                while let Some(part) = split.next() {
                    buffer.push_str(part);
                    if split.peek().is_some() {
                        buffer.push_str(mapped_value);
                    }
                }

                buffer.push('.');
                buffer.push_str(&self.tld);

                Self::emit_ref_candidate(buffer.as_str(), PermutationKind::Mapped, filter, visit);
            }
        }
    }

    fn emit_ref_candidate<FilterT, VisitT>(
        candidate: &str,
        kind: PermutationKind,
        filter: &FilterT,
        visit: &mut VisitT,
    ) where
        FilterT: FilterRef,
        VisitT: for<'a> FnMut(PermutationRef<'a>),
    {
        if let Ok(domain) = DomainRef::new(candidate) {
            if filter.matches(domain) {
                visit(PermutationRef { domain, kind });
            }
        }
    }

    /// Auxilliary function that wraps each permutation function in order to perform validation and
    /// filtering of results. This leaves us with a trimmed down list of permutations that are
    /// valid domains and accepted by the `Filter` passed.
    fn permutation<'a, S, T: Fn() -> S + 'a, U: Filter + 'a>(
        f: T,
        kind: PermutationKind,
        filter: &'a U,
    ) -> impl Iterator<Item = Permutation> + use<'a, S, T, U>
    where
        S: Iterator<Item = String> + 'a,
    {
        f().filter_map(move |candidate| {
            if let Ok(domain) = Domain::new(candidate.as_str()) {
                if filter.matches(&domain) {
                    return Some(Permutation { domain, kind });
                }
            }

            None
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::{Permissive, Substring};

    use super::*;

    #[test]
    fn test_all_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = d.all(&Permissive).collect();

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_addition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.addition(&Permissive).collect());

        assert_eq!(permutations.len(), ASCII_LOWER.len());
    }

    #[test]
    fn test_visit_addition_matches_addition() {
        let d = Domain::new("www.example.com").unwrap();
        let expected: Vec<String> = d.addition(&Permissive).map(|p| p.domain.fqdn).collect();

        let mut buffer = String::new();
        let mut actual: Vec<String> = Vec::new();
        d.visit_addition_with_buf(&Permissive, &mut buffer, &mut |p| {
            actual.push(p.domain.fqdn.to_string());
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bitsquatting_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.bitsquatting(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_homoglyph_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.homoglyph(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_hyphenation_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.hyphenation(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_insertion_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.insertion(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_omission_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.omission(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_repetition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.repetition(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_replacement_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.replacement(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_subdomain_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.subdomain(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_transposition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.transposition(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_vowel_swap_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.vowel_swap(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_keyword_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.keyword(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_tld_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations: Vec<_> = dbg!(d.tld(&Permissive).collect());

        assert!(!permutations.is_empty());
    }

    #[test]
    fn test_mapping_mode() {
        let d = Domain::new("www.exoock96z.com").unwrap();
        let permutations: Vec<_> = dbg!(d.mapped(&Permissive).collect());

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
            let permutations: Vec<_> = dbg!(domain.all(&Permissive).collect());
            assert!(!permutations.is_empty());
        }
    }

    #[test]
    fn test_domains_double_vowel_insertion() {
        let domain = Domain::new("exampleiveus.com").unwrap();
        let expected = Domain::new("exampleivesus.com").unwrap();

        let results: Vec<Permutation> = domain
            .double_vowel_insertion(&Permissive)
            .filter(|p| p.domain.fqdn == expected.fqdn)
            .collect();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn regression_test_co_uk_tld_is_valid() {
        // Ensure we do not miss two-level TLDs such as .co.uk
        let domain = Domain::new("bbc.com").unwrap();
        let expected = [
            Domain::new("bbc.co.uk").unwrap().fqdn,
            Domain::new("bbc.co.rs").unwrap().fqdn,
            Domain::new("bbc.co.uz").unwrap().fqdn,
        ];

        let results: Vec<Permutation> = domain
            .tld(&Permissive)
            .filter(|p| expected.contains(&p.domain.fqdn))
            .collect();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_mapped_generates_expected_permutation() {
        let domain = Domain::new("trm.com").unwrap();
        let expected = Domain::new("trnn.com").unwrap();

        let results: Vec<Permutation> = domain
            .mapped(&Permissive)
            .filter(|p| p.domain.fqdn == expected.fqdn)
            .collect();

        assert_eq!(results.len(), 1);
    }

    /// Regression test against <https://github.com/haveibeensquatted/twistrs/issues/102>
    #[test]
    fn test_irrelevant_tlds_not_being_generated() {
        struct InnerFilter;
        impl Filter for InnerFilter {
            type Error = ();

            fn matches(&self, domain: &Domain) -> bool {
                domain.fqdn.contains("gov")
            }
        }

        let domain = Domain::new("www.gov.uk").unwrap();
        let unexpected = Domain::new("www.alta.no").unwrap();

        let results: Vec<Permutation> = domain
            .tld(&InnerFilter)
            .filter(|p| p.domain.fqdn == unexpected.fqdn)
            .collect();

        assert_eq!(results.len(), 0);
    }

    /// Tests that the `Substring` filter behaves as expected
    #[test]
    fn test_substring_default_filter() {
        let filter = Substring::new(&["gov", "uk"]);
        let domain = Domain::new("www.gov.uk").unwrap();

        assert!(domain
            .all(&filter)
            .all(|p| p.domain.fqdn.contains("gov") || p.domain.fqdn.contains("uk")));
    }

    #[test]
    fn test_vowel_shuffling_permutation() {
        let domain = Domain::new("xiaomi.com").unwrap();
        let expected = [
            Domain::new("xoaimi.com").unwrap().fqdn,
            Domain::new("xaoimi.com").unwrap().fqdn,
            Domain::new("xiaoma.com").unwrap().fqdn,
        ];

        let results: Vec<Permutation> = domain
            .vowel_shuffle(VOWEL_SHUFFLE_CEILING, &Permissive)
            .filter(|p| expected.contains(&p.domain.fqdn))
            .collect();

        dbg!(&results);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_vowel_shuffling_limit() {
        let domain = Domain::new("haveibeensquatted.com").unwrap();
        let results: Vec<Permutation> = domain
            .vowel_shuffle(VOWEL_SHUFFLE_CEILING, &Permissive)
            .collect();

        #[allow(clippy::cast_possible_truncation)]
        let ceil: u32 = VOWEL_SHUFFLE_CEILING as u32;
        assert!(results.len() <= VOWELS.len().pow(ceil));

        let results_exceeds: Vec<Permutation> = domain
            .vowel_shuffle(VOWEL_SHUFFLE_CEILING + 1, &Permissive)
            .collect();

        #[allow(clippy::cast_possible_truncation)]
        let ceil_exceeds: u32 = VOWEL_SHUFFLE_CEILING as u32;
        assert!(results_exceeds.len() > VOWELS.len().pow(ceil_exceeds));
    }

    #[test]
    fn test_hyphenation_tld_boundary_permutation() {
        let domain = Domain::new("abcd.co.uk").unwrap();
        let expected = Domain::new("abcd-co.uk").unwrap();

        let results: Vec<Permutation> = domain
            .hyphenation_tld_boundary(&Permissive)
            .filter(|p| p.domain.fqdn == expected.fqdn)
            .collect();

        assert_eq!(results.len(), 1);
    }
}
