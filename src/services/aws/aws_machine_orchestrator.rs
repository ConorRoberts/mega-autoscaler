use super::super::machine_orchestrator::{
    CreateMachineResponse, ListMachinesResponse, Machine, MachineOrchestrator,
};
use super::aws_machine_user_data::AWSMachineUserData;
use crate::services::aws::aws_ami::Ami;
use crate::services::aws::utils::{wait_for_healthy_machine, wait_for_running_machine};
use aws_sdk_ec2 as ec2;
use aws_sdk_ec2::types::{Filter, InstanceType, ResourceType, Tag, TagSpecification};
use log::info;

pub struct AWSMachineOrchestrator {
    pub client: ec2::Client,
    pub user_data: AWSMachineUserData,
}

// # Wait for HTTP service to be responsive
// until curl -s localhost >/dev/null; do
//     sleep 1
// done

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
            .user_data(self.user_data.to_string())
            .send()
            .await
            .map(|x| {
                let instance_list = x.instances();
                let instance = instance_list.into_iter().next().unwrap();

                let id = instance.instance_id.as_ref().unwrap();

                id.into()
            })
            .map_err(|e| e.to_string())?;

        // Wait for instance to be running
        let instance = wait_for_running_machine(&self.client, &instance_id).await?;

        let http_client = reqwest::Client::new();

        if let Some(ip) = &instance.public_ip_address {
            wait_for_healthy_machine(http_client, ip).await;
        }

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
