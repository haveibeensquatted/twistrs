use dns_lookup::lookup_host;
use std::net::IpAddr;
use std::fmt;

use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;

pub type Result<T> = std::result::Result<T, EnrichmentError>;

#[derive(Copy, Clone, Debug)]
pub struct EnrichmentError;

impl fmt::Display for EnrichmentError {

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

    pub fn dns_resolvable(&self) -> Result<DomainMetadata> {
        match lookup_host(&self.fqdn) {
            Ok(ips) => {
                Ok(DomainMetadata {
                    fqdn: self.fqdn.clone(),
                    ips: Some(ips),
                    smtp: None
                })
            },
            Err(e) => {
                // @CLEANUP(JDB): This, is, BAD. This needs to, in the future, first
                //                try to resolve a domain. If this fails, we will g-
                //                et back the following OS Err:
                //
                //                ```
                //                Os {
                //                    code: 11001,
                //                    kind: Other,
                //                    message: "No such host is known.",
                //                }
                //                ```
                // 
                //                Which is an expected Err that we should catch an-
                //                d wrap into an Ok(). Other Errs should be bubble-
                //                d up to the caller.           
                // dbg!(e);
                Err(EnrichmentError)
            },
        }
    }
    
    pub fn mx_check(&self) -> Result<DomainMetadata> {
        let email = EmailBuilder::new()
            .to("twistrs@sample.tst")
            .from("twistrs@sample.tst")
            .subject("")
            .text("And that's how the cookie crumbles\n")
            .build()
            .unwrap();
    
        // Open a local connection on port 25
        let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();
    
        // Send the email
        let result = mailer.send(email.into());
    
        match result {
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
            //                an `std::io::ErrorKind::ConnfectionRefused` which is normal.
            //
            //                In such a scenario, we still do not want to panic but instead
            //                move on. Currently lettre::smtp::error::Error does not suppo-
            //                rt the `fn kind` function to be able to handle error variant-
            //                s. Try to figure out if there is another way to handle them.
            Err(_) => Ok(DomainMetadata{
                fqdn: self.fqdn.clone(),
                ips: None,
                smtp: None
            }),
        }
    }

    // pub fn all(&self) -> Result<Vec<DomainMetadata>> {

    //     // @CLEANUP(JDB): This should use try_join! in the future instead
    //     let result = futures::join!(self.dns_resolvable(), self.mx_check());

    //     Ok(vec![result.0.unwrap(), result.1.unwrap()])
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mx_check() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(domain_metadata.mx_check().is_ok());
    }

    #[test]
    fn test_all_modes() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(domain_metadata.all().is_ok());
    }

    #[test]
    fn test_dns_lookup() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(domain_metadata.dns_resolvable().is_ok());
    }
}
