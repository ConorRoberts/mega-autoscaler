use super::super::machine_orchestrator::{
    CreateMachineResponse, ListMachinesResponse, Machine, MachineOrchestrator,
};
use super::utils::create_ec2_client;
use aws_sdk_ec2 as ec2;
use aws_sdk_ec2::types::{Filter, InstanceType, ResourceType, Tag, TagSpecification};
use log::{error, info};
use pingora::prelude::*;
use pingora::services::Service;
use std::time::Duration;

enum Ami {
    /// (04-05-2025) Amazon Linux 2, 64 bit, ARM
    AmazonLinux64BitArm,
}

impl ToString for Ami {
    fn to_string(&self) -> String {
        match self {
            Self::AmazonLinux64BitArm => "ami-0400ee32fb141782f".into(),
        }
    }
}

/// Background process to monitor machines
pub struct AWSMachineService {
    pub polling_interval: Duration,
}

pub struct AWSMachineOrchestrator {
    pub client: ec2::Client,
}

impl MachineOrchestrator for AWSMachineOrchestrator {
    type CreateMachineError = ec2::Error;
    type ListMachinesError = ec2::Error;

    async fn create_machine(&self) -> Result<CreateMachineResponse, Self::CreateMachineError> {
        self.client
            .run_instances()
            .image_id(Ami::AmazonLinux64BitArm.to_string())
            .instance_type(InstanceType::T4gMicro)
            .min_count(1)
            .max_count(1)
            .tag_specifications(
                TagSpecification::builder()
                    .resource_type(ResourceType::Instance)
                    .tags(Tag::builder().key("Name").value("Something").build())
                    .tags(Tag::builder().key("Service").value("load_balancer").build())
                    .build(),
            )
            .send()
            .await
            .map(|x| {
                let instance_list = x.instances();
                let instance: &ec2::types::Instance = instance_list.first().unwrap();

                Ok(CreateMachineResponse(Machine::from(instance)))
            })?
    }

    async fn list_machines(&self) -> Result<ListMachinesResponse, Self::ListMachinesError> {
        self.client
            .describe_instances()
            .filters(
                Filter::builder()
                    .name("tag:Service")
                    .values("load_balancer")
                    .build(),
            )
            .max_results(50)
            .send()
            .await
            .map(|x| {
                let machines: Vec<Machine> = x
                    .reservations()
                    .into_iter()
                    .flat_map(|g| -> Vec<Machine> {
                        g.instances()
                            .into_iter()
                            .map(|f| Machine::from(f))
                            .collect()
                    })
                    .collect();

                Ok(ListMachinesResponse { machines })
            })?
    }
}

impl Service for AWSMachineService {
    fn name(&self) -> &str {
        "machine_service"
    }

    fn start_service<'life0, 'async_trait>(
        &'life0 mut self,
        #[cfg(unix)] _fds: Option<pingora::server::ListenFds>,
        _shutdown: pingora::server::ShutdownWatch,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let fut = async move {
            let client = create_ec2_client().await;
            let orch = AWSMachineOrchestrator { client };

            loop {
                tokio::select! {
                    _ = sleep(self.polling_interval)=>{

                        let should_create_machine = true;

                        if should_create_machine {
                            match orch.create_machine().await{
                                Ok(m) => {
                                    info!("Machine created, id=\"{}\"", m.0.id);
                                },
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            };

                        }
                        // Check machine memory
                        // Spin up more machines
                        // Take down unnecessary machines
                        info!("Polling");
                    }
                }
            }
        };

        Box::pin(fut)
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
