use clap::{App, Arg};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use twistrs::enrich::{enrich, DomainStore, EnrichmentMode};
use twistrs::permutate::{Domain, PermutationMode};

fn main() {
    let matches = App::new("twistrs-cli")
        .version("0.1.0")
        .author("Juxhin D. Brigjaj <juxhin@phishdeck.com>")
        .arg(
            Arg::with_name("permutation_mode")
                .long("permutation_mode")
                .required(false)
                .multiple(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("enrichment_mode")
                .long("enrichment_mode")
                .required(false)
                .multiple(false)
                .takes_value(true),
        )
        .arg(
            Arg::new("domain")
                .about("domain to permutate and enrich")
                .required(true),
        )
        .get_matches();

    let domain = Domain::new(&matches.value_of("domain").unwrap()).unwrap();

    let mut generated_domains = vec![];

    if matches.is_present("permutation_mode") {
        let permutation_mode = matches
            .value_of("permutation_mode")
            .unwrap()
            .parse::<PermutationMode>()
            .unwrap();

        match domain.permutate(permutation_mode) {
            Ok(permutations) => {
                generated_domains = permutations;
            }
            Err(e) => panic!(e),
        }

        println!(
            "{}",
            format!(
                "Generated {} domains...",
                &generated_domains.len().to_string()
            )
        );
    }

    if matches.is_present("enrichment_mode") {
        let enrichment_mode = matches
            .value_of("enrichment_mode")
            .unwrap()
            .parse::<EnrichmentMode>()
            .unwrap();

        println!("Enriching all domains...");
        println!("Applying enrichment mode: {:?}", enrichment_mode);

        let mut domain_store: DomainStore = Arc::new(Mutex::new(HashMap::new()));

        enrich(enrichment_mode, generated_domains, &mut domain_store).unwrap();

        for (domain, domain_metadata) in domain_store.lock().unwrap().iter() {
            println!("Enriched Domain: {}", domain);
            print!("\tIPs Found: ");

            for ip in domain_metadata.ips.iter() {
                print!("\n\t  - {}", ip);
            }

            print!("\n");
            println!("\tSMTP Listener? (MX Check): {:?}", domain_metadata.smtp);
            println!("\n");
        }
    }
}
