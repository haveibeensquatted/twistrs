//! Example demonstrating phonetic distance for a specific permutation
//! 
//! This example matches the use case from the problem statement:
//! Compare example.com and esample.com

use twistrs::{
    compute_phonetic_distance,
    permutate::{Domain, Permutation, PermutationKind},
};

fn main() {
    // Base domain: example.com
    let base = Domain::new("example.com").unwrap();
    
    // Permutation: esample.com  
    let perm_domain = Domain::new("esample.com").unwrap();
    let permutation = Permutation {
        domain: perm_domain,
        kind: PermutationKind::Mapped,
    };

    // Compute phonetic distance
    let result = compute_phonetic_distance(&base, &permutation);

    // Display results
    println!("Base domain: {}", base.fqdn);
    println!("Base label: {}", base.domain);
    println!();
    println!("Permutation domain: {}", result.permutation.domain.fqdn);
    println!("Permutation label: {}", result.permutation.domain.domain);
    println!();
    println!("Operation: {}", result.op);
    println!();
    println!("Selected encodings:");
    println!("  Base encoding: {}", result.data.encodings.domain);
    println!("  Perm encoding: {}", result.data.encodings.permutation);
    println!();
    println!("Distance: {:.4} (0.0 = most similar, 1.0 = most different)", result.data.distance);
    println!();
    
    // Show JSON output
    println!("JSON output:");
    let json = serde_json::to_string_pretty(&result).unwrap();
    println!("{}", json);
    
    // Another example with phonetically identical domains
    println!("\n\n--- Another example: phone.com vs fone.com ---\n");
    
    let base2 = Domain::new("phone.com").unwrap();
    let perm_domain2 = Domain::new("fone.com").unwrap();
    let permutation2 = Permutation {
        domain: perm_domain2,
        kind: PermutationKind::Mapped,
    };
    
    let result2 = compute_phonetic_distance(&base2, &permutation2);
    
    println!("Base: {} -> Encoding: {}", base2.domain, result2.data.encodings.domain);
    println!("Perm: {} -> Encoding: {}", result2.permutation.domain.domain, result2.data.encodings.permutation);
    println!("Distance: {:.4}", result2.data.distance);
    println!("(Note: 'phone' and 'fone' sound similar, so they have low distance)");
}
