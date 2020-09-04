use tokio::sync::mpsc;

use std::collections::HashSet;

use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;


#[tokio::main]
async fn main() {
    let domain = Domain::new("google.com").unwrap();

    let mut _permutations = domain.all().unwrap().collect::<HashSet<String>>();
    _permutations.insert(String::from(domain.fqdn.clone()));

    let (tx, mut rx) = mpsc::channel(5000);

    for (i, v) in _permutations.into_iter().enumerate() {
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

    while let Some(i) = rx.recv().await {
        match i.2 {
            Ok(v) => println!("got: {:?}", v),
            Err(_) => {},
        }
    }
}
