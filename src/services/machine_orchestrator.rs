use aws_sdk_ec2::types::Instance;

pub struct Machine {
    pub id: String,
    pub ip_address: String,
}

impl From<&Instance> for Machine {
    fn from(value: &Instance) -> Self {
        let instance_id = value.instance_id.as_ref().unwrap();
        let ip_address = value.ipv6_address().unwrap();

        Self {
            id: instance_id.into(),
            ip_address: ip_address.into(),
        }
    }
}

pub struct CreateMachineResponse(pub Machine);

pub struct ListMachinesResponse {
    pub machines: Vec<Machine>,
}

pub trait MachineOrchestrator {
    type CreateMachineError;
    type ListMachinesError;

    async fn create_machine(&self) -> Result<CreateMachineResponse, Self::CreateMachineError>;

    async fn list_machines(&self) -> Result<ListMachinesResponse, Self::ListMachinesError>;
}
