use crate::constants::{ASCII_LOWER, DOMAIN_LIST, HOMOGLYPHS, KEYBOARD_LAYOUTS, VOWELS};
use rayon::prelude::*;

use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

#[derive(Default, Debug)]
pub struct Domain<'a> {
    pub fqdn: &'a str,

    tld: String,
    domain: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PermutationMode {
    All,
    Addition,
    BitSquatting,
    Homoglyph,
    Hyphenation,
    Insertion,
    Omission,
    Repetition,
    Replacement,
    Subdomain,
    Transposition,
    VowelSwap,
}

impl<'a> Domain<'a> {
    pub fn new(fqdn: &'static str) -> Result<Domain<'a>, Error> {
        match DOMAIN_LIST.parse_domain(fqdn) {
            Ok(parsed_domain) => {
                let parts = parsed_domain
                    .root()
                    .unwrap() // TODO(jdb): Figure out how to handle this unwrapoverride enum
                    .split('.')
                    .collect::<Vec<&str>>();

                let len = parts.len();

                if len < 2 {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("fqdn format is invalid: {:?}", fqdn),
                    ));
                }

                let tld = format!("{}", String::from(parts[len - 1]));
                let domain = String::from(parts[len - 2]);

                Ok(Domain { fqdn, tld, domain })
            }

            // TODO(jdb): See how we can pass the lazy_static error here
            //            since passing Err(e) is not thread-safe
            Err(_) => Err(Error::new(ErrorKind::Other, "")),
        }
    }

    pub fn permutate(&self, mode: PermutationMode) -> Result<Vec<String>, Error> {
        match mode {
            PermutationMode::Addition => Ok(self.addition()),
            PermutationMode::BitSquatting => Ok(self.bitsquatting()),
            PermutationMode::Homoglyph => Ok(self.homoglyph()),
            PermutationMode::Hyphenation => Ok(self.hyphentation()),
            PermutationMode::Insertion => Ok(self.insertion()),
            PermutationMode::Omission => Ok(self.omission()),
            PermutationMode::Repetition => Ok(self.repetition()),
            PermutationMode::Replacement => Ok(self.replacement()),
            PermutationMode::Subdomain => Ok(self.subdomain()),
            PermutationMode::Transposition => Ok(self.transposition()),
            PermutationMode::VowelSwap => Ok(self.vowel_swap()),
            PermutationMode::All => {
                let modes = vec![
                    PermutationMode::Addition,
                    PermutationMode::BitSquatting,
                    PermutationMode::Homoglyph,
                    PermutationMode::Hyphenation,
                    PermutationMode::Insertion,
                    PermutationMode::Omission,
                    PermutationMode::Repetition,
                    PermutationMode::Replacement,
                    PermutationMode::Subdomain,
                    PermutationMode::Transposition,
                    PermutationMode::VowelSwap,
                ];

                let permutations = Arc::new(Mutex::new(vec![]));

                modes
                    .into_par_iter()
                    .for_each(|mode| match self.permutate(mode) {
                        Ok(mutations) => {
                            for mutation in mutations.iter() {
                                permutations.lock().unwrap().push(String::from(mutation));
                            }
                        }
                        Err(e) => panic!(e),
                    });

                // TODO(jdb): Not sure if this even makes sense...
                // CLEANUP(jdb): See how we can just pass in the original
                //               set of permutations without having to do
                //               this entire dance.
                let mut v = vec![];

                for p in permutations.lock().unwrap().iter() {
                    v.push(String::from(p));
                }

                Ok(v)
            }
        }
    }

    fn addition(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        for c in ASCII_LOWER.iter() {
            result.push(format!("{}{}.{}", self.domain, c.to_string(), self.tld));
        }

        result
    }

    fn bitsquatting(&self) -> Vec<String> {
        // Following implementation takes inspiration from the following content:
        //  - https://github.com/artemdinaburg/bitsquat-script/blob/master/bitsquat.py
        //  - http://dinaburg.org/bitsquatting.html
        //
        // Go through each char in the domain and XOR it against 8 separate masks:
        //  00000001 ^ chr
        //  00000010 ^ chr
        //  00000100 ^ chr
        //  00001000 ^ chr
        //  00010000 ^ chr
        //  00100000 ^ chr
        //  01000000 ^ chr
        //  10000000 ^ chr
        //
        //  Then check if the resulting bit operation falls within ASCII range.

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

        result
    }

    fn homoglyph(&self) -> Vec<String> {
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

        (&result_first_pass | &result_second_pass)
            .into_iter()
            .collect()
    }

    fn hyphentation(&self) -> Vec<String> {
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

        result
    }

    fn insertion(&self) -> Vec<String> {
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

        result
    }

    fn omission(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];

        for (i, _) in self.fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // @CLEANUP(jdb): Any way to do this nicely? Just want to avoid
            //                out of bounds issues.
            if i == self.fqdn.len() {
                break;
            }

            result.push(format!("{}{}", &self.fqdn[..i], &self.fqdn[i + 1..]));
        }

        result
    }

    fn repetition(&self) -> Vec<String> {
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

        result
    }

    fn replacement(&self) -> Vec<String> {
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

        result
    }

    fn subdomain(&self) -> Vec<String> {
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

        result
    }

    fn transposition(&self) -> Vec<String> {
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

        result
    }

    fn vowel_swap(&self) -> Vec<String> {
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

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn test_addition_mode() {
        let d = Domain::new("www.example.com").unwrap();

        match d.permutate(PermutationMode::Addition) {
            Ok(permutations) => assert_eq!(permutations.len(), ASCII_LOWER.len()),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_bitsquatting_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::BitSquatting) {
            Ok(permutations) => assert!(permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_homoglyph_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Homoglyph) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_hyphenation_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Hyphenation) {
            Ok(permutations) => assert!(permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_insertion_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Insertion) {
            Ok(permutations) => assert!(permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_omission_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Omission) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_repetition_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Repetition) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_replacement_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Replacement) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_subdomain_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Subdomain) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_transposition_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::Transposition) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_vowel_swap_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.permutate(PermutationMode::VowelSwap) {
            Ok(permutations) => {
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }
}
