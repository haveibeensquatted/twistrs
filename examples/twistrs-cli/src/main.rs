use clap::{App, Arg};
// use colored::*;

use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

#[tokio::main]
async fn main() {
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

    for domain in domain.all().unwrap() {
        tokio::spawn(async move {
            match DomainMetadata::new(domain).dns_resolvable().await {
                Ok(result) => println!("{:?}", result),
                Err(_) => {}
            }
        });
    }
}
