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

    fn filter(&self, domain: &Domain) -> bool {
        self.substrings
            .iter()
            .any(|s| domain.fqdn.contains(s.as_ref()))
    }
}
