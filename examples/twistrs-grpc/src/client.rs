use domain_enumeration::domain_enumeration_client::DomainEnumerationClient;
use domain_enumeration::Fqdn;

mod domain_enumeration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Channel::from_static("http://127.0.0.1:8080")
        .connect()
        .await?;

    let mut client = DomainEnumerationClient::new(channel);

    println!("[+] Starting DNS resolutions...");

    let request = tonic::Request::new(Fqdn {
        fqdn: String::from("google.com"),
    });

    let mut response = client.send_dns_resolution(request).await?.into_inner();

    while let Some(res) = response.message().await? {
        println!("Response: {:?}", res);
    }

    println!("[+] Starting MX Checks...");

    let request = tonic::Request::new(Fqdn {
        fqdn: String::from("google.com"),
    });

    let mut response = client.send_mx_check(request).await?.into_inner();

    while let Some(res) = response.message().await? {
        println!("Response: {:?}", res);
    }

    Ok(())
}
