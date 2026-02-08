//! Example demonstrating phonetic distance computation for domain permutations.
//!
//! This example shows how to:
//! 1. Generate permutations for a domain
//! 2. Compute phonetic distance using Metaphone 3
//! 3. Filter permutations by phonetic similarity

use twistrs::{
    compute_phonetic_distance,
    filter::Permissive,
    permutate::Domain,
};

fn main() {
    // Create a base domain
    let base_domain = Domain::new("example.com").unwrap();
    
    println!("Base domain: {}", base_domain.fqdn);
    println!("Computing phonetic distances for permutations...\n");

    // Generate a few permutations and compute their phonetic distance
    let mut results: Vec<_> = base_domain
        .all(&Permissive)
        .take(20) // Just take the first 20 for demonstration
        .map(|perm| compute_phonetic_distance(&base_domain, &perm))
        .collect();

    // Sort by distance (most similar first)
    results.sort_by(|a, b| {
        a.data.distance
            .partial_cmp(&b.data.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Display top 10 most phonetically similar permutations
    println!("Top 10 most phonetically similar permutations:\n");
    for (i, result) in results.iter().take(10).enumerate() {
        println!("{}. Domain: {}", i + 1, result.permutation.domain.fqdn);
        println!("   Kind: {:?}", result.permutation.kind);
        println!("   Base encoding: {}", result.data.encodings.domain);
        println!("   Perm encoding: {}", result.data.encodings.permutation);
        println!("   Distance: {:.4}", result.data.distance);
        println!();
    }

    // Convert to JSON to show the output format
    println!("\nExample JSON output for first result:");
    let json = serde_json::to_string_pretty(&results[0]).unwrap();
    println!("{}", json);
}
