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
//! use twistrs::permutate::Domain;
//!
//! let domain = Domain::new("google.com").unwrap();
//! let domain_permutations = domain.all().unwrap().collect::<Vec<String>>();
//! ```
//!
//! Additionally the permutation module can be used independently
//! from the enrichment module.
use crate::constants::{ASCII_LOWER, EFFECTIVE_TLDS, HOMOGLYPHS, KEYBOARD_LAYOUTS, VOWELS};

use std::collections::HashSet;
use std::fmt;

// Include further constants such as dictionaries that are 
// generated during compile time.
include!(concat!(env!("OUT_DIR"), "/dictionaries.rs"));

/// Temporary type-alias over `EnrichmentError`.
type Result<T> = std::result::Result<T, PermutationError>;

/// Wrapper around an FQDN to perform permutations against.
#[derive(Default, Debug)]
pub struct Domain<'a> {
    /// The domain FQDN to generate permutations from.
    pub fqdn: &'a str,

    /// The top-level domain of the FQDN (e.g. `.com`).
    tld: String,

    /// The remainder of the domain (e.g. `google`).
    domain: String,
}

#[deprecated(
    since = "0.1.0",
    note = "Prone to be removed in the future, does not currently provide any context."
)]
#[derive(Copy, Clone, Debug)]
pub struct PermutationError;

impl fmt::Display for PermutationError {
    // @CLEANUP(jdb): Make this something meaningful, if it needs to be
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl<'a> Domain<'a> {
    /// Wrap a desired FQDN into a `Domain` container. Internally
    /// will perform additional operations to break the domain into
    /// one or more chunks to be used during domain permutations.
    pub fn new(fqdn: &'a str) -> Result<Domain<'a>> {
        match EFFECTIVE_TLDS.parse_domain(fqdn) {
            Ok(parsed_domain) => {

                dbg!(&parsed_domain);
                let parts = parsed_domain
                    .root()
                    .unwrap() // TODO(jdb): Figure out how to handle this unwrap
                    .split('.')
                    .collect::<Vec<&str>>();

                let len = parts.len();

                if len < 2 {
                    return Err(PermutationError);
                }

                let tld = format!("{}", String::from(parts[len - 1]));
                let domain = String::from(parts[len - 2]);

                Ok(Domain { fqdn, tld, domain })
            }

            Err(_) => Err(PermutationError),
        }
    }

    /// Generate any and all possible domain permutations for a given `Domain`.
    ///
    /// Returns `Iterator<String>` with an iterator of domain permutations
    /// and includes the results of all other individual permutation methods.
    ///
    /// Any future permutations will also be included into this function call
    /// without any changes required from any client implementations.
    pub fn all(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let permutations = self
            .addition()
            .and_then(|i| Ok(i.chain(self.bitsquatting()?)))
            .and_then(|i| Ok(i.chain(self.homoglyph()?)))
            .and_then(|i| Ok(i.chain(self.hyphentation()?)))
            .and_then(|i| Ok(i.chain(self.insertion()?)))
            .and_then(|i| Ok(i.chain(self.omission()?)))
            .and_then(|i| Ok(i.chain(self.repetition()?)))
            .and_then(|i| Ok(i.chain(self.replacement()?)))
            .and_then(|i| Ok(i.chain(self.subdomain()?)))
            .and_then(|i| Ok(i.chain(self.transposition()?)))
            .and_then(|i| Ok(i.chain(self.vowel_swap()?)))
            .and_then(|i| Ok(i.chain(self.keyword()?)))
            .and_then(|i| Ok(i.chain(self.tld()?)))?;

        Ok(Box::new(permutations))
    }

    /// Add every ASCII lowercase character between the Domain
    /// (e.g. `google`) and top-level domain (e.g. `.com`).
    pub fn addition(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];
        for c in ASCII_LOWER.iter() {
            result.push(format!("{}{}.{}", self.domain, c.to_string(), self.tld));
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Following implementation takes inspiration from the following content:
    ///
    ///  - https://github.com/artemdinaburg/bitsquat-script/blob/master/bitsquat.py
    ///  - http://dinaburg.org/bitsquatting.html
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
    pub fn bitsquatting(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];
        let fqdn = self.fqdn.to_string();

        for c in fqdn.chars().collect::<Vec<char>>().iter() {
            for mask_index in 0..8 {
                let mask = 1 << mask_index;

                // Can the below panic? Should we use a wider range (u32)?
                let squatted_char: u8 = mask ^ (*c as u8);

                // Make sure we remain with ASCII range that we are happy with
                if (squatted_char >= 48 && squatted_char <= 57)
                    || (squatted_char >= 97 && squatted_char <= 122)
                    || squatted_char == 45
                {
                    for idx in 1..fqdn.len() {
                        let mut permutation = self.fqdn.to_string();
                        permutation.insert(idx, squatted_char as char);
                        result.push(permutation);
                    }
                }
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that replaces ASCII characters with multiple homoglyphs
    /// similar to the respective ASCII character.
    pub fn homoglyph(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        // @CLEANUP(jdb): Tidy this entire mess up
        let mut result_first_pass: HashSet<String> = HashSet::new();
        let mut result_second_pass: HashSet<String> = HashSet::new();

        let fqdn = self.fqdn.to_string().chars().collect::<Vec<char>>();

        for ws in 1..self.fqdn.len() {
            for i in 0..(self.fqdn.len() - ws) + 1 {
                let win: String = fqdn[i..i + ws].iter().collect();
                let mut j = 0;

                while j < ws {
                    let c: char = win.chars().nth(j).unwrap();

                    if HOMOGLYPHS.contains_key(&c) {
                        for glyph in HOMOGLYPHS.get(&c) {
                            let _glyph = glyph.chars().collect::<Vec<char>>();

                            for g in _glyph {
                                let new_win = win.replace(c, &g.to_string());
                                result_first_pass.insert(format!(
                                    "{}{}{}",
                                    &self.fqdn[..i],
                                    &new_win,
                                    &self.fqdn[i + ws..]
                                ));
                            }
                        }
                    }

                    j += 1;
                }
            }
        }

        for domain in result_first_pass.iter() {
            // We need to do this as we are dealing with UTF8 characters
            // meaning that we cannot simple iterate over single byte
            // values (as certain characters are composed of two or more)
            let _domain = domain.chars().collect::<Vec<char>>();

            for ws in 1..fqdn.len() {
                for i in 0..(fqdn.len() - ws) + 1 {
                    let win: String = _domain[i..i + ws].iter().collect();
                    let mut j = 0;

                    while j < ws {
                        let c: char = win.chars().nth(j).unwrap();

                        if HOMOGLYPHS.contains_key(&c) {
                            for glyph in HOMOGLYPHS.get(&c) {
                                let _glyph = glyph.chars().collect::<Vec<char>>();

                                for g in _glyph {
                                    let new_win = win.replace(c, &g.to_string());
                                    result_second_pass.insert(format!(
                                        "{}{}{}",
                                        &self.fqdn[..i],
                                        &new_win,
                                        &self.fqdn[i + ws..]
                                    ));
                                }
                            }
                        }

                        j += 1;
                    }
                }
            }
        }

        Ok(Box::new(
            (&result_first_pass | &result_second_pass)
                .into_iter()
                .collect::<Vec<String>>()
                .into_iter(),
        ))
    }

    /// Permutation method that inserts hyphens (i.e. `-`) between each
    /// character in the domain where valid.
    pub fn hyphentation(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];
        let fqdn = self.fqdn.to_string();

        for (i, _) in fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // Skip the first index, as domains cannot start with hyphen
            if i == 0 {
                continue;
            }

            let mut permutation = self.fqdn.to_string();
            permutation.insert(i, '-');
            result.push(permutation);
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that inserts specific characters that are close to
    /// any character in the domain depending on the keyboard (e.g. `Q` next
    /// to `W` in qwerty keyboard layout.
    pub fn insertion(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];
        let fqdn = self.fqdn.to_string();

        for (i, c) in fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // We do not want to insert in the beginning or in the end of the domain
            if i == 0 || i == fqdn.len() - 1 {
                continue;
            }

            for keyboard_layout in KEYBOARD_LAYOUTS.iter() {
                if keyboard_layout.contains_key(c) {
                    for keyboard_char in keyboard_layout
                        .get(c)
                        .unwrap()
                        .chars()
                        .collect::<Vec<char>>()
                        .iter()
                    {
                        let mut permutation = self.fqdn.to_string();
                        permutation.insert(i, *keyboard_char);
                        result.push(permutation);
                    }
                }
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that selectively removes a character from the domain.
    pub fn omission(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];

        for (i, _) in self.fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // @CLEANUP(jdb): Any way to do this nicely? Just want to avoid
            //                out of bounds issues.
            if i == self.fqdn.len() {
                break;
            }

            result.push(format!("{}{}", &self.fqdn[..i], &self.fqdn[i + 1..]));
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that repeats characters twice provided they are
    /// alphabetic characters (e.g. `google.com` -> `gooogle.com`).
    pub fn repetition(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];

        for (i, c) in self.fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // @CLEANUP(jdb): Any way to do this nicely? Just want to avoid
            //                out of bounds issues.
            if i == self.fqdn.len() {
                break;
            }

            if c.is_alphabetic() {
                result.push(format!("{}{}{}", &self.fqdn[..=i], c, &self.fqdn[i + 1..]));
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method similar to insertion, except that it replaces a given
    /// character with another character in proximity depending on keyboard layout.
    pub fn replacement(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];

        for (i, c) in self.fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // We do not want to insert in the beginning or in the end of the domain
            if i == 0 || i == self.fqdn.len() - 1 {
                continue;
            }

            for keyboard_layout in KEYBOARD_LAYOUTS.iter() {
                if keyboard_layout.contains_key(c) {
                    for keyboard_char in keyboard_layout
                        .get(c)
                        .unwrap()
                        .chars()
                        .collect::<Vec<char>>()
                        .iter()
                    {
                        result.push(format!(
                            "{}{}{}",
                            &self.fqdn[..i],
                            *keyboard_char,
                            &self.fqdn[i + 1..]
                        ));
                    }
                }
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    pub fn subdomain(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];
        let fqdn = self.fqdn.chars().collect::<Vec<char>>();

        for (i, c) in fqdn.iter().enumerate() {
            if i == 0 || i > self.fqdn.len() - 3 {
                continue;
            }

            let prev_char = &fqdn[i - 1];
            let invalid_chars = vec!['-', '.'];

            if !invalid_chars.contains(c) && !invalid_chars.contains(prev_char) {
                result.push(format!("{}.{}", &self.fqdn[..i], &self.fqdn[i..]));
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that swaps out characters in the domain (e.g.
    /// `google.com` -> `goolge.com`).
    pub fn transposition(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];
        let fqdn = self.fqdn.chars().collect::<Vec<char>>();

        for (i, c) in fqdn.iter().enumerate() {
            if i == 0 || i == self.fqdn.len() - 1 {
                continue;
            }

            let prev_char = &fqdn[i - 1];
            if c != prev_char {
                result.push(format!(
                    "{}{}{}{}",
                    &self.fqdn[..i],
                    &fqdn[i + 1],
                    c,
                    &self.fqdn[i + 2..]
                ));
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that swaps vowels for other vowels (e.g.
    /// `google.com` -> `gougle.com`).
    pub fn vowel_swap(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];

        for (i, c) in self.fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            for vowel in VOWELS.iter() {
                if VOWELS.contains(c) {
                    result.push(format!(
                        "{}{}{}",
                        &self.fqdn[..i],
                        *vowel,
                        &self.fqdn[i + 1..]
                    ));
                }
            }
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation mode that appends and prepends common keywords to the
    /// domain in the following order:
    /// 
    /// 1. Prepend keyword and dash (e.g. `foo.com` -> `word-foo.com`)
    /// 2. Prepend keyword (e.g. `foo.com` -> `wordfoo.com`)
    /// 3. Append keyword and dash (e.g. `foo.com` -> `foo-word.com`)
    /// 4. Append keyword and dash (e.g. `foo.com` -> `fooword.com`)
    pub fn keyword(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];

        for keyword in KEYWORDS.iter() {
            result.push(format!(
                "{}-{}.{}",
                &self.domain,
                keyword,
                &self.tld
            ));

            result.push(format!(
                "{}{}.{}",
                &self.domain,
                keyword,
                &self.tld
            ));

            result.push(format!(
                "{}-{}.{}",
                keyword,
                &self.domain,
                &self.tld
            ));

            result.push(format!(
                "{}{}.{}",
                keyword,
                &self.domain,
                &self.tld
            ));
        }

        Ok(Box::new(result.into_iter()))
    }

    /// Permutation method that appends all TLDs as variations of the 
    /// root domain passed. Note that this each TLD generates two
    /// TLDs:
    /// 
    /// 1. TLD stripping the current TLD (e.g. `foo.com` -> `foo.it`)
    /// 2. TLD appended to the current TLD (e.g. `foo.com` -> `foo.com.mt`)
    pub fn tld(&self) -> Result<Box<dyn Iterator<Item = String>>> {
        let mut result: Vec<String> = vec![];

        for tld in TLDS.iter() {

            // Push first TLD appending to previous TLD
            result.push(format!(
                "{}.{}.{}",
                &self.domain,
                &self.tld,
                tld
            ));

            // Push second TLD stripping previous TLD
            result.push(format!(
                "{}.{}",
                &self.domain,
                tld
            ));            
        }

        Ok(Box::new(result.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.all();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }    

    #[test]
    fn test_addition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.addition();

        assert!(permutations.is_ok());
        assert_eq!(
            permutations.unwrap().collect::<Vec<String>>().len(),
            ASCII_LOWER.len()
        );
    }

    #[test]
    fn test_bitsquatting_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.bitsquatting();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_homoglyph_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.homoglyph();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_hyphenation_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.hyphentation();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_insertion_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.insertion();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_omission_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.omission();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_repetition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.repetition();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_replacement_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.replacement();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_subdomain_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.subdomain();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_transposition_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.transposition();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_vowel_swap_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.vowel_swap();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_keyword_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.keyword();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }

    #[test]
    fn test_tld_mode() {
        let d = Domain::new("www.example.com").unwrap();
        let permutations = d.tld();

        assert!(permutations.is_ok());
        assert!(permutations.unwrap().collect::<Vec<String>>().len() > 0);
    }
}
