use async_trait::async_trait;
use log::{error, info};
use pingora::lb::discovery::ServiceDiscovery;
use pingora::lb::{Backend, Extensions};
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;
use std::collections::{BTreeSet, HashMap};

use crate::services::machine_orchestrator::MachineOrchestrator;

use super::aws_machine_orchestrator::AWSMachineOrchestrator;
use super::utils::create_ec2_client;

pub struct AWSServiceDiscovery;

#[async_trait]
impl ServiceDiscovery for AWSServiceDiscovery {
    async fn discover(&self) -> Result<(BTreeSet<Backend>, HashMap<u64, bool>)> {
        let client = create_ec2_client().await;

        info!("Polling for upstreams");

        let srv = AWSMachineOrchestrator {
            client,
            docker_image: "nginx:latest".into(),
        };

        let machines = srv.list_machines().await.unwrap();

        // let backends = vec!["1.1.1.1"]
        //     .into_iter()
        //     .map(|addr| Backend {
        //         addr: addr.parse::<SocketAddr>().unwrap(),
        //         weight: 1,
        //         ext: Extensions::new(),
        //     })
        //     .collect::<BTreeSet<_>>();
        let backends = machines
            .machines
            .into_iter()
            .map(|x| x.ip_address.parse::<SocketAddr>().unwrap())
            .map(|addr| Backend {
                addr,
                weight: 1,
                ext: Extensions::new(),
            })
            .collect::<BTreeSet<_>>();

        // Check memory/cpu usage, spin up more machines if necessary

        let should_create_machine = true;

        if should_create_machine {
            match srv.create_machine().await {
                Ok(m) => {
                    info!("Machine created, id=\"{}\"", m.0.id);
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            };
        }

        Ok((backends, HashMap::new()))
    }
}
