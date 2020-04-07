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
use dns::stub;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};

#[derive(Default, Debug)]
pub struct Domain<'a> {
    pub fqdn: &'a str,
    pub permutations: HashSet<String>,

    tld: String,
    domain: String,
}

// TODO(jdb): We need some sort of structure that allows
//            us to enrich the data from permutations that
//            are resolvable or interesting. So we could
//            keep something similar to:
//
//            "my.cool.fqdn": {
//              "mx": true,
//              "http": {
//                  "banner": "nginx/1.0",
//                  "ttl": true
//              }
//              "dns": {
//                  "ips": [ 1.1.1.1, 1.2.3.4 ]
//              }
//            }

#[derive(Copy, Clone)]
pub enum PermutationMode {
    All,
    Addition,
    BitSquatting,
    Homoglyph,
    Hyphenation,
    Insertion,
    // TODO(jdb): Add remaining modes
}

impl<'a> Domain<'a> {
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

                let tld = format!("{}", String::from(parts[len - 1]));
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

        stub(
            self.permutations
                .iter()
                .map(|s| &**s)
                .collect::<Vec<&str>>(),
        );

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
                        self.permutations.insert(permutation);
                    }
                }
            }
        }

        self
    }

    fn hyphentation(&'a mut self) -> &Domain<'a> {
        let fqdn = self.fqdn.to_string();

        for (i, _) in fqdn.chars().collect::<Vec<char>>().iter().enumerate() {
            // Skip the first index, as domains cannot start with hyphen
            if i == 0 {
                continue;
            }

            let mut permutation = self.fqdn.to_string();
            permutation.insert(i, '-');
            self.permutations.insert(permutation);
        }

        self
    }

    fn insertion(&'a mut self) -> &Domain<'a> {
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
                        self.permutations.insert(permutation);
                    }
                }
            }
        }

        self
    }
}

// CLEANUP(jdb): Move this into its own module
mod dns {
    use dns_lookup::lookup_host;
    use rayon::prelude::*;
    use std::net::IpAddr;

    fn dns_resolvable<'a>(addr: &'a &'a str) -> Option<Vec<IpAddr>> {
        match lookup_host(addr) {
            Ok(ips) => Some(ips),
            Err(_) => None,
        }
    }

    pub fn stub(domains: Vec<&str>) {
        let result: Vec<_> = domains.par_iter().filter_map(dns_resolvable).collect();

        println!("Result: {:#?}", result);
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
