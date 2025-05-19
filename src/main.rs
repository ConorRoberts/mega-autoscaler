mod lb;
mod services;

use env_logger;
use lb::LB;
use pingora::lb::selection::weighted::Weighted;
use pingora::lb::{Backends, LoadBalancer};
use pingora::prelude::*;
use pingora::services::background;
// use services::aws::aws_machine_service::AWSMachineService;
use services::aws::aws_service_discovery::AWSServiceDiscovery;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    env_logger::init();

    let opt = Opt::default();

    let mut my_server = Server::new(Some(opt)).unwrap();
    my_server.bootstrap();

    let bg = background_service("discovery", AWSServiceDiscovery);
    let t = bg.task();
    // let backends = Backends::new(Box::new(AWSServiceDiscovery));

    let upstreams: Vec<&'static str> = vec![];
    let mut load_balancer = LoadBalancer::try_from_iter(upstreams).unwrap();
    // load_balancer.update_frequency = Some(Duration::from_secs(5));

    let mut lb = http_proxy_service(&my_server.configuration, LB(Arc::new(load_balancer)));

    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(lb);

    // let machine_service = AWSMachineService {
    //     polling_interval: Duration::from_secs(5),
    // };
    // my_server.add_service(machine_service);

    my_server.run_forever();
}
