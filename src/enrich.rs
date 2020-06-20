use dns_lookup::lookup_host;
use rayon::prelude::*;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;

pub enum EnrichmentMode {
    DnsLookup,
    MxCheck,
    SmtpBanner,
    HttpBanner,
    GeoIpLookup,
    WhoIsLookup,
    All,
}

// TODO(jdb): Does it make sense that this container is kept in the
//            library? We need a way to store resolved domains in a
//            thread-safe manner. In this case, we need rayon to be
//            to add all resolved domains in a single container wit-
//            hout having to worry about thread safety as much.
type DomainStore = Arc<Mutex<HashMap<String, DomainMetadata>>>;

/// Container to store interesting FQDN metadata
/// on domains that we resolvable and have some
/// interesting properties.
#[derive(Debug)]
pub struct DomainMetadata {
    ips: Box<Vec<IpAddr>>,
    smtp: Option<SmtpMetadata>,
}

#[derive(Debug)]
struct ResolvedDomain {
    fqdn: String,
    ips: Vec<IpAddr>,
}

#[derive(Debug)]
struct SmtpMetadata {
    // @CLEANUP(jdb): It's not ideal to keep having to duplicate the
    //                fqdn field on each struct here just to be able
    //                to pass it down to rayon...
    fqdn: String,
    is_positive: bool,
    message: String,
}

fn dns_resolvable(addr: String) -> Option<ResolvedDomain> {
    match lookup_host(&addr) {
        Ok(ips) => Some(ResolvedDomain { fqdn: addr, ips }),
        Err(_) => None,
    }
}

fn mx_check(addr: String) -> Option<SmtpMetadata> {
    let email = EmailBuilder::new()
        .to("twistr@example.org")
        .from("twistr@example.com")
        .subject("")
        .text("And that's how the cookie crumbles")
        .build()
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();

    // Send the email
    let result = mailer.send(email.into());

    match result {
        Ok(response) => Some(SmtpMetadata {
            fqdn: addr,
            is_positive: response.is_positive(),
            message: response
                .message
                .into_iter()
                .map(|s| s.to_string())
                .collect::<String>(),
        }),
        Err(_) => None,
    }
}

// @TODO(jdb): Review this function signature a bit more in the future as
//             currently we are able to just pass a vec of domains to it
//             without necessarily having them come from the permutation
//             engine.
//
//             The reasoning behind this is that is a client wants to use
//             the data enrichment _without_ coupling it with the permut-
//             ation engine, they should be able to do so either way.
pub fn enrich<'a>(
    mode: EnrichmentMode,
    domains: Vec<&'a str>,
    domain_store: &'a mut DomainStore,
) -> Result<&'a DomainStore, &'static str> {
    let domains: Vec<String> = domains.into_iter().map(|x| x.to_owned()).collect();

    match mode {
        EnrichmentMode::DnsLookup => {
            let local_copy: Vec<String> = domains.iter().cloned().collect();
            let resolved_domains = local_copy.into_par_iter().filter_map(dns_resolvable);

            // TODO(jdb): See if we can change this to try_for_each instead
            //            so that the closure can return a result and so t-
            //            hat we can use the shorthand `?` instead of tryi-
            //            ng to unwrap.
            resolved_domains.into_par_iter().for_each(|resolved| {
                let mut _domain_store = domain_store.lock().unwrap();
                match _domain_store.get_mut(&resolved.fqdn) {
                    Some(domain_metadata) => {
                        domain_metadata.ips = Box::new(resolved.ips);
                    }
                    None => {
                        let domain_metadata = DomainMetadata {
                            ips: Box::new(resolved.ips),
                            smtp: None,
                        };
                        _domain_store.insert(resolved.fqdn, domain_metadata);
                    }
                }
            });
        }
        EnrichmentMode::MxCheck => {
            // @CLEANUP(jdb): This should iterate over the resolved domains only?
            let local_copy: Vec<String> = domains.iter().cloned().collect();
            let smtp_domains = local_copy.into_par_iter().filter_map(mx_check);

            smtp_domains.into_par_iter().for_each(|smtp_server| {
                let mut _domain_store = domain_store.lock().unwrap();

                match _domain_store.get_mut(&smtp_server.fqdn) {
                    Some(domain_metadata) => {
                        domain_metadata.smtp = Some(SmtpMetadata {
                            fqdn: smtp_server.fqdn.to_string(),
                            is_positive: smtp_server.is_positive,
                            message: smtp_server.message,
                        });
                    }
                    None => {
                        let domain_metadata = DomainMetadata {
                            ips: Box::new(vec![]),
                            smtp: Some(SmtpMetadata {
                                fqdn: smtp_server.fqdn.to_string(),
                                is_positive: smtp_server.is_positive,
                                message: smtp_server.message,
                            }),
                        };

                        // @CLEANUP(jdb): Remove this clone...
                        _domain_store.insert(smtp_server.fqdn.clone(), domain_metadata);
                    }
                }
            });
        }
        _ => return Err("enrichment mode not yet implemented"),
    }

    Ok(domain_store)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_lookup() {
        let mut domain_store = Arc::new(Mutex::new(HashMap::new()));
        assert!(enrich(
            EnrichmentMode::DnsLookup,
            vec!["example.com"],
            &mut domain_store,
        )
        .is_ok())
    }

    #[test]
    fn test_mx_check() {
        let mut domain_store = Arc::new(Mutex::new(HashMap::new()));
        assert!(enrich(
            EnrichmentMode::MxCheck,
            vec!["example.com"],
            &mut domain_store,
        )
        .is_ok())
    }
}
