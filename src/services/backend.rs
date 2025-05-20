use pingora::{
    lb::{Backend, Extensions},
    protocols::l4::socket::SocketAddr,
};

use super::machine_orchestrator::{Machine, MachineError};

impl TryFrom<Machine> for Backend {
    type Error = MachineError;

    fn try_from(value: Machine) -> Result<Self, Self::Error> {
        let ip_with_port = format!("{}:80", value.ip_address);
        Ok(Backend {
            // If we don't give this a port, it gets parsed a SocketAddr::Unix
            addr: ip_with_port
                .parse::<SocketAddr>()
                .map_err(|_| MachineError("Could not parse IP address"))?,
            weight: 1,
            ext: Extensions::new(),
        })
    }
}
