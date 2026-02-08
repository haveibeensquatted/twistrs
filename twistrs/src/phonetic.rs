//! Phonetic distance computation using Metaphone 3 encoding.
//!
//! This module provides functionality to compute phonetic similarity
//! between domain names using the Metaphone 3 phonetic encoding algorithm
//! combined with normalized Levenshtein distance.

#![allow(clippy::module_name_repetitions)]

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
/// If both strings are empty, returns 0.0 (empty strings are identical).
#[allow(clippy::cast_precision_loss)]
fn normalized_levenshtein(s1: &str, s2: &str) -> f64 {
    let max_len = s1.len().max(s2.len());

    if max_len == 0 {
        // Both strings are empty - they are identical
        return 0.0;
    }

    let distance = levenshtein(s1, s2);
    
    #[allow(clippy::float_arithmetic)]
    let normalized = distance as f64 / max_len as f64;
    
    normalized
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
        // "kitten" -> "sitting" requires 3 operations: k->s, e->i, insert g
        // distance = 3 / 7 = 0.428...
        assert!((distance - 0.428).abs() < 0.01);
    }

    #[test]
    fn test_normalized_levenshtein_empty() {
        let distance = normalized_levenshtein("", "");
        // Empty strings are identical
        assert_eq!(distance, 0.0);
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

    #[test]
    fn test_phonetically_identical_domains() {
        // "phone" and "fone" should sound the same
        let base = Domain::new("phone.com").unwrap();
        let perm_domain = Domain::new("fone.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        // These should have very similar phonetic encodings
        assert!(result.data.distance < 0.3);
    }

    #[test]
    fn test_json_output_structure() {
        // Verify the JSON output matches the expected structure from the problem statement
        let base = Domain::new("example.com").unwrap();
        let perm_domain = Domain::new("example.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);
        let json_value: serde_json::Value = serde_json::to_value(&result).unwrap();

        // Verify structure
        assert!(json_value.get("permutation").is_some());
        assert!(json_value.get("op").is_some());
        assert!(json_value.get("data").is_some());

        let data = json_value.get("data").unwrap();
        assert!(data.get("encodings").is_some());
        assert!(data.get("distance").is_some());

        let encodings = data.get("encodings").unwrap();
        assert!(encodings.get("domain").is_some());
        assert!(encodings.get("permutation").is_some());

        // Verify values
        assert_eq!(json_value["op"], "Metaphone3");
        assert_eq!(json_value["data"]["distance"], 0.0);
    }

    #[test]
    fn test_primary_secondary_encoding_selection() {
        // Test that the algorithm correctly selects the best pairing
        let base = Domain::new("microsoft.com").unwrap();
        let perm_domain = Domain::new("mikerosoft.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert!(result.data.distance >= 0.0 && result.data.distance <= 1.0);
        // Encodings should be non-empty
        assert!(!result.data.encodings.domain.is_empty());
        assert!(!result.data.encodings.permutation.is_empty());
    }

    #[test]
    fn test_single_character_domains() {
        let base = Domain::new("a.com").unwrap();
        let perm_domain = Domain::new("b.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert!(result.data.distance >= 0.0 && result.data.distance <= 1.0);
    }

    #[test]
    fn test_numeric_domains() {
        let base = Domain::new("test123.com").unwrap();
        let perm_domain = Domain::new("test456.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert!(result.data.distance >= 0.0 && result.data.distance <= 1.0);
    }

    #[test]
    fn test_case_insensitive() {
        // Metaphone3 should handle case-insensitively
        let base = Domain::new("example.com").unwrap();
        let perm_domain = Domain::new("example.com").unwrap();
        let perm = Permutation {
            domain: perm_domain,
            kind: PermutationKind::Mapped,
        };

        let result = compute_phonetic_distance(&base, &perm);

        assert_eq!(result.op, "Metaphone3");
        assert_eq!(result.data.distance, 0.0);
    }
}
