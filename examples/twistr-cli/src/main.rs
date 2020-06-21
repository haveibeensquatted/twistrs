use clap::{App, Arg};

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

    if matches.is_present("permutation_mode") {
        let permutation_mode = matches
            .value_of("permutation_mode")
            .unwrap()
            .parse::<PermutationMode>()
            .unwrap();

        match domain.permutate(permutation_mode) {
            Ok(permutations) => {
                for permutation in permutations {
                    println!("Generated permutation: {}", permutation);
                }
            }
            Err(e) => panic!(e),
        }
    }
}
