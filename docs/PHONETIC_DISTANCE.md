# Phonetic Distance for Domain Permutations

This module implements phonetic distance computation for domain permutations using the **Metaphone 3** phonetic encoding algorithm combined with normalized Levenshtein distance.

## Overview

The phonetic distance feature helps identify domain permutations that sound similar to the base domain, even if they are spelled differently. This is useful for detecting potential typosquatting or phishing domains that exploit phonetic similarity.

## Algorithm

The implementation follows these steps:

1. **Extract domain labels**: Only the registrable second-level domain label is compared. TLDs, subdomains, and prefixes like "www" are ignored.
   - Example: For `www.example.com` → compare `example`

2. **Compute Metaphone 3 encodings**: Each domain label is encoded using Metaphone 3, which produces both primary and secondary phonetic keys.
   - Example: `phone` → primary: `FN`, secondary: (empty)
   - Example: `fone` → primary: `FN`, secondary: (empty)

3. **Calculate normalized Levenshtein distance**: The edit distance is computed for all valid pairings of encodings:
   - Base primary ↔ Permutation primary
   - Base primary ↔ Permutation secondary
   - Base secondary ↔ Permutation primary
   - Base secondary ↔ Permutation secondary
   
   Pairings with empty keys are skipped.

4. **Normalize distance**: Distance is normalized by dividing by the maximum length of the two encodings:
   ```
   normalized_distance = levenshtein(key1, key2) / max(len(key1), len(key2))
   ```
   - Result is in range [0.0, 1.0]
   - 0.0 = most similar (identical)
   - 1.0 = most different

5. **Select best pairing**: The pairing with the smallest normalized distance is selected. In case of ties, the first pairing in this order is chosen: `Ap-Bp`, `Ap-Bs`, `As-Bp`, `As-Bs`.

6. **Return result**: The function returns a `PhoneticResult` containing:
   - Original permutation data
   - Operation type ("Metaphone3")
   - Selected encodings and distance

## Usage

### Basic Example

```rust
use twistrs::{
    compute_phonetic_distance,
    permutate::{Domain, Permutation, PermutationKind},
};

// Create base domain
let base = Domain::new("example.com").unwrap();

// Create a permutation
let perm_domain = Domain::new("eksample.com").unwrap();
let permutation = Permutation {
    domain: perm_domain,
    kind: PermutationKind::Mapped,
};

// Compute phonetic distance
let result = compute_phonetic_distance(&base, &permutation);

println!("Operation: {}", result.op);
println!("Base encoding: {}", result.data.encodings.domain);
println!("Perm encoding: {}", result.data.encodings.permutation);
println!("Distance: {:.4}", result.data.distance);
```

### Analyzing Permutations

```rust
use twistrs::{
    compute_phonetic_distance,
    filter::Permissive,
    permutate::Domain,
};

let base = Domain::new("example.com").unwrap();

// Compute phonetic distance for all permutations
let mut results: Vec<_> = base
    .all(&Permissive)
    .map(|perm| compute_phonetic_distance(&base, &perm))
    .collect();

// Sort by distance (most similar first)
results.sort_by(|a, b| {
    a.data.distance
        .partial_cmp(&b.data.distance)
        .unwrap_or(std::cmp::Ordering::Equal)
});

// Show top 10 most phonetically similar
for result in results.iter().take(10) {
    println!("{}: distance={:.4}", 
        result.permutation.domain.fqdn, 
        result.data.distance);
}
```

## JSON Output Format

The result can be serialized to JSON:

```json
{
  "permutation": {
    "domain": {
      "fqdn": "example.com",
      "tld": "com",
      "domain": "example"
    },
    "kind": "Mapped"
  },
  "op": "Metaphone3",
  "data": {
    "encodings": {
      "domain": "AKSMPL",
      "permutation": "AKSMPL"
    },
    "distance": 0.0
  }
}
```

## Examples

### Phonetically Identical Domains

```rust
// "phone" and "fone" sound the same
let base = Domain::new("phone.com").unwrap();
let perm = create_permutation("fone.com");
let result = compute_phonetic_distance(&base, &perm);

// Both encode to "FN", distance = 0.0
assert_eq!(result.data.distance, 0.0);
```

### Phonetically Similar Domains

```rust
// "example" and "eksample" sound similar
let base = Domain::new("example.com").unwrap();
let perm = create_permutation("eksample.com");
let result = compute_phonetic_distance(&base, &perm);

// Encodings are similar, distance is low
// example → AKSMPL
// eksample → ASMPL
// distance ≈ 0.167
```

### Phonetically Different Domains

```rust
// "google" and "amazon" sound different
let base = Domain::new("google.com").unwrap();
let perm = create_permutation("amazon.com");
let result = compute_phonetic_distance(&base, &perm);

// Encodings are very different, distance is high
// distance > 0.5
```

## Use Cases

1. **Typosquatting Detection**: Identify domains that sound like legitimate brands
2. **Phishing Prevention**: Flag domains that exploit phonetic similarity
3. **Brand Protection**: Monitor for phonetically similar domain registrations
4. **Security Analysis**: Assess risk of phonetic confusion attacks

## Dependencies

- **metaphone3** (0.1.0): Rust implementation of Metaphone 3 algorithm
- **strsim** (0.11): String similarity metrics including Levenshtein distance

## See Also

- [Metaphone 3 Algorithm](https://en.wikipedia.org/wiki/Metaphone)
- [Levenshtein Distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
- [Domain Name Permutations](../README.md)
