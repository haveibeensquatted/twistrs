use dns_lookup::lookup_host;
use std::net::IpAddr;
use std::fmt;

use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;


type Result<T> = std::result::Result<T, EnrichmentError>;

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
#[derive(Debug)]
pub struct DomainMetadata<'a> {
    pub fqdn: &'a str,
    pub ips: Option<Vec<IpAddr>>,
    pub smtp: Option<SmtpMetadata>,
}

#[derive(Debug)]
struct ResolvedDomain {
    ips: Vec<IpAddr>,
}

#[derive(Debug)]
pub struct SmtpMetadata {
    is_positive: bool,
    message: String,
}

impl<'a> DomainMetadata<'a> {
    pub fn new(fqdn: &'a str) -> DomainMetadata<'a> {
        DomainMetadata {
            fqdn: fqdn,
            ips: None,
            smtp: None,
        }
    }

    pub fn dns_resolvable(&mut self) -> Result<&DomainMetadata> {
        match lookup_host(&self.fqdn) {
            Ok(ips) => {
                self.ips =  Some(ips);
                Ok(self)
            },
            Err(_) => Err(EnrichmentError),
        }
    }
    
    pub fn mx_check(&mut self) -> Result<&DomainMetadata> {
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
                self.smtp = Some(SmtpMetadata {
                    is_positive: response.is_positive(),
                    message: response
                        .message
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect::<String>(),
                });

                Ok(self)
            },

            // @CLEANUP(JDB): Currently for most scenarios, the following call with return
            //                an `std::io::ErrorKind::ConnfectionRefused` which is normal.
            //
            //                In such a scenario, we still do not want to panic but instead
            //                move on. Currently lettre::smtp::error::Error does not suppo-
            //                rt the `fn kind` function to be able to handle error variant-
            //                s. Try to figure out if there is another way to handle them.
            Err(_) => Ok(self),
        }
    }

    pub fn all(&mut self) -> Result<&DomainMetadata> {
        &self.dns_resolvable();
        &self.mx_check();
        
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn test_mx_check() {
        let mut domain_metadata = DomainMetadata::new("example.com");
        assert!(domain_metadata.mx_check().is_ok());
    }

    #[test]
    fn test_all_modes() {
        let mut domain_metadata = DomainMetadata::new("example.com");
        assert!(domain_metadata.all().is_ok());
    }

    #[test]
    fn test_dns_lookup() {
        let mut domain_metadata = DomainMetadata::new("example.com");
        assert!(domain_metadata.dns_resolvable().is_ok());
    }
}
