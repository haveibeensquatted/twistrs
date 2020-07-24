use clap::{App, Arg};
// use colored::*;

use twistrs::permutate::Domain;
use twistrs::enrich::DomainMetadata;

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

    let generated_domains = domain.all().unwrap();

    for generated_domain in generated_domains {
        match DomainMetadata::new(generated_domain).dns_resolvable().await {
            Ok(result) => println!("{:?}", result),
            Err(_) => {},
        }
    }
}
