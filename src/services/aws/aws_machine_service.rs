use super::super::machine_orchestrator::MachineOrchestrator;
use super::aws_machine_orchestrator::AWSMachineOrchestrator;
use super::utils::create_ec2_client;
use log::{error, info};
use pingora::prelude::*;
use pingora::services::Service;
use std::time::Duration;

/// Background process to monitor machines
pub struct AWSMachineService {
    pub polling_interval: Duration,
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
