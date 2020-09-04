use std::net::IpAddr;
use std::fmt;

use tokio::net;

use async_smtp::{
    ClientSecurity, Envelope, SendableEmail, SmtpClient
};


pub type Result<T> = std::result::Result<T, EnrichmentError>;

#[derive(Copy, Clone, Debug)]
pub struct EnrichmentError;

impl fmt::Display for EnrichmentError {

    // @CLEANUP(jdb): Make this something meaningful, if it needs to be
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

/// Container to store interesting FQDN metadata
/// on domains that we resolvable and have some
/// interesting properties.
#[derive(Debug, Clone)]
pub struct DomainMetadata {
    pub fqdn: String,
    pub ips: Option<Vec<IpAddr>>,
    pub smtp: Option<SmtpMetadata>,
}

#[derive(Debug, Clone)]
pub struct SmtpMetadata {
    is_positive: bool,
    message: String,
}

impl DomainMetadata {
    pub fn new(fqdn: String) -> DomainMetadata {
        DomainMetadata {
            fqdn: fqdn,
            ips: None,
            smtp: None,
        }
    }

    pub async fn dns_resolvable(&self) -> Result<DomainMetadata> {
        match net::lookup_host(&format!("{}:80", self.fqdn)[..]).await {
            Ok(addrs) => {
                Ok(DomainMetadata {
                    fqdn: self.fqdn.clone(),
                    ips: Some(addrs.map(|addr| addr.ip()).collect()),
                    smtp: None
                })
            },
            Err(_) => {
                Err(EnrichmentError)
            }
        }
    }
    
    pub async fn mx_check(&self) -> Result<DomainMetadata> {

        let email = SendableEmail::new(
            Envelope::new(
                Some("twistrs@example.com".parse().unwrap()),
                vec!["twistrs@example.com".parse().unwrap()],
            ).unwrap(),
            "Twistrs", 
            "And that's how the cookie crumbles\n",
        );

        let smtp_domain = format!("{}:25", &self.fqdn);

        match SmtpClient::with_security(smtp_domain.clone(), ClientSecurity::None).await {

            // TODO(jdb): Figure out how to clean this up
            Ok(smtp) => {
                match smtp.into_transport().connect_and_send(email).await {
                    Ok(response) => {
                        Ok(DomainMetadata {
                            fqdn: self.fqdn.clone(),
                            ips: None,
                            smtp: Some(SmtpMetadata {
                                is_positive: response.is_positive(),
                                message: response
                                    .message
                                    .into_iter()
                                    .map(|s| s.to_string())
                                    .collect::<String>(),
                            })
                        })
                    },

                    // @CLEANUP(JDB): Currently for most scenarios, the following call with return
                    //                an `std::io::ErrorKind::ConnectionRefused` which is normal.
                    //
                    //                In such a scenario, we still do not want to panic but instead
                    //                move on. Currently lettre::smtp::error::Error does not suppo-
                    //                rt the `fn kind` function to be able to handle error variant-
                    //                s. Try to figure out if there is another way to handle them.
                    Err(e) => {
                        dbg!(e);
                        Err(EnrichmentError)
                    },
                }
            },
            Err(_) => {
                Ok(DomainMetadata{
                    fqdn: self.fqdn.clone(),
                    ips: None,
                    smtp: None
                })                
            }
        }
    }

    pub async fn all(&self) -> Result<Vec<DomainMetadata>> {

        // @CLEANUP(JDB): This should use try_join! in the future instead
        let result = futures::join!(self.dns_resolvable(), self.mx_check());

        Ok(vec![result.0.unwrap(), result.1.unwrap()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;


    #[test]
    fn test_mx_check() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(block_on(domain_metadata.mx_check()).is_ok());
    }

    #[test]
    fn test_all_modes() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(block_on(domain_metadata.all()).is_ok());
    }

    #[test]
    fn test_dns_lookup() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(block_on(domain_metadata.dns_resolvable()).is_ok());
    }
}
