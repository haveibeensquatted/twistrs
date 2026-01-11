use crate::permutate::DomainRef;
use crate::Domain;

/// The `Filter` trait provides functions that allow filtering of permutations given a certain
/// condition. This is useful when certain permutation methods (e.g.,
/// [`tld`](./permutate/Domain#tld)) expose permutations that you would like to dismiss.
pub trait Filter {
    type Error;

    fn matches(&self, domain: &Domain) -> bool;

    /// **Note** &mdash; this is currently not being used internally.
    fn try_matches(&self, domain: &Domain) -> Result<bool, Self::Error> {
        Ok(Self::matches(self, domain))
    }
}

/// An allocation-free filter that operates on [`DomainRef`].
#[allow(clippy::module_name_repetitions)]
pub trait FilterRef {
    type Error;

    fn matches(&self, domain: DomainRef<'_>) -> bool;

    /// **Note** &mdash; this is currently not being used internally.
    fn try_matches(&self, domain: DomainRef<'_>) -> Result<bool, Self::Error> {
        Ok(Self::matches(self, domain))
    }
}

/// Open filter, all results are retained; similar to a wildcard.
#[derive(Default, Copy, Clone)]
pub struct Permissive;

impl Filter for Permissive {
    type Error = ();

    fn matches(&self, _: &Domain) -> bool {
        true
    }
}

impl FilterRef for Permissive {
    type Error = ();

    fn matches(&self, _: DomainRef<'_>) -> bool {
        true
    }
}

/// When passed a slice of string patterns, will filter out values that do **not** contain any of
/// the substrings.
///
/// Example usage may be filtering the [`tld`](./permutate/Domain#tld) permutations to only include
/// TLDs that contain part of the origin TLD.
#[derive(Default, Copy, Clone)]
pub struct Substring<'a, S: AsRef<str> + 'a> {
    substrings: &'a [S],
}

impl<'a, S: AsRef<str>> Substring<'a, S> {
    pub fn new(substrings: &'a [S]) -> Self {
        Self { substrings }
    }
}

impl<S: AsRef<str>> Filter for Substring<'_, S> {
    type Error = ();

    fn matches(&self, domain: &Domain) -> bool {
        self.substrings
            .iter()
            .any(|s| domain.fqdn.contains(s.as_ref()))
    }
}

impl<S: AsRef<str>> FilterRef for Substring<'_, S> {
    type Error = ();

    fn matches(&self, domain: DomainRef<'_>) -> bool {
        self.substrings
            .iter()
            .any(|s| domain.fqdn.contains(s.as_ref()))
    }
}
