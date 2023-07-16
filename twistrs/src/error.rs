use crate::enrich::EnrichmentError;
use crate::permutate::PermutationError;
use std::convert::Infallible;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    PermutationError(#[from] PermutationError),

    #[error(transparent)]
    EnrichmentError(#[from] EnrichmentError),

    #[error(transparent)]
    Infallible(#[from] Infallible),
}
