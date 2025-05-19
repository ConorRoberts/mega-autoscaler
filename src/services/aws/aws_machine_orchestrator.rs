use super::super::machine_orchestrator::{
    CreateMachineResponse, ListMachinesResponse, Machine, MachineOrchestrator,
};
use crate::services::aws::aws_ami::Ami;
use aws_sdk_ec2 as ec2;
use aws_sdk_ec2::types::{
    Filter, InstanceStateName, InstanceType, ResourceType, Tag, TagSpecification,
};
use base64::{Engine as _, engine::general_purpose};
use log::info;

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
    type CreateMachineError = String;
    type ListMachinesError = String;

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
            })
            .map_err(|e| e.to_string())?;

        let instance = loop {
            let running_instance = self
                .client
                .describe_instances()
                .instance_ids(&instance_id)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .reservations
                .into_iter()
                .flatten()
                .flat_map(|r| r.instances.unwrap_or_default())
                .next()
                .unwrap();

            if let Some(state) = running_instance.state().unwrap().name() {
                if *state == InstanceStateName::Running {
                    break running_instance;
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        };

        info!("Got new instance");

        Ok(CreateMachineResponse(
            Machine::try_from(&instance).map_err(|e| e.0)?,
        ))
    }

    async fn list_machines(&self) -> Result<ListMachinesResponse, Self::ListMachinesError> {
        let mut filters: Vec<Filter> = Vec::new();

        filters.push(
            Filter::builder()
                .name("tag:Service")
                .values("load_balancer")
                .build(),
        );

        filters.push(
            Filter::builder()
                .name("instance-state-name")
                .values("running")
                .build(),
        );

        self.client
            .describe_instances()
            .max_results(50)
            .set_filters(Some(filters))
            .send()
            .await
            .map(|x| {
                let machines: Result<Vec<Machine>, _> = x
                    .reservations
                    .into_iter()
                    .flatten()
                    .flat_map(|g| {
                        g.instances
                            .into_iter()
                            .flatten()
                            .map(|h| Machine::try_from(&h).map_err(|e| e.0))
                    })
                    .into_iter()
                    .collect();

                // Ok(ListMachinesResponse {
                //     machines: vec![Machine {
                //         id: "something".into(),
                //         ip_address: "1.1.1.1".into(),
                //     }],
                // })

                Ok(ListMachinesResponse {
                    machines: machines?,
                })
            })
            .map_err(|e| e.to_string())?
    }
}
