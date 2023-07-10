//! The enrichment module exposes functionality to enrich
//! a given domain with interesting metadata. Currently
//! including:
//!
//! * DNS resolution (through HTTP/80 lookup).
//! * Open SMTP server (for email misdirects).
//!
//! Example:
//!
//! ```
//! use twistrs::enrich::DomainMetadata;
//!
//! #[tokio::main]
//! async fn main() {
//!     let domain_metadata = DomainMetadata::new(String::from("google.com"));
//!     domain_metadata.dns_resolvable().await;
//! }
//! ```
//!
//! Note that the enrichment module is independent from the
//! permutation module and can be used with any given FQDN.

use serde::Serialize;
use std::fmt;
use std::net::IpAddr;

#[cfg(feature = "geoip_lookup")]
use maxminddb;
#[cfg(feature = "geoip_lookup")]
use maxminddb::geoip2;

#[cfg(feature = "whois_lookup")]
use whois_rust::WhoIsLookupOptions;

#[cfg(feature = "smtp_lookup")]
use async_smtp::{ClientSecurity, Envelope, SendableEmail, ServerAddress, SmtpClient};
use hyper::{Body, Request};
use tokio::net;

use crate::constants::HTTP_CLIENT;

#[cfg(feature = "whois_lookup")]
use crate::constants::WHOIS;

/// Temporary type-alias over `EnrichmentError`.
pub type Result<T> = std::result::Result<T, EnrichmentError>;

#[derive(Copy, Clone, Debug)]
#[deprecated(
    since = "0.1.0",
    note = "Prone to be removed in the future, does not currently provide any context."
)]
pub struct EnrichmentError;

impl fmt::Display for EnrichmentError {
    // @CLEANUP(jdb): Make this something meaningful, if it needs to be
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

/// Container to store interesting FQDN metadata
/// on domains that we resolve.
///
/// Whenever any domain enrichment occurs, the
/// following struct is return to indicate the
/// information that was derived.
///
/// **N.B**—there will be cases where a single
/// domain can have multiple `DomainMetadata`
/// instancees associated with it.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DomainMetadata {
    /// The domain that is being enriched.
    pub fqdn: String,

    /// Any IPv4 and IPv6 ips that were discovered during
    /// domain resolution.
    pub ips: Option<Vec<IpAddr>>,

    /// Any SMTP message data (if any) that was returned by
    /// an SMTP server.
    pub smtp: Option<SmtpMetadata>,

    /// HTTP server banner data extracted.
    pub http_banner: Option<String>,

    /// IP addresses resolved through GeoIP lookup to City, Country, Continent.
    pub geo_ip_lookups: Option<Vec<(IpAddr, String)>>,

    /// Block of text returned by the WhoIs registrar.
    pub who_is_lookup: Option<String>,
}

/// SMTP specific metadata generated by a partic
/// ular domain.
#[derive(Debug, Clone, Serialize)]
pub struct SmtpMetadata {
    /// Whether the email was dispatched successfully
    pub is_positive: bool,

    /// Message received back from the SMTP server
    pub message: String,
}

impl DomainMetadata {
    /// Create a new empty state for a particular FQDN.
    pub fn new(fqdn: String) -> DomainMetadata {
        DomainMetadata {
            fqdn,
            ..Default::default()
        }
    }

    /// Asynchronous DNS resolution on a `DomainMetadata` instance.
    ///
    /// Returns `Ok(DomainMetadata)` is the domain was resolved,
    /// otherwise returns `Err(EnrichmentError)`.
    ///
    /// **N.B**—also host lookups are done over port 80.
    pub async fn dns_resolvable(&self) -> Result<DomainMetadata> {
        match net::lookup_host(&format!("{}:80", self.fqdn)[..]).await {
            Ok(addrs) => Ok(DomainMetadata {
                fqdn: self.fqdn.clone(),
                ips: Some(addrs.map(|addr| addr.ip()).collect()),
                smtp: None,
                http_banner: None,
                geo_ip_lookups: None,
                who_is_lookup: None,
            }),
            Err(_) => Err(EnrichmentError),
        }
    }

    /// Asynchronous SMTP check. Attempts to establish an SMTP
    /// connection to the FQDN on port 25 and send a pre-defi
    /// ned email.
    ///
    /// Currently returns `Ok(DomainMetadata)` always, which
    /// internally contains `Option<SmtpMetadata>`. To check
    /// if the SMTP relay worked, check that
    /// `DomainMetadata.smtp` is `Some(v)`.
    #[cfg(feature = "smtp_lookup")]
    pub async fn mx_check(&self) -> Result<DomainMetadata> {
        let email = SendableEmail::new(
            Envelope::new(
                Some("twistrs@example.com".parse().unwrap()),
                vec!["twistrs@example.com".parse().unwrap()],
            )
            .unwrap(),
            "Twistrs",
            "And that's how the cookie crumbles\n",
        );

        let smtp = SmtpClient::with_security(
            ServerAddress {
                host: self.fqdn.to_string(),
                port: 25,
            },
            ClientSecurity::None,
        );

        match smtp.into_transport().connect_and_send(email).await {
            Ok(response) => Ok(DomainMetadata {
                fqdn: self.fqdn.clone(),
                ips: None,
                smtp: Some(SmtpMetadata {
                    is_positive: response.is_positive(),
                    message: response.message.into_iter().collect::<String>(),
                }),
                http_banner: None,
                geo_ip_lookups: None,
                who_is_lookup: None,
            }),

            // @CLEANUP(JDB): Currently for most scenarios, the following call with return
            //                an `std::io::ErrorKind::ConnectionRefused` which is normal.
            //
            //                In such a scenario, we still do not want to panic but instead
            //                move on. Currently lettre::smtp::error::Error does not support
            //                the `fn kind` function to be able to handle error variants.
            //
            //                Try to figure out if there is another way to handle them.
            Err(_) => Ok(DomainMetadata::new(self.fqdn.clone())),
        }
    }

    /// Asynchronous HTTP Banner fetch. Searches and parses `server` header
    /// from an HTTP request to gather the HTTP banner.
    ///
    /// Note that a `HEAD` request is issued to minimise bandwidth. Also note
    /// that the internal [`HttpConnector`](https://docs.rs/hyper/0.13.8/hyper/client/struct.HttpConnector.html)
    /// sets the response buffer window to 1024 bytes, the CONNECT timeout to
    /// 5s and enforces HTTP scheme.
    ///
    /// ```
    /// use twistrs::enrich::DomainMetadata;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let domain_metadata = DomainMetadata::new(String::from("www.phishdeck.com"));
    ///     println!("{:?}", domain_metadata.http_banner().await);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Currently panics if the HTTP server header value is not parseable.
    /// For more information please refer to the
    /// [Hyper implementation](https://docs.rs/hyper/hyper/header/struct.HeaderValue.html#method.to_str).
    pub async fn http_banner(&self) -> Result<DomainMetadata> {
        // Construst the basic request to be sent out
        let request = Request::builder()
            .method("HEAD")
            .uri(format!("http://{}", &self.fqdn))
            .header("User-Agent", "github-twistrs-http-banner/1.0")
            .body(Body::from("")) // This is annoying
            .unwrap();

        match HTTP_CLIENT.request(request).await {
            Ok(response) => match response.headers().get("server") {
                Some(server) => Ok(DomainMetadata {
                    fqdn: self.fqdn.clone(),
                    ips: None,
                    smtp: None,
                    http_banner: Some(String::from(server.to_str().unwrap())),
                    geo_ip_lookups: None,
                    who_is_lookup: None,
                }),
                None => Ok(DomainMetadata::new(self.fqdn.clone())),
            },
            Err(_) => Ok(DomainMetadata::new(self.fqdn.clone())),
        }
    }

    /// Asynchronous cached GeoIP lookup. Interface deviates from the usual enrichment
    /// interfaces and requires the callee to pass a [`maxminddb::Reader`](https://docs.rs/maxminddb/0.15.0/maxminddb/struct.Reader.html)
    /// to perform the lookup through. Internally, the maxminddb call is blocking and
    /// may result in performance drops, however the lookups are in-memory.
    ///
    /// The only reason you would want to do this, is to be able to get back a `DomainMetadata`
    /// to then process as you would with other enrichment methods. Internally the lookup will
    /// try to stitch together the City, Country & Continent that the [`IpAddr`](https://doc.rust-lang.org/std/net/enum.IpAddr.html)
    /// resolves to.
    ///
    /// ```
    /// use maxminddb::Reader;
    /// use twistrs::enrich::DomainMetadata;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let reader = maxminddb::Reader::open_readfile("./data/MaxMind-DB/test-data/GeoIP2-City-Test.mmdb").unwrap();
    ///     let domain_metadata = DomainMetadata::new(String::from("www.phishdeck.com"));
    ///     println!("{:?}", domain_metadata.geoip_lookup(&reader).await);
    /// }
    /// ```
    ///
    /// ### Panics
    ///
    /// Currently assumes that if a City/Country/Continent is found, that the English ("en")
    /// result is available.
    ///
    /// ### Features
    ///
    /// This function requires the `geoip_lookup` feature toggled.
    #[cfg(feature = "geoip_lookup")]
    pub async fn geoip_lookup(&self, geoip: &maxminddb::Reader<Vec<u8>>) -> Result<DomainMetadata> {
        let mut result: Vec<(IpAddr, String)> = Vec::new();

        match &self.ips {
            Some(ips) => {
                for ip in ips.iter() {
                    match geoip.lookup::<geoip2::City>(*ip) {
                        Ok(lookup_result) => {
                            let mut geoip_string = String::new();

                            if lookup_result.city.is_some() {
                                geoip_string.push_str(
                                    lookup_result
                                        .city
                                        .unwrap()
                                        .names
                                        .unwrap()
                                        .get("en")
                                        .unwrap(),
                                );
                            }

                            if lookup_result.country.is_some() {
                                if geoip_string.len() > 0 {
                                    geoip_string.push_str(", ");
                                }

                                geoip_string.push_str(
                                    lookup_result
                                        .country
                                        .unwrap()
                                        .names
                                        .unwrap()
                                        .get("en")
                                        .unwrap(),
                                );
                            }

                            if lookup_result.continent.is_some() {
                                if geoip_string.len() > 0 {
                                    geoip_string.push_str(", ");
                                }

                                geoip_string.push_str(
                                    lookup_result
                                        .continent
                                        .unwrap()
                                        .names
                                        .unwrap()
                                        .get("en")
                                        .unwrap(),
                                );
                            }

                            result.push((*ip, geoip_string));
                        }
                        Err(_) => {}
                    }
                }

                Ok(DomainMetadata::new(self.fqdn.clone()))
            }
            None => Ok(DomainMetadata::new(self.fqdn.clone())),
        }
    }

    /// Asyncrhonous WhoIs lookup using cached WhoIs server config. Note that
    /// the internal lookups are not async and so this should be considered
    /// a heavy/slow call.
    ///
    /// ```
    /// use twistrs::enrich::DomainMetadata;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let domain_metadata = DomainMetadata::new(String::from("www.phishdeck.com"));
    ///     println!("{:?}", domain_metadata.whois_lookup().await);
    /// }
    /// ```
    ///
    /// ### Features
    ///
    /// This function requires the `whois_lookup` feature toggled.
    #[cfg(feature = "whois_lookup")]
    pub async fn whois_lookup(&self) -> Result<DomainMetadata> {
        let mut result = DomainMetadata::new(self.fqdn.clone());

        match WhoIsLookupOptions::from_string(&self.fqdn) {
            Ok(mut lookup_options) => {
                lookup_options.timeout = Some(std::time::Duration::from_secs(5)); // Change default timeout from 60s to 5s
                lookup_options.follow = 1; // Only allow at most one redirect

                match WHOIS.lookup(lookup_options) {
                    Ok(lookup_result) => {
                        result.who_is_lookup = Some(String::from(
                            &lookup_result
                                .split("\r\n")
                                // The only entries we care about are the ones that start with 3 spaces.
                                // Ideally the whois_rust library would have parsed this nicely for us.
                                .filter(|s| s.starts_with("   "))
                                .collect::<Vec<&str>>()
                                .join("\n"),
                        ));
                    }
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }

        Ok(result)
    }

    /// Performs all FQDN enrichment methods on a given FQDN.
    /// This is the only function that returns a `Vec<DomainMetadata>`.
    ///
    /// **N.B** -- this is currently very slow, and serializes the
    /// operations rather than running them concurrently. It should
    /// only be used for testing or debugging purposes.
    ///
    /// # Panics
    ///
    /// Currently panics if any of the internal enrichment methods returns
    /// an Err.
    pub async fn all(&self) -> Result<Vec<DomainMetadata>> {
        // @CLEANUP(JDB): This should use try_join! in the future instead

        #[cfg(feature = "smtp_lookup")]
        let mx_check = self.mx_check();

        let result = futures::join!(self.dns_resolvable(), self.http_banner());

        Ok(vec![
            result.0.unwrap(),
            #[cfg(feature = "smtp_lookup")]
            mx_check.await.unwrap(),
            result.1.unwrap(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "geoip_lookup")]
    use maxminddb;

    use futures::executor::block_on;

    #[tokio::test]
    async fn test_all_modes() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(block_on(domain_metadata.all()).is_ok());
    }

    #[tokio::test]
    async fn test_mx_check() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(block_on(domain_metadata.mx_check()).is_ok());
    }

    #[tokio::test]
    async fn test_dns_lookup() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(block_on(domain_metadata.dns_resolvable()).is_ok());
    }

    #[tokio::test]
    async fn test_http_banner() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(domain_metadata.http_banner().await.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "geoip_lookup")]
    async fn test_geoip_lookup() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"))
            .dns_resolvable()
            .await
            .unwrap();

        // MaxmindDB CSV entry for example.com subnet, prone to failure but saves space
        let reader =
            maxminddb::Reader::open_readfile("./data/MaxMind-DB/test-data/GeoIP2-City-Test.mmdb")
                .unwrap();

        assert!(domain_metadata.geoip_lookup(&reader).await.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "whois_lookup")]
    async fn test_whois_lookup() {
        let domain_metadata = DomainMetadata::new(String::from("example.com"));
        assert!(domain_metadata.whois_lookup().await.is_ok());
    }
}
