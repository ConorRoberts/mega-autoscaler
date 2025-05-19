use aws_sdk_ec2::types::Instance;

pub struct Machine {
    pub id: String,
    pub ip_address: String,
}

#[derive(Debug)]
pub struct MachineError(pub &'static str);

impl TryFrom<&Instance> for Machine {
    type Error = MachineError;

    fn try_from(value: &Instance) -> Result<Self, MachineError> {
        let instance_id = value
            .instance_id
            .as_ref()
            .ok_or(MachineError("instance_id missing"))?;
        let ip_address = value
            .public_ip_address
            .as_ref()
            .ok_or(MachineError("ip_address missing"))?;

        Ok(Self {
            id: instance_id.into(),
            ip_address: ip_address.into(),
        })
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

    /// Gets a list of running machines that can be used to serve traffic.
    async fn list_machines(&self) -> Result<ListMachinesResponse, Self::ListMachinesError>;
}
