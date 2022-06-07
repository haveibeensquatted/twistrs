use tokio::sync::mpsc;

use tonic::{transport::Server, Request, Response, Status};

use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

use domain_enumeration::domain_enumeration_server::{DomainEnumeration, DomainEnumerationServer};

use domain_enumeration::{DomainEnumerationResponse, Fqdn, MxCheckResponse};

mod domain_enumeration;

#[derive(Default)]
pub struct DomainEnumerationService {}

#[tonic::async_trait]
impl DomainEnumeration for DomainEnumerationService {
    type SendDnsResolutionStream = mpsc::Receiver<Result<DomainEnumerationResponse, Status>>;
    type SendMxCheckStream = mpsc::Receiver<Result<MxCheckResponse, Status>>;

    async fn send_dns_resolution(
        &self,
        request: Request<Fqdn>,
    ) -> Result<Response<Self::SendDnsResolutionStream>, Status> {
        let (tx, rx) = mpsc::channel(64);

        for permutation in Domain::new(&request.get_ref().fqdn).unwrap().all() {
            let domain_metadata = DomainMetadata::new(permutation.clone());
            let mut tx = tx.clone();

            // Spawn DNS Resolution check
            tokio::spawn(async move {
                match domain_metadata.dns_resolvable().await {
                    Ok(metadata) => match metadata.ips {
                        Some(ips) => {
                            if let Err(_) = tx
                                .send(Ok(DomainEnumerationResponse {
                                    fqdn: format!("{}", permutation.clone()),
                                    ips: ips.into_iter().map(|x| format!("{}", x)).collect(),
                                }))
                                .await
                            {
                                println!("receiver dropped");
                                return;
                            }
                        }
                        None => {}
                    },
                    Err(_) => {}
                }

                drop(tx);
            });
        }

        drop(tx);

        Ok(Response::new(rx))
    }

    async fn send_mx_check(
        &self,
        request: Request<Fqdn>,
    ) -> Result<Response<Self::SendMxCheckStream>, Status> {
        let (tx, rx) = mpsc::channel(64);

        for permutation in Domain::new(&request.get_ref().fqdn).unwrap().all() {
            let domain_metadata = DomainMetadata::new(permutation.clone());
            let mut tx = tx.clone();

            // Spawn DNS Resolution check
            tokio::spawn(async move {
                match domain_metadata.mx_check().await {
                    Ok(metadata) => match metadata.smtp {
                        Some(smtp) => {
                            if let Err(_) = tx
                                .send(Ok(MxCheckResponse {
                                    fqdn: format!("{}", permutation.clone()),
                                    is_positive: smtp.is_positive,
                                    message: smtp.message,
                                }))
                                .await
                            {
                                println!("receiver dropped");
                                return;
                            }
                        }
                        None => {}
                    },
                    Err(_) => {}
                }

                drop(tx);
            });
        }

        drop(tx);

        Ok(Response::new(rx))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();

    let rpc_service = DomainEnumerationService::default();

    println!("[+] Listening on {}", addr);

    Server::builder()
        .add_service(DomainEnumerationServer::new(rpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
