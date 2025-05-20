use async_trait::async_trait;
use log::{error, info};
use pingora::lb::Backend;
use pingora::lb::discovery::ServiceDiscovery;
use pingora::prelude::*;
use pingora::services::background::BackgroundService;
use std::collections::{BTreeSet, HashMap};
use std::time::Duration;

use crate::services::discovery::ServiceDiscoveryConfig;
use crate::services::machine_orchestrator::MachineOrchestrator;

use super::aws_machine_orchestrator::AWSMachineOrchestrator;
use super::utils::create_ec2_client;

pub struct AWSServiceDiscovery(ServiceDiscoveryConfig);

impl AWSServiceDiscovery {
    pub fn new(config: ServiceDiscoveryConfig) -> Self {
        Self(config)
    }
}

impl BackgroundService for AWSServiceDiscovery {
    fn start<'life0, 'async_trait>(
        &'life0 self,
        shutdown: pingora::server::ShutdownWatch,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let fut = async move {
            loop {
                if *shutdown.borrow() {
                    break;
                }

                sleep(Duration::from_secs(5)).await;

                self.discover().await.unwrap();

                info!("Polling");
            }
        };

        Box::pin(fut)
    }
}

#[async_trait]
impl ServiceDiscovery for AWSServiceDiscovery {
    async fn discover(&self) -> Result<(BTreeSet<Backend>, HashMap<u64, bool>)> {
        let client = create_ec2_client().await;

        info!("Polling for upstreams");

        let srv = AWSMachineOrchestrator {
            client,
            docker_image: self.0.docker_image.clone(),
        };

        let machines = srv.list_machines().await;

        if let Err(e) = &machines {
            error!("{:?}", e);
        }

        let mut backends = machines
            .unwrap()
            .machines
            .into_iter()
            .map(|x| Backend::try_from(x).unwrap())
            .collect::<BTreeSet<_>>();

        info!("Backends: {}", backends.len());

        // Check memory/cpu usage, spin up more machines if necessary

        let should_create_machine = backends.is_empty();

        if should_create_machine {
            match srv.create_machine().await {
                Ok(m) => {
                    info!("Machine created, id=\"{}\"", m.0.id);
                    backends.insert(Backend::try_from(m.0).unwrap());
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            };
        }

        Ok((backends, HashMap::new()))
    }
}
