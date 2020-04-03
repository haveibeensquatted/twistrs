#![cfg_attr(test, deny(warnings))]
#![deny(
    // TODO(jdb): Uncomment missing docs later on
    //missing_docs,
    future_incompatible,
    nonstandard_style,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_qualifications
)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
    clippy::decimal_literal_representation,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::expl_impl_clone_on_copy,
    clippy::fallible_impl_from,
    clippy::filter_map,
    clippy::filter_map_next,
    clippy::find_map,
    clippy::float_arithmetic,
    clippy::get_unwrap,
    clippy::if_not_else,
    clippy::inline_always,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::map_flatten,
    clippy::match_same_arms,
    clippy::maybe_infinite_iter,
    clippy::mem_forget,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::non_ascii_literal,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::path_buf_push_overwrite,
    clippy::print_stdout,
    clippy::pub_enum_variant_names,
    clippy::redundant_closure_for_method_calls,
    clippy::replace_consts,
    clippy::result_map_unwrap_or_else,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::single_match_else,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::type_repetition_in_bounds,
    clippy::unicode_not_nfc,
    // clippy::unimplemented,
    clippy::unseparated_literal_suffix,
    clippy::used_underscore_binding,
    clippy::wildcard_dependencies,
    // clippy::wildcard_enum_match_arm,
    clippy::wrong_pub_self_convention,
)]
#![recursion_limit = "128"]

#[macro_use]
extern crate lazy_static;

mod constants;

use constants::{ASCII_LOWER, DOMAIN_LIST, KEYBOARD_LAYOUTS};
use std::collections::HashSet;
use std::io::{Error, ErrorKind};

#[derive(Default, Debug)]
pub struct Domain<'a> {
    pub fqdn: &'a str,
    pub permutations: HashSet<String>,

    tld: String,
    domain: String,
}

pub enum PermutationMode {
    #[allow(dead_code)]
    All,
    Addition,
    BitSquatting,

    #[allow(dead_code)]
    Homoglyph,

    Hyphenation,
    Insertion,
    // TODO(jdb): Add remaining modes
}

impl<'a> Domain<'a> {
    // TODO(jdb): See how to clean this up
    fn inline_char_insert(
        i: usize,
        prefix: &'a Vec<char>,
        suffix: &'a Vec<char>,
        dst: &'a mut Vec<char>,
        c: char,
    ) -> &'a Vec<char> {
        prefix[..i]
            .into_iter()
            .enumerate()
            .for_each(|(_, c)| dst.push(*c));

        dst.push(c);

        prefix[i..]
            .into_iter()
            .enumerate()
            .for_each(|(_, c)| dst.push(*c));

        suffix[..]
            .into_iter()
            .enumerate()
            .for_each(|(_, c)| dst.push(*c));

        dst
    }

    pub fn new(fqdn: &'static str) -> Result<Domain<'a>, Error> {
        match DOMAIN_LIST.parse_domain(fqdn) {
            Ok(parsed_domain) => {
                let parts = parsed_domain
                    .root()
                    .unwrap() // TODO(jdb): Figure out how to handle this unwrap
                    .split('.')
                    .collect::<Vec<&str>>();

                let len = parts.len();

                if len < 2 {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("fqdn format is invalid: {:?}", fqdn),
                    ));
                }

                let tld = format!(".{}", String::from(parts[len - 1]));
                let domain = String::from(parts[len - 2]);

                Ok(Domain {
                    fqdn,
                    permutations: HashSet::new(),
                    tld,
                    domain,
                })
            }

            // TODO(jdb): See how we can pass the lazy_static error here
            //            since passing Err(e) is not thread-safe
            Err(_) => Err(Error::new(ErrorKind::Other, "")),
        }
    }

    pub fn mutate(&'a mut self, mode: PermutationMode) -> Result<&Domain<'a>, Error> {
        match mode {
            PermutationMode::Addition => Ok(self.addition()),
            PermutationMode::BitSquatting => Ok(self.bitsquatting()),
            PermutationMode::Hyphenation => Ok(self.hyphentation()),
            PermutationMode::Insertion => Ok(self.insertion()),
            _ => Err(Error::new(
                ErrorKind::Other,
                "permutation mode passed is currently unimplemented",
            )),
        }
    }

    fn addition(&'a mut self) -> &Domain<'a> {
        for c in ASCII_LOWER.iter() {
            self.permutations
                .insert(format!("{}{}.{}", self.domain, c.to_string(), self.tld));
        }

        self
    }

    fn bitsquatting(&'a mut self) -> &Domain<'a> {
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

        let domain = self.domain.chars().collect::<Vec<char>>();

        for c in domain.iter() {
            for mask_index in 0..8 {
                let mask = 1 << mask_index;

                // Can the below panic? Should we use a wider range (u32)?
                let squatted_char = mask ^ (*c as u8);

                // Make sure we remain with ASCII range that we are happy with
                if (squatted_char >= 48 && squatted_char <= 57)
                    || (squatted_char >= 97 && squatted_char <= 122)
                    || squatted_char == 45
                {
                    // TODO(jdb): See if there is a cleaner way to achieve this
                    let mut squatted_domain = Vec::with_capacity(domain.len() + self.tld.len());

                    self.permutations.insert(
                        Domain::inline_char_insert(
                            mask_index,
                            &domain,
                            &self.tld.chars().collect::<Vec<char>>(),
                            &mut squatted_domain,
                            squatted_char as char,
                        )
                        .into_iter()
                        .collect::<String>(),
                    );
                }
            }
        }

        self
    }

    fn hyphentation(&'a mut self) -> &Domain<'a> {
        let domain = &self.domain.chars().collect::<Vec<char>>();

        for (i, _) in domain.iter().enumerate() {
            // Skip the first index, as domains cannot start with hyphen
            if i == 0 {
                continue;
            }

            // Create buffer for new domain with same capacity plus 1
            // to allow for hyphen ('-') to be pushed.
            let mut squatted_domain: Vec<char> = Vec::with_capacity(domain.len() + 1);

            self.permutations.insert(
                Domain::inline_char_insert(
                    i,
                    domain,
                    &self.tld.chars().collect::<Vec<char>>(),
                    &mut squatted_domain,
                    '-',
                )
                .into_iter()
                .collect::<String>(),
            );
        }

        self
    }

    fn insertion(&'a mut self) -> &Domain<'a> {
        let domain = &self.domain.chars().collect::<Vec<char>>();

        for (i, c) in domain.iter().enumerate() {
            // We do not want to insert in the beginning or in the end of the domain
            if i == 0 || 1 == domain.len() - 1 {
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
                        let mut squatted_domain: Vec<char> = Vec::with_capacity(domain.len() + 1);

                        self.permutations.insert(
                            Domain::inline_char_insert(
                                i,
                                domain,
                                &self.tld.chars().collect::<Vec<char>>(),
                                &mut squatted_domain,
                                *keyboard_char,
                            )
                            .into_iter()
                            .collect::<String>(),
                        );
                    }
                }
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_mode() {
        let mut d = Domain::new("www.example.com").unwrap();
        assert_eq!(d.permutations.len(), 0);

        match d.mutate(PermutationMode::Addition) {
            Ok(v) => assert_eq!(v.permutations.len(), ASCII_LOWER.len()),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_bitsquatting_mode() {
        let mut d = Domain::new("www.example.com").unwrap();
        assert_eq!(d.permutations.len(), 0);

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::BitSquatting) {
            Ok(v) => assert!(v.permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_hyphenation_mode() {
        let mut d = Domain::new("www.example.com").unwrap();
        assert_eq!(d.permutations.len(), 0);

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::Hyphenation) {
            Ok(v) => assert!(v.permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_insertion_mode() {
        let mut d = Domain::new("www.example.com").unwrap();
        assert_eq!(d.permutations.len(), 0);

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::Insertion) {
            Ok(v) => assert!(v.permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }
}
