use clap::{App, Arg};

use twistrs::filter::Permissive;
use twistrs::permutate::Domain;

fn main() {
    let matches = App::new("twistrs-cli-stream-ref")
        .version("0.1.0")
        .author("Juxhin D. Brigjaj <juxhin@phishdeck.com>")
        .about("Example of the allocation-free streaming (cursor) API")
        .arg(Arg::new("domain").required(true))
        .arg(
            Arg::new("limit")
                .long("limit")
                .takes_value(true)
                .help("Stop after N permutations"),
        )
        .get_matches();

    let fqdn = matches.value_of("domain").unwrap();
    let limit = matches
        .value_of("limit")
        .and_then(|v| v.parse::<usize>().ok());

    let domain = Domain::new(fqdn).unwrap();

    let mut yielded = 0usize;
    let mut stream = domain.stream_all(&Permissive);
    while stream.advance() {
        let p = stream.get().unwrap();

        // NOTE: `p.domain.fqdn` borrows from the cursor's internal reusable buffer.
        // If you need to hold onto it (e.g. across an `await`), clone it into an owned `String`.
        println!("{} {:?}", p.domain.fqdn, p.kind);

        yielded += 1;
        if limit.is_some_and(|l| yielded >= l) {
            break;
        }
    }
}

