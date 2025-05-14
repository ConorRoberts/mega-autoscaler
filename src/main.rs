mod lb;
mod services;

use env_logger;
use lb::LB;
use pingora::lb::{Backends, LoadBalancer};
use pingora::prelude::*;
use services::aws::aws_machine_service::AWSMachineService;
use services::aws::aws_service_discovery::AWSServiceDiscovery;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    env_logger::init();

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let backends = Backends::new(Box::new(AWSServiceDiscovery));
    let load_balancer = LoadBalancer::from_backends(backends);
    let mut lb = http_proxy_service(&my_server.configuration, LB(Arc::new(load_balancer)));

    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(lb);

    let machine_service = AWSMachineService {
        polling_interval: Duration::from_secs(5),
    };
    my_server.add_service(machine_service);

    my_server.run_forever();
}
