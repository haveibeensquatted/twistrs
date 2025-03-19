use crate::Domain;

pub trait Filter {
    type Error;

    fn filter(&self, domain: &Domain) -> bool;

    fn try_filter(&self, domain: &Domain) -> Result<bool, Self::Error> {
        Ok(Self::filter(self, domain))
    }
}

#[derive(Default, Copy, Clone)]
pub struct Permissive;

impl Filter for Permissive {
    type Error = ();

    fn filter(&self, _: &Domain) -> bool {
        true
    }
}

#[derive(Default, Copy, Clone)]
pub struct Substring<'a> {
    pub substrings: &'a [&'a str],
}

impl Filter for Substring<'_> {
    type Error = ();

    fn filter(&self, domain: &Domain) -> bool {
        self.substrings.iter().any(|s| domain.fqdn.contains(s))
    }
}
