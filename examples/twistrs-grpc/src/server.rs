mod domain_enumeration;

use tokio::sync::mpsc;

use tonic::{transport::Server, Request, Response, Status};

use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

use domain_enumeration::domain_enumeration_server::{DomainEnumeration, DomainEnumerationServer};
use domain_enumeration::{DomainEnumerationResponse, Fqdn, MxCheckResponse};

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

        for permutation in Domain::new(&request.get_ref().fqdn).unwrap().all().unwrap() {
            let domain_metadata = DomainMetadata::new(permutation.clone());
            let mut tx = tx.clone();

            // Spawn DNS Resolution check
            tokio::spawn(async move {
                if let Ok(metadata) = domain_metadata.dns_resolvable().await {
                    if let Some(ips) = metadata.ips {
                        if tx
                            .send(Ok(DomainEnumerationResponse {
                                fqdn: permutation.clone().to_string(),
                                ips: ips.into_iter().map(|x| format!("{}", x)).collect(),
                            }))
                            .await
                            .is_err()
                        {
                            println!("receiver dropped");
                            return;
                        }
                    }
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

        for permutation in Domain::new(&request.get_ref().fqdn).unwrap().all().unwrap() {
            let domain_metadata = DomainMetadata::new(permutation.clone());
            let mut tx = tx.clone();

            // Spawn DNS Resolution check
            tokio::spawn(async move {
                if let Ok(metadata) = domain_metadata.mx_check().await {
                    if let Some(smtp) = metadata.smtp {
                        if tx
                            .send(Ok(MxCheckResponse {
                                fqdn: permutation.clone().to_string(),
                                is_positive: smtp.is_positive,
                                message: smtp.message,
                            }))
                            .await
                            .is_err()
                        {
                            println!("receiver dropped");
                            return;
                        }
                    }
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
