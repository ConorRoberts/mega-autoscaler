use super::super::machine_orchestrator::{
    CreateMachineResponse, ListMachinesResponse, Machine, MachineOrchestrator,
};
use super::utils::create_ec2_client;
use aws_sdk_ec2 as ec2;
use aws_sdk_ec2::types::{
    Filter, InstanceStateName, InstanceType, ResourceType, Tag, TagSpecification,
};
use base64::{Engine as _, engine::general_purpose};
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
    pub docker_image: String,
}

fn create_user_data(docker_image: &String) -> String {
    let formatted_string = format!(
        "
        #!/bin/bash
        yum update -y

        docker run -d --restart=always -p 80:80 {}

        # Wait for HTTP service to be responsive
        until curl -s localhost >/dev/null; do
            sleep 1
        done
    ",
        docker_image
    );

    let encoded = general_purpose::STANDARD.encode(formatted_string);

    return encoded;
}

impl MachineOrchestrator for AWSMachineOrchestrator {
    type CreateMachineError = ec2::Error;
    type ListMachinesError = ec2::Error;

    async fn create_machine(&self) -> Result<CreateMachineResponse, Self::CreateMachineError> {
        let instance_id: String = self
            .client
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
            .user_data(create_user_data(&self.docker_image))
            .send()
            .await
            .map(|x| {
                let instance_list = x.instances();
                let instance = instance_list.into_iter().next().unwrap();

                let id = instance.instance_id.as_ref().unwrap();

                id.into()
            })?;

        let running_instance = self
            .client
            .describe_instances()
            .instance_ids(&instance_id)
            .send()
            .await?
            .reservations
            .unwrap_or_default()
            .into_iter()
            .flat_map(|r| r.instances.unwrap_or_default())
            .next()
            .unwrap();

        let instance = loop {
            info!("Polling for new instance");

            let running_instance = self
                .client
                .describe_instances()
                .instance_ids(&instance_id)
                .send()
                .await?
                .reservations
                .unwrap_or_default()
                .into_iter()
                .flat_map(|r| r.instances.unwrap_or_default())
                .next()
                .unwrap();

            info!("{:?}", running_instance);

            if let Some(state) = running_instance.state().unwrap().name() {
                if *state == InstanceStateName::Running {
                    break running_instance.clone();
                }
            }

            // info!("{} count", res.reservations.iter().count());

            // if let Some(reservations) = res.reservations {
            //     if let Some(instance) = reservations
            //         .iter()
            //         .flat_map(|r| {
            //             let instances = r.instances.as_ref();

            //             instances.unwrap()
            //         })
            //         .find(|i| {
            //             matches!(
            //                 i.state().and_then(|s| s.name()),
            //                 Some(InstanceStateName::Running)
            //             )
            //         })
            //     {
            //         break instance.clone(); // Clone to move out of the loop
            //     }
            // }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        };

        info!("Got new instance");

        Ok(CreateMachineResponse(Machine::try_from(&instance).unwrap()))
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
                // TODO PUT BACK
                // let machines: Vec<Machine> = x
                //     .reservations()
                //     .into_iter()
                //     .flat_map(|g| -> Vec<Machine> {
                //         g.instances()
                //             .into_iter()
                //             .map(|f| Machine::try_from(f).unwrap())
                //             .collect()
                //     })
                //     .collect();

                Ok(ListMachinesResponse {
                    machines: vec![Machine {
                        id: "something".into(),
                        ip_address: "1.1.1.1".into(),
                    }],
                })
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
            let orch = AWSMachineOrchestrator {
                client,
                docker_image: "nginx:latest".into(),
            };

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
