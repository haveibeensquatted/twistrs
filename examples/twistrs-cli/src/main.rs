use clap::{App, Arg};
use colored::*;

use tokio::sync::mpsc;
use twistrs::filter::Permissive;
use twistrs::permutate::Domain;

use anyhow::Result;
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::Instant;

async fn resolve_ips(fqdn: &str) -> std::io::Result<Vec<IpAddr>> {
    let addrs = tokio::net::lookup_host(format!("{}:80", fqdn)).await?;
    Ok(addrs.map(|addr| addr.ip()).collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();

    let matches = App::new("twistrs-cli")
        .version("0.1.0")
        .author("Juxhin D. Brigjaj <juxhin@phishdeck.com>")
        .arg(Arg::new("domain").required(true))
        .get_matches();

    let domain = Domain::new(matches.value_of("domain").unwrap()).unwrap();

    let domain_permutations = domain
        .all(&Permissive)
        .map(|p| p.domain.fqdn)
        .collect::<HashSet<String>>();
    let domain_permutation_count = domain_permutations.len();

    let (tx, mut rx) = mpsc::channel(5000);

    for (i, v) in domain_permutations.into_iter().enumerate() {
        let mut tx = tx.clone();

        tokio::spawn(async move {
            let ips = resolve_ips(&v).await;
            if tx.send((i, v, ips)).await.is_err() {
                println!("received dropped");
                return;
            }

            drop(tx);
        });
    }

    drop(tx);

    let mut enumeration_count = 0;

    while let Some(i) = rx.recv().await {
        if let Ok(ips) = i.2 {
            enumeration_count += 1;
            println!(
                "\n{}\nDomain: {}\n IPs: {:?}",
                "Resolved Domain".bold(),
                &i.1,
                ips
            );
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

    Ok(())
}
