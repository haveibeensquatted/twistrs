use colored::*;
use clap::{App, Arg};

use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;
use tokio::sync::mpsc;

use std::time::Instant;
use std::collections::HashSet;


#[tokio::main]
async fn main() {
    let start_time = Instant::now();

    let matches = App::new("twistrs-cli")
        .version("0.1.0")
        .author("Juxhin D. Brigjaj <juxhin@phishdeck.com>")
        .arg(
            Arg::new("domain")
                .about("domain to permutate and enrich")
                .required(true),
        )
        .get_matches();

    let domain = Domain::new(&matches.value_of("domain").unwrap()).unwrap();

    let mut domain_permutations = domain.all().unwrap().collect::<HashSet<String>>();
    let domain_permutation_count = domain_permutations.len();

    domain_permutations.insert(String::from(domain.fqdn.clone()));

    let (tx, mut rx) = mpsc::channel(5000);

    for (i, v) in domain_permutations.into_iter().enumerate() {
        let domain_metadata = DomainMetadata::new(v.clone());
        let mut tx = tx.clone();

        tokio::spawn(async move {
            if let Err(_) = tx.send((i, v.clone(), domain_metadata.dns_resolvable().await)).await {
                println!("received dropped");
                return;
            }

            drop(tx);
        });
    }

    drop(tx);

    let mut enumeration_count = 0;

    while let Some(i) = rx.recv().await {
        match i.2 {
            Ok(v) => {
                match v.ips {
                    Some(_) => {
                        enumeration_count += 1;
                        println!(
                            "\n{}\nDomain: {}\n IPs: {:?}",
                            "Enriched Domain".bold(),
                            &v.fqdn,
                            &v.ips
                        );
                    },
                    None => {},
                }
            },
            Err(_) => {},
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
