//! Phonetic distance computation using Metaphone 3 encoding.
//!
//! This module provides functionality to compute phonetic similarity
//! between domain names using the Metaphone 3 phonetic encoding algorithm
//! combined with normalized Levenshtein distance.

use crate::permutate::{Domain, Permutation};
use metaphone3::Metaphone3;
use serde::{Deserialize, Serialize};
use strsim::levenshtein;

/// Represents the result of a phonetic distance computation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PhoneticResult {
    /// The original permutation being analyzed.
    pub permutation: Permutation,
    /// The operation type (always "Metaphone3" for phonetic distance).
    pub op: String,
    /// The phonetic distance data including encodings and distance.
    pub data: PhoneticData,
}

/// Contains the phonetic encodings and computed distance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PhoneticData {
    /// The selected Metaphone 3 encodings for both domains.
    pub encodings: PhoneticEncodings,
    /// The normalized Levenshtein distance (0.0 = most similar, 1.0 = most different).
    pub distance: f64,
}

/// The selected Metaphone 3 encodings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhoneticEncodings {
    /// The encoding for the base domain label.
    pub domain: String,
    /// The encoding for the permutation domain label.
    pub permutation: String,
}

/// Computes the phonetic distance between a base domain and a permutation.
///
/// # Algorithm
/// 1. Extracts the domain labels (second-level domain) from both domains
/// 2. Computes Metaphone 3 encodings (primary and secondary) for each label
/// 3. Calculates normalized Levenshtein distance for all valid pairings
/// 4. Selects the pairing with the smallest distance
/// 5. Returns the selected encodings and distance
///
/// # Arguments
/// * `base_domain` - The original domain to compare against
/// * `permutation` - The permutation to compute distance for
///
/// # Returns
/// A `PhoneticResult` containing the permutation, operation type, and phonetic data.
///
/// # Example
/// ```
/// use twistrs::permutate::Domain;
/// use twistrs::phonetic::compute_phonetic_distance;
///
/// let base = Domain::new("example.com").unwrap();
/// let perm_domain = Domain::new("eksample.com").unwrap();
/// let perm = twistrs::permutate::Permutation {
///     domain: perm_domain,
///     kind: twistrs::permutate::PermutationKind::Mapped,
/// };
///
/// let result = compute_phonetic_distance(&base, &perm);
/// assert_eq!(result.op, "Metaphone3");
/// assert!(result.data.distance >= 0.0 && result.data.distance <= 1.0);
/// ```
pub fn compute_phonetic_distance(base_domain: &Domain, permutation: &Permutation) -> PhoneticResult {
    let mut encoder = Metaphone3::new();

    // Extract domain labels (ignore TLD and subdomains)
    let base_label = &base_domain.domain;
    let perm_label = &permutation.domain.domain;

    // Compute Metaphone 3 encodings
    let (base_primary, base_secondary) = encoder.encode(base_label);
    let (perm_primary, perm_secondary) = encoder.encode(perm_label);

    // Compute normalized Levenshtein distance for all valid pairings
    let mut best_distance = f64::MAX;
    let mut best_pairing = (0, 0); // (base_idx, perm_idx) where 0=primary, 1=secondary

    // Define the order of pairings: Ap-Bp, Ap-Bs, As-Bp, As-Bs
    let pairings = [
        (base_primary.as_str(), perm_primary.as_str(), 0, 0),
        (base_primary.as_str(), perm_secondary.as_str(), 0, 1),
        (base_secondary.as_str(), perm_primary.as_str(), 1, 0),
        (base_secondary.as_str(), perm_secondary.as_str(), 1, 1),
    ];

    for (base_key, perm_key, base_idx, perm_idx) in &pairings {
        // Skip if either key is empty
        if base_key.is_empty() || perm_key.is_empty() {
            continue;
        }

        // Calculate normalized Levenshtein distance
        let distance = normalized_levenshtein(base_key, perm_key);

        // Select the pairing with smallest distance (ties go to first in order)
        if distance < best_distance {
            best_distance = distance;
            best_pairing = (*base_idx, *perm_idx);
        }
    }

    // If no valid pairing was found (all keys were empty), use distance 1.0
    if best_distance == f64::MAX {
        best_distance = 1.0;
    }

    // Get the selected encodings
    let selected_base = if best_pairing.0 == 0 {
        base_primary.to_string()
    } else {
        base_secondary.to_string()
    };

    let selected_perm = if best_pairing.1 == 0 {
        perm_primary.to_string()
    } else {
        perm_secondary.to_string()
    };

    PhoneticResult {
        permutation: permutation.clone(),
        op: "Metaphone3".to_string(),
        data: PhoneticData {
            encodings: PhoneticEncodings {
                domain: selected_base,
                permutation: selected_perm,
            },
            distance: best_distance,
        },
    }
}

/// Computes the normalized Levenshtein distance between two strings.
///
/// # Arguments
/// * `s1` - First string
/// * `s2` - Second string
///
/// # Returns
/// A float in the range [0.0, 1.0] where:
/// - 0.0 means the strings are identical
/// - 1.0 means the strings are completely different
///
/// The distance is normalized by dividing by the maximum length of the two strings.
/// If both strings are empty, returns 1.0.
fn normalized_levenshtein(s1: &str, s2: &str) -> f64 {
    let max_len = s1.len().max(s2.len());

    if max_len == 0 {
        return 1.0;
    }

    let distance = levenshtein(s1, s2);
    distance as f64 / max_len as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permutate::{Domain, PermutationKind};

    #[test]
    fn test_normalized_levenshtein_identical() {
        let distance = normalized_levenshtein("test", "test");
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_normalized_levenshtein_different() {
        let distance = normalized_levenshtein("abc", "xyz");
        assert_eq!(distance, 1.0); // 3 changes / 3 max_len = 1.0
    }

    #[test]
    fn test_normalized_levenshtein_partial() {
        let distance = normalized_levenshtein("kitten", "sitting");
        // "kitten" -> "sitting" requires 3 operations: s->k, i->e, g insertion
        // distance = 3 / 7 = 0.428...
        assert!((distance - 0.428).abs() < 0.01);
    }

    #[test]
    fn test_normalized_levenshtein_empty() {
        let distance = normalized_levenshtein("", "");
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_normalized_levenshtein_one_empty() {
        let distance = normalized_levenshtein("", "abc");
        assert_eq!(distance, 1.0); // 3 / 3 = 1.0
    }

    #[test]
    fn test_compute_phonetic_distance_identical() {
        let base = Domain::new("example.com").unwrap();
        let perm_domain = Domain::new("example.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert_eq!(result.data.distance, 0.0);
        assert_eq!(result.data.encodings.domain, result.data.encodings.permutation);
    }

    #[test]
    fn test_compute_phonetic_distance_similar() {
        let base = Domain::new("example.com").unwrap();
        let perm_domain = Domain::new("eksample.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert!(result.data.distance >= 0.0 && result.data.distance <= 1.0);
        // "example" and "eksample" should have similar encodings
    }

    #[test]
    fn test_compute_phonetic_distance_different() {
        let base = Domain::new("google.com").unwrap();
        let perm_domain = Domain::new("amazon.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert!(result.data.distance > 0.0 && result.data.distance <= 1.0);
    }

    #[test]
    fn test_phonetic_result_serialization() {
        let base = Domain::new("test.com").unwrap();
        let perm_domain = Domain::new("tast.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        // Test that the structure can be serialized
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Metaphone3"));
        assert!(json.contains("distance"));
        assert!(json.contains("encodings"));
    }
}
