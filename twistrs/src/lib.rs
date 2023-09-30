//! Twistrs is a domain name permutation and enumeration library
//! that is built on top of async Rust.
//!
//! The library is designed to be fast, modular and easy-to-use
//! for clients.
//!
//! The two primary structs to look into are [Domain](./permutate/struct.Domain.html)
//! and [`DomainMetadata`](./enrich/struct.DomainMetadata.html).
//!
//! Additionally the module documentation for [permutation](./permutate/index.html)
//! and [enumeration](./enrich/index.html) provides more
//! granular details on how each module may be used indepedently.
//!
//! domain permutation and enrichment asynchronously.
//!
//! ### Example
//!
//! The following is a trivial example using [Tokio mpsc](https://docs.rs/tokio/0.2.22/tokio/sync/mpsc/index.html).
//!
//! ```
//! use twistrs::enrich::DomainMetadata;
//! use twistrs::permutate::Domain;
//!
//! use tokio::sync::mpsc;
//!
//!
//! #[tokio::main]
//! async fn main() {
//!     let domain = Domain::new("google.com").unwrap();
//!     let permutations = domain.addition();
//!
//!     let (tx, mut rx) = mpsc::channel(1000);
//!
//!     for permutation in permutations {
//!         let domain_metadata = DomainMetadata::new(permutation.domain.fqdn.clone());
//!         let mut tx = tx.clone();
//!
//!         tokio::spawn(async move {
//!             if let Err(_) = tx.send((permutation.clone(), domain_metadata.dns_resolvable().await)).await {
//!                 println!("received dropped");
//!                 return;
//!             }
//!
//!             drop(tx);
//!         });
//!     }
//!
//!     drop(tx);
//!
//!     while let Some(i) = rx.recv().await {
//!         println!("{:?}", i);
//!     }
//! }
//!
//! ```
//!

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
    clippy::manual_filter_map,
    clippy::filter_map_next,
    clippy::manual_find_map,
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
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::map_unwrap_or,
    clippy::path_buf_push_overwrite,
    clippy::print_stdout,
    clippy::redundant_closure_for_method_calls,
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
)]
#![recursion_limit = "128"]

#[macro_use]
extern crate lazy_static;

pub mod constants;
pub mod enrich;
pub mod error;
pub mod permutate;
