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
use dns::enrich;

use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

/// Container to store interesting FQDN metadata
/// on domains that we resolvable and have some
/// interesting properties.
#[derive(Debug)]
pub struct DomainMetadata {
    ips: Box<Vec<IpAddr>>,
}

// TODO(jdb): Does it make sense that this container is kept in the
//            library? We need a way to store resolved domains in a
//            thread-safe manner. In this case, we need rayon to be
//            to add all resolved domains in a single container wit-
//            hout having to worry about thread safety as much.
type DomainStore = Arc<Mutex<HashMap<String, DomainMetadata>>>;

#[derive(Default, Debug)]
pub struct Domain<'a> {
    pub fqdn: &'a str,

    tld: String,
    domain: String,
}

#[derive(Copy, Clone)]
pub enum PermutationMode {
    All,
    Addition,
    BitSquatting,
    Homoglyph,
    Hyphenation,
    Insertion,
    Omission,
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

                Ok(Domain { fqdn, tld, domain })
            }

            // TODO(jdb): See how we can pass the lazy_static error here
            //            since passing Err(e) is not thread-safe
            Err(_) => Err(Error::new(ErrorKind::Other, "")),
        }
    }

    pub fn mutate(&self, mode: PermutationMode) -> Result<Vec<String>, Error> {
        match mode {
            PermutationMode::Addition => Ok(self.addition()),
            PermutationMode::BitSquatting => Ok(self.bitsquatting()),
            PermutationMode::Hyphenation => Ok(self.hyphentation()),
            PermutationMode::Insertion => Ok(self.insertion()),
            PermutationMode::Omission => Ok(self.omission()),
            _ => Err(Error::new(
                ErrorKind::Other,
                "permutation mode passed is currently unimplemented",
            )),
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
}

// CLEANUP(jdb): Move this into its own module
mod dns {
    use super::{DomainMetadata, DomainStore};
    use dns_lookup::lookup_host;
    use rayon::prelude::*;
    use std::net::IpAddr;

    struct ResolvedDomain {
        fqdn: String,
        ips: Vec<IpAddr>,
    }

    fn dns_resolvable<'a>(addr: &'a str) -> Option<ResolvedDomain> {
        match lookup_host(addr) {
            Ok(ips) => Some(ResolvedDomain {
                fqdn: String::from(addr),
                ips,
            }),
            Err(_) => None,
        }
    }

    pub fn enrich(domains: Vec<&str>, domain_store: &mut DomainStore) {
        // First level of enrichment, resolves the domain and adds its
        // list of resolved IPs.
        let domains = domains.into_par_iter().filter_map(dns_resolvable);

        domains.for_each(|resolved| {
            let mut _domain_store = domain_store.lock().unwrap();

            match _domain_store.get_mut(&resolved.fqdn) {
                Some(domain_metadata) => {
                    domain_metadata.ips = Box::new(resolved.ips);
                }
                None => {
                    let domain_metadata = DomainMetadata {
                        ips: Box::new(resolved.ips),
                    };
                    _domain_store.insert(resolved.fqdn, domain_metadata);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_mode() {
        let d = Domain::new("www.example.com").unwrap();

        match d.mutate(PermutationMode::Addition) {
            Ok(permutations) => assert_eq!(permutations.len(), ASCII_LOWER.len()),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_bitsquatting_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::BitSquatting) {
            Ok(permutations) => assert!(permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_hyphenation_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::Hyphenation) {
            Ok(permutations) => assert!(permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_insertion_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::Insertion) {
            Ok(permutations) => assert!(permutations.len() > 0),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_omission_mode() {
        let d = Domain::new("www.example.com").unwrap();

        // These are kind of lazy for the time being...
        match d.mutate(PermutationMode::Omission) {
            Ok(permutations) => {
                dbg!(&permutations);
                assert!(permutations.len() > 0);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_data_enrichment() {
        let d = Domain::new("www.example.com").unwrap();
        let mut resolved_domains = Arc::new(Mutex::new(HashMap::new()));

        match d.mutate(PermutationMode::Addition) {
            Ok(permutations) => {
                enrich(
                    permutations.iter().map(|s| &**s).collect::<Vec<&str>>(),
                    &mut resolved_domains,
                );
            }
            Err(e) => panic!(e),
        }

        match d.mutate(PermutationMode::Insertion) {
            Ok(permutations) => {
                enrich(
                    permutations.iter().map(|s| &**s).collect::<Vec<&str>>(),
                    &mut resolved_domains,
                );
            }
            Err(e) => panic!(e),
        }

        match d.mutate(PermutationMode::BitSquatting) {
            Ok(permutations) => {
                enrich(
                    permutations.iter().map(|s| &**s).collect::<Vec<&str>>(),
                    &mut resolved_domains,
                );
            }
            Err(e) => panic!(e),
        }
    }
}
