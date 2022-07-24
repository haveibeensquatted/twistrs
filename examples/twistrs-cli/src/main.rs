use clap::{App, Arg};
use colored::*;

use tokio::sync::mpsc;
use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

use std::collections::HashSet;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start_time = Instant::now();

    let matches = App::new("twistrs-cli")
        .version("0.1.0")
        .author("Juxhin D. Brigjaj <juxhin@phishdeck.com>")
        .arg(Arg::new("domain").required(true))
        .get_matches();

    let domain = Domain::new(matches.value_of("domain").unwrap()).unwrap();

    let mut domain_permutations = domain.all().collect::<HashSet<String>>();
    let domain_permutation_count = domain_permutations.len();

    domain_permutations.insert(String::from(&(*domain.fqdn)));

    let (tx, mut rx) = mpsc::channel(5000);

    for (i, v) in domain_permutations.into_iter().enumerate() {
        let domain_metadata = DomainMetadata::new(v.clone());
        let mut tx = tx.clone();

        tokio::spawn(async move {
            if tx
                .send((i, v.clone(), domain_metadata.dns_resolvable().await))
                .await
                .is_err()
            {
                println!("received dropped");
                return;
            }

            drop(tx);
        });
    }

    drop(tx);

    let mut enumeration_count = 0;

    while let Some(i) = rx.recv().await {
        if let Ok(v) = i.2 {
            if v.ips.is_some() {
                enumeration_count += 1;
                println!(
                    "\n{}\nDomain: {}\n IPs: {:?}",
                    "Enriched Domain".bold(),
                    &v.fqdn,
                    &v.ips
                );
            }
        }
    }

    println!(
        "\n{}: {}",
        "Total number of unique domain permutations generated".bold(),
        domain_permutation_count.to_string().cyan()
    );

    println!(
        "{}: {}",
        "Total number of domains enriched".bold(),
        enumeration_count.to_string().cyan()
    );

    println!(
        "{}: {} seconds",
        "Execution time".bold(),
        start_time.elapsed().as_secs()
    );
}
